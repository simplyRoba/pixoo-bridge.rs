use crate::pixels::{
    decode_upload, encode_pic_data, uniform_pixel_buffer, DecodedFrame, ImageError, PIXOO_FRAME_DIM,
};
use crate::pixoo::{map_pixoo_error, PixooClient, PixooCommand};
use crate::remote::RemoteFetchError;
use crate::state::AppState;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::sync::Arc;
use tracing::error;
use validator::Validate;

use super::common::{
    internal_server_error, json_error, service_unavailable, validation_error_simple, ValidatedJson,
};

const SINGLE_FRAME_PIC_SPEED_MS: u32 = 9999;

pub fn mount_draw_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/draw/fill", post(draw_fill))
        .route("/draw/upload", post(draw_upload))
        .route("/draw/remote", post(draw_remote))
}

#[derive(Debug, Deserialize, Validate)]
struct DrawFillRequest {
    #[validate(range(min = 0, max = 255))]
    red: u16,
    #[validate(range(min = 0, max = 255))]
    green: u16,
    #[validate(range(min = 0, max = 255))]
    blue: u16,
}

#[derive(Debug, Deserialize, Validate)]
struct DrawRemoteRequest {
    #[validate(custom(function = "validate_remote_link"))]
    link: String,
}

fn validate_remote_link(link: &str) -> Result<(), validator::ValidationError> {
    let url = reqwest::Url::parse(link).map_err(|_| {
        let mut error = validator::ValidationError::new("invalid_url");
        error.message = Some("link must be an absolute http or https url".into());
        error
    })?;

    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        let mut error = validator::ValidationError::new("invalid_scheme");
        error.message = Some("link must be an absolute http or https url".into());
        return Err(error);
    }

    if url.host_str().is_none() {
        let mut error = validator::ValidationError::new("invalid_host");
        error.message = Some("link must be an absolute http or https url".into());
        return Err(error);
    }

    Ok(())
}

#[tracing::instrument(skip(state, payload))]
async fn draw_fill(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<DrawFillRequest>,
) -> Response {
    let client = &state.pixoo_client;

    let Ok(red) = u8::try_from(payload.red) else {
        return internal_server_error("invalid red value");
    };
    let Ok(green) = u8::try_from(payload.green) else {
        return internal_server_error("invalid green value");
    };
    let Ok(blue) = u8::try_from(payload.blue) else {
        return internal_server_error("invalid blue value");
    };

    let buffer = uniform_pixel_buffer(red, green, blue);
    let pic_data = match encode_pic_data(&buffer) {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "failed to encode draw payload");
            return internal_server_error("failed to encode draw payload");
        }
    };

    let pic_id = match get_next_pic_id(client).await {
        Ok(value) => value,
        Err(resp) => return resp,
    };

    send_draw_frame(client, pic_id, 1, 0, SINGLE_FRAME_PIC_SPEED_MS, pic_data).await
}

#[tracing::instrument(skip(state, multipart))]
async fn draw_upload(State(state): State<Arc<AppState>>, mut multipart: Multipart) -> Response {
    // Extract the file field from the multipart request
    let (bytes, content_type) = match extract_file_field(&mut multipart).await {
        Ok(result) => result,
        Err(resp) => return resp,
    };

    // Check file size against configured limit
    if bytes.len() > state.max_image_size {
        return payload_too_large(state.max_image_size, bytes.len());
    }

    let client = &state.pixoo_client;
    let frames = match decode_frames(&bytes, content_type.as_deref(), "file") {
        Ok(frames) => frames,
        Err(resp) => return resp,
    };

    send_frames(client, frames, state.animation_speed_factor).await
}

#[tracing::instrument(skip(state, payload))]
async fn draw_remote(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<DrawRemoteRequest>,
) -> Response {
    let asset = match state.remote_fetcher.fetch(&payload.link).await {
        Ok(asset) => asset,
        Err(RemoteFetchError::TooLarge { limit, actual }) => {
            return payload_too_large(limit, actual);
        }
        Err(err) => {
            error!(error = %err, "remote fetch failed");
            return remote_fetch_failed(&err.to_string());
        }
    };

    let frames = match decode_frames(&asset.bytes, asset.content_type.as_deref(), "link") {
        Ok(frames) => frames,
        Err(resp) => return resp,
    };

    let client = &state.pixoo_client;
    send_frames(client, frames, state.animation_speed_factor).await
}

