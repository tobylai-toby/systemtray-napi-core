use base64::prelude::{Engine as _, BASE64_STANDARD};
use tray_icon::Icon;
pub fn parse_icon_from_string(icon: String) -> Icon {
  // if base64 dataurl
  if icon.starts_with("data:image/") {
    let base64_marker = "base64,";
    let base64_index = icon
      .find(base64_marker)
      .expect("Invalid data URL: no base64 marker");
    let base64_data = &icon[base64_index + base64_marker.len()..];
    let decoded_bytes = BASE64_STANDARD
      .decode(base64_data)
      .expect("Failed to decode base64 data");
    let mime_type = &icon[5..base64_index - 1];
    let image = image::load_from_memory_with_format(
      &decoded_bytes,
      image::ImageFormat::from_mime_type(mime_type).unwrap_or(image::ImageFormat::Jpeg),
    )
    .expect("Failed to load image from memory");
    let rgba = image.into_rgba8();
    let (w, h) = rgba.dimensions();
    let icon_data = rgba.into_raw();
    return Icon::from_rgba(icon_data, w, h).expect("Failed to create icon from image");
  };
  let image = image::open(icon).expect("Failed to open image");
  let rgba = image.into_rgba8();
  let (w, h) = rgba.dimensions();
  let icon_data = rgba.into_raw();
  Icon::from_rgba(icon_data, w, h).expect("Failed to create icon from image")
}
