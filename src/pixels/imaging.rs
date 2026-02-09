use image::codecs::gif::GifDecoder;
use image::codecs::webp::WebPDecoder;
use image::{
    imageops::FilterType, AnimationDecoder, DynamicImage, ImageFormat, ImageReader, RgbaImage,
};
use std::io::Cursor;
use tracing::warn;

use super::{PIXOO_FRAME_DIM, PIXOO_FRAME_LEN};

const MAX_ANIMATION_FRAMES: usize = 60;

pub struct DecodedFrame {
    pub rgb_buffer: Vec<u8>,
    pub delay_ms: u32,
}

#[derive(Debug)]
pub enum ImageError {
    UnsupportedFormat,
    DecodeFailed(String),
}

/// Decodes an uploaded image into one or more frames suitable for the Pixoo display.
///
/// # Errors
///
/// Returns [`ImageError::UnsupportedFormat`] if the format is not JPEG, PNG, WebP, or GIF.
/// Returns [`ImageError::DecodeFailed`] if the image cannot be decoded or processed.
pub fn decode_upload(
    bytes: &[u8],
    content_type: Option<&str>,
) -> Result<Vec<DecodedFrame>, ImageError> {
    let format = detect_format(bytes, content_type)?;

    match format {
        ImageFormat::Gif => decode_animated_gif(bytes),
        ImageFormat::WebP if is_animated_webp(bytes) => decode_animated_webp(bytes),
        _ => decode_static(bytes, format),
    }
}

fn detect_format(bytes: &[u8], content_type: Option<&str>) -> Result<ImageFormat, ImageError> {
    // Try content type first, but skip generic/missing types
    if let Some(ct) = content_type {
        match ct {
            "image/jpeg" => return Ok(ImageFormat::Jpeg),
            "image/png" => return Ok(ImageFormat::Png),
            "image/webp" => return Ok(ImageFormat::WebP),
            "image/gif" => return Ok(ImageFormat::Gif),
            "application/octet-stream" | "" => {} // fall through to magic bytes
            _ => return Err(ImageError::UnsupportedFormat),
        }
    }

    // Fall back to magic byte detection
    ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()
        .and_then(|reader| reader.format())
        .and_then(|fmt| match fmt {
            ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP | ImageFormat::Gif => {
                Some(fmt)
            }
            _ => None,
        })
        .ok_or(ImageError::UnsupportedFormat)
}

fn is_animated_webp(bytes: &[u8]) -> bool {
    WebPDecoder::new(Cursor::new(bytes))
        .map(|dec| dec.has_animation())
        .unwrap_or(false)
}

fn decode_static(bytes: &[u8], format: ImageFormat) -> Result<Vec<DecodedFrame>, ImageError> {
    let img = ImageReader::with_format(Cursor::new(bytes), format)
        .decode()
        .map_err(|err| ImageError::DecodeFailed(err.to_string()))?;

    let frame = resize_and_extract(&img);
    Ok(vec![DecodedFrame {
        rgb_buffer: frame,
        delay_ms: 0,
    }])
}

fn decode_animated_gif(bytes: &[u8]) -> Result<Vec<DecodedFrame>, ImageError> {
    let decoder = GifDecoder::new(Cursor::new(bytes))
        .map_err(|err| ImageError::DecodeFailed(err.to_string()))?;

    decode_animation_frames(decoder)
}

fn decode_animated_webp(bytes: &[u8]) -> Result<Vec<DecodedFrame>, ImageError> {
    let decoder = WebPDecoder::new(Cursor::new(bytes))
        .map_err(|err| ImageError::DecodeFailed(err.to_string()))?;

    decode_animation_frames(decoder)
}