async fn extract_file_field(
    multipart: &mut Multipart,
) -> Result<(Vec<u8>, Option<String>), Response> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            let content_type = field.content_type().map(String::from);
            let bytes = field.bytes().await.map_err(|err| {
                let message = err.to_string();
                validation_error_simple("file", &message)
            })?;

            if bytes.is_empty() {
                return Err(validation_error_simple("file", "file is empty"));
            }

            return Ok((bytes.to_vec(), content_type));
        }
    }

    Err(validation_error_simple("file", "missing file field"))
}

#[allow(clippy::result_large_err)]
fn decode_frames(
    bytes: &[u8],
    content_type: Option<&str>,
    field: &str,
) -> Result<Vec<DecodedFrame>, Response> {
    let frames = match decode_upload(bytes, content_type) {
        Ok(frames) => frames,
        Err(ImageError::UnsupportedFormat) => {
            return Err(validation_error_simple(field, "unsupported image format"));
        }
        Err(ImageError::DecodeFailed(msg)) => {
            error!(error = %msg, "failed to process image");
            return Err(validation_error_simple(field, "failed to process image"));
        }
    };

    if frames.is_empty() {
        return Err(validation_error_simple(field, "image contains no frames"));
    }

    Ok(frames)
}

async fn get_next_pic_id(client: &PixooClient) -> Result<i64, Response> {
    let response = match client
        .send_command(&PixooCommand::DrawGetGifId, Map::new())
        .await
    {
        Ok(response) => response,
        Err(err) => {
            let (status, body) = map_pixoo_error(&err, "Pixoo draw id command");
            error!(error = ?err, status = %status, "Pixoo draw id command failed");
            return Err((status, body).into_response());
        }
    };

    let Some(value) = response.get("PicId") else {
        error!(response = ?response, "missing PicId in draw response");
        return Err(service_unavailable());
    };

    let parsed = match value {
        Value::Number(number) => number
            .as_i64()
            .or_else(|| number.as_u64().and_then(|v| i64::try_from(v).ok()))
            .ok_or_else(|| "PicId is not an integer".to_string()),
        Value::String(text) => text
            .parse::<i64>()
            .map_err(|_| "PicId is not an integer".to_string()),
        _ => Err("PicId is not an integer".to_string()),
    };

    match parsed {
        Ok(value) => Ok(value),
        Err(err) => {
            error!(error = %err, response = ?response, "invalid PicId in draw response");
            Err(service_unavailable())
        }
    }
}

async fn send_draw_frame(
    client: &PixooClient,
    pic_id: i64,
    pic_num: u32,
    pic_offset: u32,
    pic_speed: u32,
    pic_data: String,
) -> Response {
    let mut args = Map::new();
    args.insert("PicId".to_string(), Value::from(pic_id));
    args.insert("PicNum".to_string(), Value::from(pic_num));
    args.insert("PicOffset".to_string(), Value::from(pic_offset));
    args.insert("PicWidth".to_string(), Value::from(PIXOO_FRAME_DIM));
    args.insert("PicSpeed".to_string(), Value::from(pic_speed));
    args.insert("PicData".to_string(), Value::String(pic_data));

    match client.send_command(&PixooCommand::DrawSendGif, args).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let (status, body) = map_pixoo_error(&err, "Pixoo draw send command");
            error!(error = ?err, status = %status, "Pixoo draw send command failed");
            (status, body).into_response()
        }
    }
}

async fn send_frames(
    client: &PixooClient,
    frames: Vec<DecodedFrame>,
    speed_factor: f64,
) -> Response {
    let pic_id = match get_next_pic_id(client).await {
        Ok(value) => value,
        Err(resp) => return resp,
    };

    // Frame count is capped at 60, so this conversion is safe.
    let pic_num = u32::try_from(frames.len()).unwrap();

    for (offset, frame) in frames.iter().enumerate() {
        let pic_data = match encode_pic_data(&frame.rgb_buffer) {
            Ok(value) => value,
            Err(err) => {
                error!(error = %err, frame = offset, "failed to encode frame");
                return internal_server_error("failed to encode frame");
            }
        };

        let pic_speed = if frame.delay_ms == 0 {
            SINGLE_FRAME_PIC_SPEED_MS
        } else {
            // f64::from(u32) is lossless; speed_factor > 0 is validated at config time
            let speed = (f64::from(frame.delay_ms) * speed_factor).round().max(1.0);
            // Saturating cast: guaranteed ≥ 1.0; values > u32::MAX saturate to u32::MAX
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let ms = speed as u32;
            ms
        };

        // offset is max 59, so this conversion is safe
        let pic_offset = u32::try_from(offset).unwrap();
        let resp = send_draw_frame(client, pic_id, pic_num, pic_offset, pic_speed, pic_data).await;
        if resp.status() != StatusCode::OK {
            return resp;
        }
    }

    StatusCode::OK.into_response()
}

