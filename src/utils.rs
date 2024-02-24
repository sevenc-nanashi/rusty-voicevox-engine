use base64::Engine;
use image::ImageEncoder;

pub fn image_to_base64(image: &image::RgbaImage) -> String {
    let mut buf = Vec::new();
    image::codecs::png::PngEncoder::new(&mut buf)
        .write_image(image, image.width(), image.height(), image::ColorType::Rgba8)
        .unwrap();
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(&buf)
}

pub fn process_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(process_path::get_executable_path().unwrap().parent().unwrap())
}
