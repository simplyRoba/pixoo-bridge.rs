mod canvas;
mod encoding;
pub mod imaging;

/// Pixoo display dimension in pixels (the display is square).
pub const PIXOO_FRAME_DIM: u32 = 64;

/// Bytes per pixel (RGB).
pub const PIXOO_PIXEL_BYTES: usize = 3;

/// Total bytes for a single frame (64 × 64 × 3).
pub const PIXOO_FRAME_LEN: usize =
    PIXOO_FRAME_DIM as usize * PIXOO_FRAME_DIM as usize * PIXOO_PIXEL_BYTES;

pub use canvas::uniform_pixel_buffer;
pub use encoding::encode_pic_data;
pub use imaging::{decode_upload, DecodedFrame, ImageError};