fn payload_too_large(limit: usize, actual: usize) -> Response {
    json_error(StatusCode::PAYLOAD_TOO_LARGE, "file too large")
        .limit_actual(limit, actual)
        .finish()
}

fn remote_fetch_failed(message: &str) -> Response {
    json_error(StatusCode::SERVICE_UNAVAILABLE, "remote fetch failed")
        .message(message)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::mount_draw_routes;
    use super::SINGLE_FRAME_PIC_SPEED_MS;
    use crate::pixels::{encode_pic_data, uniform_pixel_buffer};
    use crate::pixoo::{PixooClient, PixooClientConfig};
    use crate::remote::{RemoteFetchConfig, RemoteFetcher};
    use crate::routes::common::testing::send_json_request;
    use crate::state::AppState;
    use axum::body::{to_bytes, Body};
    use axum::extract::State as AxumState;
    use axum::http::{Method, Request, StatusCode};
    use axum::routing::post as axum_post;
    use axum::{Json, Router};
    use httpmock::{Method as MockMethod, MockServer};
    use serde_json::{json, Value};
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct PixooMockState {
        requests: Arc<Mutex<Vec<Value>>>,
    }

    async fn pixoo_mock_handler(
        AxumState(state): AxumState<PixooMockState>,
        Json(body): Json<Value>,
    ) -> (StatusCode, Json<Value>) {
        state.requests.lock().unwrap().push(body.clone());
        let command = body
            .get("Command")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if command == "Draw/GetHttpGifId" {
            (
                StatusCode::OK,
                Json(json!({ "error_code": 0, "PicId": 42 })),
            )
        } else {
            (StatusCode::OK, Json(json!({ "error_code": 0 })))
        }
    }

    async fn start_pixoo_mock() -> (String, Arc<Mutex<Vec<Value>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let state = PixooMockState {
            requests: requests.clone(),
        };
        let app = Router::new()
            .route("/post", axum_post(pixoo_mock_handler))
            .with_state(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server");
        });
        (format!("http://{addr}"), requests)
    }

    fn build_draw_app(state: Arc<AppState>) -> Router {
        mount_draw_routes(Router::new()).with_state(state)
    }

    #[tokio::test]
    async fn draw_fill_sends_expected_command_sequence() {
        let (base_url, requests) = start_pixoo_mock().await;
        let client = PixooClient::new(base_url, PixooClientConfig::default()).expect("client");
        let app = build_draw_app(Arc::new(AppState::with_client(client)));

        let (status, body) = send_json_request(
            &app,
            Method::POST,
            "/draw/fill",
            Some(json!({ "red": 32, "green": 128, "blue": 16 })),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert!(body.is_empty());

        let captured = requests.lock().unwrap();
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[0]["Command"], "Draw/GetHttpGifId");
        assert_eq!(captured[1]["Command"], "Draw/SendHttpGif");
        assert_eq!(captured[1]["PicId"], 42);
        assert_eq!(captured[1]["PicNum"], 1);
        assert_eq!(captured[1]["PicOffset"], 0);
        assert_eq!(captured[1]["PicWidth"], 64);
        assert_eq!(captured[1]["PicSpeed"], SINGLE_FRAME_PIC_SPEED_MS);
        let expected_buffer = uniform_pixel_buffer(32, 128, 16);
        let expected_pic_data = encode_pic_data(&expected_buffer).expect("picdata");
        assert_eq!(captured[1]["PicData"], expected_pic_data);
    }

    #[tokio::test]
    async fn draw_fill_rejects_invalid_payload() {
        let (base_url, _requests) = start_pixoo_mock().await;
        let client = PixooClient::new(base_url, PixooClientConfig::default()).expect("client");
        let app = build_draw_app(Arc::new(AppState::with_client(client)));
        let (status, body) = send_json_request(
            &app,
            Method::POST,
            "/draw/fill",
            Some(json!({ "red": 999, "green": 0, "blue": 0 })),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert!(json_body["details"]["red"].is_array());
    }

    // --- draw_upload tests ---

    use image::codecs::gif::GifEncoder;
    use image::{DynamicImage, Frame, ImageBuffer, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;
    use std::time::Duration;

    fn create_test_png() -> Vec<u8> {
        let img: RgbaImage = ImageBuffer::from_fn(16, 16, |_, _| Rgba([255, 0, 0, 255]));
        let mut buf = Vec::new();
        DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .expect("write png");
        buf
    }

    fn create_test_gif(frame_count: usize) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut encoder = GifEncoder::new(&mut buf);
            encoder
                .set_repeat(image::codecs::gif::Repeat::Infinite)
                .unwrap();
            for i in 0..frame_count {
                let v = ((i * 10) % 256) as u8;
                let img: RgbaImage = ImageBuffer::from_fn(8, 8, |_, _| Rgba([v, v, v, 255]));
                let frame = Frame::from_parts(
                    img,
                    0,
                    0,
                    image::Delay::from_saturating_duration(Duration::from_millis(100)),
                );
                encoder.encode_frame(frame).expect("encode frame");
            }
        }
        buf
    }

    fn multipart_body(field_name: &str, content_type: &str, data: &[u8]) -> (String, Vec<u8>) {
        let boundary = "----TestBoundary12345";
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"{field_name}\"; filename=\"test.bin\"\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(format!("Content-Type: {content_type}\r\n\r\n").as_bytes());
        body.extend_from_slice(data);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        (format!("multipart/form-data; boundary={boundary}"), body)
    }

    fn multipart_body_no_file() -> (String, Vec<u8>) {
        let boundary = "----TestBoundary12345";
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"other\"\r\n\r\nsome value\r\n",
        );
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        (format!("multipart/form-data; boundary={boundary}"), body)
    }

    async fn send_multipart_request(
        app: &Router,
        content_type_header: &str,
        body: Vec<u8>,
    ) -> (StatusCode, String) {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/draw/upload")
            .header("content-type", content_type_header)
            .body(Body::from(body))
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        let status = response.status();
        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_default();
        (status, String::from_utf8_lossy(&body_bytes).to_string())
    }

    fn upload_test_state(base_url: String) -> Arc<AppState> {
        let client = PixooClient::new(base_url, PixooClientConfig::default()).expect("client");
        Arc::new(AppState::with_client(client))
    }

    fn remote_test_state(base_url: String, max_image_size: usize) -> Arc<AppState> {
        let client = PixooClient::new(base_url, PixooClientConfig::default()).expect("client");
        let remote_fetcher = RemoteFetcher::new(RemoteFetchConfig::new(
            Duration::from_millis(5_000),
            max_image_size,
        ))
        .expect("remote fetcher");
        Arc::new(AppState {
            health_forward: false,
            pixoo_client: client,
            animation_speed_factor: 1.4,
            max_image_size,
            remote_fetcher,
        })
    }

    async fn send_remote_request(app: &Router, link: &str) -> (StatusCode, String) {
        send_json_request(
            app,
            Method::POST,
            "/draw/remote",
            Some(json!({ "link": link })),
        )
        .await
    }

    #[tokio::test]
    async fn upload_static_png_sends_single_frame() {
        let (base_url, requests) = start_pixoo_mock().await;
        let app = build_draw_app(upload_test_state(base_url));

        let png_data = create_test_png();
        let (ct, body) = multipart_body("file", "image/png", &png_data);
        let (status, _) = send_multipart_request(&app, &ct, body).await;

        assert_eq!(status, StatusCode::OK);

        let captured = requests.lock().unwrap();
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[0]["Command"], "Draw/GetHttpGifId");
        assert_eq!(captured[1]["Command"], "Draw/SendHttpGif");
        assert_eq!(captured[1]["PicNum"], 1);
        assert_eq!(captured[1]["PicOffset"], 0);
        assert_eq!(captured[1]["PicWidth"], 64);
    }

    #[tokio::test]
    async fn upload_animated_gif_sends_multiple_frames() {
        let (base_url, requests) = start_pixoo_mock().await;
        let app = build_draw_app(upload_test_state(base_url));

        let gif_data = create_test_gif(3);
        let (ct, body) = multipart_body("file", "image/gif", &gif_data);
        let (status, _) = send_multipart_request(&app, &ct, body).await;

        assert_eq!(status, StatusCode::OK);

        let captured = requests.lock().unwrap();
        // 1 GetHttpGifId + 3 SendHttpGif
        assert_eq!(captured.len(), 4);
        assert_eq!(captured[0]["Command"], "Draw/GetHttpGifId");
        for i in 0..3 {
            assert_eq!(captured[i + 1]["Command"], "Draw/SendHttpGif");
            assert_eq!(captured[i + 1]["PicNum"], 3);
            assert_eq!(captured[i + 1]["PicOffset"], i as i64);
            assert_eq!(captured[i + 1]["PicWidth"], 64);
        }
    }

    #[tokio::test]
    async fn upload_missing_file_field_returns_400() {
        let (base_url, _) = start_pixoo_mock().await;
        let app = build_draw_app(upload_test_state(base_url));

        let (ct, body) = multipart_body_no_file();
        let (status, resp_body) = send_multipart_request(&app, &ct, body).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&resp_body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
    }

    #[tokio::test]
    async fn upload_empty_file_returns_400() {
        let (base_url, _) = start_pixoo_mock().await;
        let app = build_draw_app(upload_test_state(base_url));

        let (ct, body) = multipart_body("file", "image/png", b"");
        let (status, resp_body) = send_multipart_request(&app, &ct, body).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&resp_body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
    }

    #[tokio::test]
    async fn upload_oversized_file_returns_413() {
        let (base_url, _) = start_pixoo_mock().await;
        let client = PixooClient::new(base_url, PixooClientConfig::default()).expect("client");
        let mut state = AppState::with_client(client);
        state.max_image_size = 100; // 100 byte limit
        let app = build_draw_app(Arc::new(state));

        let png_data = create_test_png(); // larger than 100 bytes
        let (ct, body) = multipart_body("file", "image/png", &png_data);
        let (status, resp_body) = send_multipart_request(&app, &ct, body).await;

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        let json_body: Value = serde_json::from_str(&resp_body).unwrap();
        assert_eq!(json_body["error"], "file too large");
        assert_eq!(json_body["limit"], 100);
    }

    #[tokio::test]
    async fn upload_unsupported_format_returns_400() {
        let (base_url, _) = start_pixoo_mock().await;
        let app = build_draw_app(upload_test_state(base_url));

        let (ct, body) = multipart_body("file", "image/bmp", b"fake bmp data");
        let (status, resp_body) = send_multipart_request(&app, &ct, body).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&resp_body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert!(json_body["details"]["file"]
            .as_str()
            .unwrap()
            .contains("unsupported image format"));
    }

    #[tokio::test]
    async fn remote_png_download_sends_single_frame() {
        let (base_url, requests) = start_pixoo_mock().await;
        let remote_server = MockServer::start_async().await;
        let png_data = create_test_png();
        remote_server.mock(|when, then| {
            when.method(MockMethod::GET).path("/logo.png");
            then.status(200)
                .header("content-type", "image/png")
                .body(png_data.clone());
        });

        let app = build_draw_app(remote_test_state(base_url, 5 * 1024 * 1024));
        let link = format!("{}/logo.png", remote_server.base_url());
        let (status, body) = send_remote_request(&app, &link).await;

        assert_eq!(status, StatusCode::OK);
        assert!(body.is_empty());

        let captured = requests.lock().unwrap();
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[0]["Command"], "Draw/GetHttpGifId");
        assert_eq!(captured[1]["Command"], "Draw/SendHttpGif");
        assert_eq!(captured[1]["PicNum"], 1);
    }

    #[tokio::test]
    async fn remote_invalid_url_returns_400() {
        let (base_url, _) = start_pixoo_mock().await;
        let app = build_draw_app(remote_test_state(base_url, 5 * 1024 * 1024));

        let (status, body) = send_remote_request(&app, "ftp://example.com/logo.png").await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
    }

    #[tokio::test]
    async fn remote_oversized_payload_returns_413() {
        let (base_url, _) = start_pixoo_mock().await;
        let remote_server = MockServer::start_async().await;
        let payload = vec![1_u8; 200];
        remote_server.mock(|when, then| {
            when.method(MockMethod::GET).path("/big.png");
            then.status(200)
                .header("content-type", "image/png")
                .header("content-length", "200")
                .body(payload.clone());
        });

        let app = build_draw_app(remote_test_state(base_url, 100));
        let link = format!("{}/big.png", remote_server.base_url());
        let (status, body) = send_remote_request(&app, &link).await;

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "file too large");
        assert_eq!(json_body["limit"], 100);
        assert_eq!(json_body["actual"], 200);
    }

    #[tokio::test]
    async fn remote_download_error_returns_503() {
        let (base_url, _) = start_pixoo_mock().await;
        let remote_server = MockServer::start_async().await;
        remote_server.mock(|when, then| {
            when.method(MockMethod::GET).path("/fail.png");
            then.status(500).body("nope");
        });

        let app = build_draw_app(remote_test_state(base_url, 5 * 1024 * 1024));
        let link = format!("{}/fail.png", remote_server.base_url());
        let (status, body) = send_remote_request(&app, &link).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "remote fetch failed");
    }
}
