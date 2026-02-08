/// Pixoo display width in pixels.
pub const PIXOO_FRAME_WIDTH: usize = 64;

/// Pixoo display height in pixels.
pub const PIXOO_FRAME_HEIGHT: usize = 64;

/// Bytes per pixel (RGB).
pub const PIXOO_PIXEL_BYTES: usize = 3;

/// Total bytes for a single frame (64 × 64 × 3).
pub const PIXOO_FRAME_LEN: usize = PIXOO_FRAME_WIDTH * PIXOO_FRAME_HEIGHT * PIXOO_PIXEL_BYTES;

/// Creates a uniform pixel buffer where every pixel has the same RGB color.
pub fn uniform_pixel_buffer(red: u8, green: u8, blue: u8) -> Vec<u8> {
    let mut buffer = vec![0u8; PIXOO_FRAME_LEN];
    for chunk in buffer.chunks_exact_mut(PIXOO_PIXEL_BYTES) {
        chunk[0] = red;
        chunk[1] = green;
        chunk[2] = blue;
    }
    buffer
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
}
