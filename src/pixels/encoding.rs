use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;

use super::PIXOO_FRAME_LEN;

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
    use crate::pixels::{uniform_pixel_buffer, PIXOO_FRAME_DIM};

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
    fn encode_pic_data_uniform_buffer_matches_spec_example() {
        let buffer = uniform_pixel_buffer(255, 0, 128);
        let encoded = encode_pic_data(&buffer).expect("encoded");
        let mut expected_pixels = Vec::with_capacity(PIXOO_FRAME_LEN);
        for _ in 0..(PIXOO_FRAME_DIM * PIXOO_FRAME_DIM) {
            expected_pixels.extend_from_slice(&[255, 0, 128]);
        }
        let expected = STANDARD_NO_PAD.encode(&expected_pixels);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_pic_data_rejects_invalid_length() {
        let err = encode_pic_data(&[0u8; 10]).expect_err("expected length error");
        assert!(err.contains("expected"));
    }
}
