use base64::prelude::*;

pub fn draw_color() {
    let imgx = 64;
    let imgy = 64;

    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // TODO there is fill function for ImageBuffer i do not understand :( maybe in the future

    for (_x, _y, pixel) in imgbuf.enumerate_pixels_mut() {
        // TODO use color provided in call
        *pixel = image::Rgb([255, 255, 255]);
    }

    to_base64(&imgbuf);
}

// TODO traits are like extension functions (as i understood) 
// so this might be a good thing to try with a trait on ImageBuffer
fn to_base64(image: &image::RgbImage) -> String {
    //TODO address this clone it makes me sad :( 
   BASE64_STANDARD.encode(image.clone().into_raw())
}