fn decode_animation_frames<'a>(
    decoder: impl AnimationDecoder<'a>,
) -> Result<Vec<DecodedFrame>, ImageError> {
    let all_frames: Vec<_> = decoder
        .into_frames()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ImageError::DecodeFailed(err.to_string()))?;

    let total = all_frames.len();
    if total > MAX_ANIMATION_FRAMES {
        warn!(
            original_frames = total,
            max_frames = MAX_ANIMATION_FRAMES,
            "Animation exceeds maximum frame count; truncating to {} frames",
            MAX_ANIMATION_FRAMES
        );
    }

    let frames = all_frames
        .into_iter()
        .take(MAX_ANIMATION_FRAMES)
        .map(|frame| {
            let (numer, denom) = frame.delay().numer_denom_ms();
            let delay_ms = if denom == 0 { 0 } else { numer / denom };
            let img = DynamicImage::ImageRgba8(frame.into_buffer());
            let rgb_buffer = resize_and_extract(&img);
            DecodedFrame {
                rgb_buffer,
                delay_ms,
            }
        })
        .collect();

    Ok(frames)
}

fn resize_and_extract(img: &DynamicImage) -> Vec<u8> {
    let resized = img.resize_exact(PIXOO_FRAME_DIM, PIXOO_FRAME_DIM, FilterType::Triangle);
    let rgba = resized.to_rgba8();
    composite_to_rgb(&rgba)
}

/// Composites RGBA pixels against a black background and returns flat RGB bytes.
fn composite_to_rgb(rgba: &RgbaImage) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(PIXOO_FRAME_LEN);

    for pixel in rgba.pixels() {
        let [r, g, b, a] = pixel.0;
        rgb.push(premultiply(r, a));
        rgb.push(premultiply(g, a));
        rgb.push(premultiply(b, a));
    }

    rgb
}

