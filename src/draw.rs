use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;

pub const PIXOO_FRAME_WIDTH: usize = 64;
pub const PIXOO_FRAME_HEIGHT: usize = 64;
pub const PIXOO_PIXEL_BYTES: usize = 3;
pub const PIXOO_FRAME_LEN: usize = PIXOO_FRAME_WIDTH * PIXOO_FRAME_HEIGHT * PIXOO_PIXEL_BYTES;

pub fn uniform_pixel_buffer(red: u8, green: u8, blue: u8) -> Vec<u8> {
    let mut buffer = vec![0u8; PIXOO_FRAME_LEN];
    for chunk in buffer.chunks_exact_mut(PIXOO_PIXEL_BYTES) {
        chunk[0] = red;
        chunk[1] = green;
        chunk[2] = blue;
    }
    buffer
}

/// Encode a 64x64 RGB pixel buffer into Base64 `PicData`.
///
/// # Errors
///
/// Returns an error when the buffer length does not match the expected
/// 64x64x3 byte size.
pub fn encode_pic_data(pixels: &[u8]) -> Result<String, String> {
    if pixels.len() != PIXOO_FRAME_LEN {
        return Err(format!(
            "expected {PIXOO_FRAME_LEN} bytes, got {}",
            pixels.len()
        ));
    }
    Ok(STANDARD_NO_PAD.encode(pixels))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_buffer_sets_expected_bytes() {
        let buffer = uniform_pixel_buffer(255, 0, 128);
        assert_eq!(buffer.len(), PIXOO_FRAME_LEN);
        assert_eq!(&buffer[0..3], &[255, 0, 128]);
        let tail = &buffer[PIXOO_FRAME_LEN - 3..PIXOO_FRAME_LEN];
        assert_eq!(tail, &[255, 0, 128]);
    }

    #[test]
    fn encode_pic_data_black_buffer_is_all_a() {
        let buffer = uniform_pixel_buffer(0, 0, 0);
        let encoded = encode_pic_data(&buffer).expect("encoded");
        let expected_len = (PIXOO_FRAME_LEN / 3) * 4;
        let expected = "A".repeat(expected_len);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_pic_data_white_buffer_is_all_slash() {
        let buffer = uniform_pixel_buffer(255, 255, 255);
        let encoded = encode_pic_data(&buffer).expect("encoded");
        let expected_len = (PIXOO_FRAME_LEN / 3) * 4;
        let expected = "/".repeat(expected_len);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_pic_data_rejects_invalid_length() {
        let err = encode_pic_data(&[0u8; 10]).expect_err("expected length error");
        assert!(err.contains("expected"));
    }
}
