use std::cmp::max;
use std::io::Cursor;

use anyhow::Result;
use image::codecs::jpeg::JpegEncoder;
use image::DynamicImage;

fn if_need_thumbnail(w: u32, h: u32) -> bool {
    max(w, h) > 700
}

pub fn to_webp(file: &[u8]) -> Result<Vec<u8>> {
    let mut reader = image::ImageReader::new(Cursor::new(file))
        .with_guessed_format()?.decode()?;

    if if_need_thumbnail(reader.width(), reader.height()) {
        reader = reader.thumbnail(700, 700);
    }

    let w = reader.width();
    let h = reader.height();

    let webp = match &reader {
        DynamicImage::ImageRgb8(_) => {
            webp::Encoder::from_rgb(reader.as_bytes(), w, h)
        }
        DynamicImage::ImageRgba8(_) => {
            webp::Encoder::from_rgba(reader.as_bytes(), w, h)
        }
        _ => {
            let nr = reader.as_rgb8().ok_or(anyhow::anyhow!("TODO: panic message"))?;
            webp::Encoder::from_rgb(nr, w, h)
        }
    };

    Ok(webp.encode(30f32).to_vec())
}

pub fn only_thumbnail(file: &[u8]) -> Result<Vec<u8>> {
    let mut reader = image::ImageReader::new(Cursor::new(file))
        .with_guessed_format()?.decode()?;


    if if_need_thumbnail(reader.width(), reader.height()) {
        reader = reader.thumbnail(700, 700);
    }

    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buf, 30);
    reader.write_with_encoder(encoder)?;

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_webp() {
        let file = include_bytes!("../../test.jpg");
        to_webp(file).expect("TODO: panic message");
    }

    #[test]
    fn test_only_thumbnail() {
        let file = include_bytes!("../../test.jpg");
        only_thumbnail(file).expect("TODO: panic message");
    }
}