/// Premultiplies a color channel by alpha against a black background.
/// Max value: 255 * 255 / 255 = 255, so the result always fits in `u8`.
fn premultiply(channel: u8, alpha: u8) -> u8 {
    // Unwrap is safe: max result is 255 * 255 / 255 = 255
    u8::try_from(u16::from(channel) * u16::from(alpha) / 255).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn load_fixture(name: &str) -> Vec<u8> {
        std::fs::read(fixtures_dir().join(name))
            .unwrap_or_else(|err| panic!("failed to read fixture {name}: {err}"))
    }

    #[test]
    fn decodes_static_jpeg_black() {
        let data = load_fixture("black_100x100.jpg");
        let frames = decode_upload(&data, Some("image/jpeg")).expect("decode");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].rgb_buffer.len(), PIXOO_FRAME_LEN);
        assert_eq!(frames[0].rgb_buffer[0], 0);
        assert_eq!(frames[0].rgb_buffer[1], 0);
        assert_eq!(frames[0].rgb_buffer[2], 0);
    }

    #[test]
    fn decodes_static_jpeg_white() {
        let data = load_fixture("white_100x100.jpg");
        let frames = decode_upload(&data, Some("image/jpeg")).expect("decode");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].rgb_buffer.len(), PIXOO_FRAME_LEN);
        assert_eq!(frames[0].rgb_buffer[0], 255);
        assert_eq!(frames[0].rgb_buffer[1], 255);
        assert_eq!(frames[0].rgb_buffer[2], 255);
    }

    #[test]
    fn decodes_static_png() {
        let data = load_fixture("red_32x32.png");
        let frames = decode_upload(&data, Some("image/png")).expect("decode");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].rgb_buffer.len(), PIXOO_FRAME_LEN);
        // All pixels should be red
        assert_eq!(frames[0].rgb_buffer[0], 255);
        assert_eq!(frames[0].rgb_buffer[1], 0);
        assert_eq!(frames[0].rgb_buffer[2], 0);
    }

    #[test]
    fn decodes_static_webp() {
        let data = load_fixture("white_8x8.webp");
        let frames = decode_upload(&data, Some("image/webp")).expect("decode");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].rgb_buffer.len(), PIXOO_FRAME_LEN);
        // All pixels should be white
        assert_eq!(frames[0].rgb_buffer[0], 255);
        assert_eq!(frames[0].rgb_buffer[1], 255);
        assert_eq!(frames[0].rgb_buffer[2], 255);
    }

    #[test]
    fn decodes_animated_gif_multiple_frames() {
        let data = load_fixture("black_white_animated_100x100_200ms.gif");
        let frames = decode_upload(&data, Some("image/gif")).expect("decode");
        assert!(
            frames.len() >= 2,
            "expected multiple frames, got {}",
            frames.len()
        );
        for frame in &frames {
            assert_eq!(frame.rgb_buffer.len(), PIXOO_FRAME_LEN);
        }
    }

    #[test]
    fn decodes_animated_webp_multiple_frames() {
        let data = load_fixture("black_gray_white_animated_8x8_100ms.webp");
        let frames = decode_upload(&data, Some("image/webp")).expect("decode");
        assert_eq!(frames.len(), 3);
        for frame in &frames {
            assert_eq!(frame.rgb_buffer.len(), PIXOO_FRAME_LEN);
            assert!(frame.delay_ms >= 90 && frame.delay_ms <= 110);
        }
    }

    #[test]
    fn animated_gif_respects_frame_delay() {
        let data = load_fixture("black_white_animated_100x100_1000ms.gif");
        let frames = decode_upload(&data, Some("image/gif")).expect("decode");
        // GIF delay encoding rounds to 10ms units, so 1000ms should come back as ~1000ms
        for frame in &frames {
            assert!(
                frame.delay_ms >= 990 && frame.delay_ms <= 1010,
                "expected ~1000ms, got {}ms",
                frame.delay_ms
            );
        }
    }

    #[test]
    fn truncates_gif_at_60_frames() {
        let data = load_fixture("gray_animated_8x8_50ms_80frames.gif");
        let frames = decode_upload(&data, Some("image/gif")).expect("decode");
        assert_eq!(frames.len(), MAX_ANIMATION_FRAMES);
    }

    #[test]
    fn exactly_60_frames_not_truncated() {
        let data = load_fixture("gray_animated_8x8_50ms_60frames.gif");
        let frames = decode_upload(&data, Some("image/gif")).expect("decode");
        assert_eq!(frames.len(), 60);
    }

    #[test]
    fn rejects_unsupported_format() {
        let result = decode_upload(b"not an image", Some("image/bmp"));
        assert!(matches!(result, Err(ImageError::UnsupportedFormat)));
    }

    #[test]
    fn rejects_corrupt_data() {
        // text.png is a text file with an image extension
        let data = load_fixture("text.png");
        let result = decode_upload(&data, Some("image/png"));
        assert!(matches!(result, Err(ImageError::DecodeFailed(_))));
    }

    #[test]
    fn falls_back_to_magic_bytes_on_missing_content_type() {
        let data = load_fixture("red_32x32.png");
        let frames = decode_upload(&data, None).expect("decode");
        assert_eq!(frames.len(), 1);
    }

    #[test]
    fn falls_back_to_magic_bytes_on_octet_stream() {
        let data = load_fixture("white_8x8.webp");
        let frames = decode_upload(&data, Some("application/octet-stream")).expect("decode");
        assert_eq!(frames.len(), 1);
    }

    #[test]
    fn alpha_composited_against_black() {
        let data = load_fixture("semitransparent_4x4.png");
        let frames = decode_upload(&data, Some("image/png")).expect("decode");
        assert_eq!(frames.len(), 1);
        // RGBA(255, 128, 64, 128), alpha = 128/255 ≈ 0.502
        // R: 255 * 0.502 ≈ 128, G: 128 * 0.502 ≈ 64, B: 64 * 0.502 ≈ 32
        let r = frames[0].rgb_buffer[0];
        let g = frames[0].rgb_buffer[1];
        let b = frames[0].rgb_buffer[2];
        assert!(r > 120 && r < 135, "red={r} expected ~128");
        assert!(g > 58 && g < 70, "green={g} expected ~64");
        assert!(b > 28 && b < 38, "blue={b} expected ~32");
    }
}
