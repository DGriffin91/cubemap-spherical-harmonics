use std::env;

use cubemap_spherical_harmonics::process;

use image::io::Reader as ImageReader;

/// Example usage
fn main() {
    let mut args = env::args();
    args.next();
    let img = ImageReader::open(args.next().unwrap())
        .unwrap()
        .decode()
        .unwrap()
        .into_rgb32f();
    let width = img.width();
    let mut faces = Vec::new();
    for i in 0..6 {
        faces.push(image::imageops::crop_imm(&img, 0, i as u32 * width, width, width).to_image());
    }
    let sh = process(&faces).unwrap();
    println!("{:#?}", sh);
}
