use std::io::Write;
use image::{
    codecs::jpeg::JpegEncoder,
    codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder},
    DynamicImage, ImageEncoder,
};
use crate::error::CoreError;
use crate::model::{CompressionConfig, OutputFormat};

pub fn encode_image(
    img: &DynamicImage,
    format: OutputFormat,
    compression: &CompressionConfig,
    writer: &mut impl Write,
) -> Result<(), CoreError> {
    match format {
        OutputFormat::Jpeg => {
            let quality = compression.quality.unwrap_or(85).clamp(1, 100);
            let rgb = img.to_rgb8();
            let encoder = JpegEncoder::new_with_quality(writer, quality);
            encoder
                .write_image(
                    rgb.as_raw(),
                    rgb.width(),
                    rgb.height(),
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| CoreError::Encode(e.to_string()))
        }
        OutputFormat::Png => {
            let level = compression.png_compression.unwrap_or(6).min(9);
            let png_encoder = PngEncoder::new_with_quality(
                writer,
                CompressionType::Level(level),
                PngFilter::Adaptive,
            );
            png_encoder
                .write_image(
                    img.to_rgba8().as_raw(),
                    img.width(),
                    img.height(),
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|e| CoreError::Encode(e.to_string()))
        }
        OutputFormat::WebP { .. } => {
            let mut buf = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::WebP)
                .map_err(|e| CoreError::Encode(e.to_string()))?;
            writer.write_all(&buf).map_err(|e| CoreError::Encode(e.to_string()))
        }
        OutputFormat::Avif => {
            let mut buf = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Avif)
                .map_err(|e| CoreError::Encode(e.to_string()))?;
            writer.write_all(&buf).map_err(|e| CoreError::Encode(e.to_string()))
        }
        OutputFormat::Tiff => {
            let mut buf = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Tiff)
                .map_err(|e| CoreError::Encode(e.to_string()))?;
            writer.write_all(&buf).map_err(|e| CoreError::Encode(e.to_string()))
        }
        OutputFormat::Bmp => {
            let mut buf = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Bmp)
                .map_err(|e| CoreError::Encode(e.to_string()))?;
            writer.write_all(&buf).map_err(|e| CoreError::Encode(e.to_string()))
        }
        OutputFormat::Gif => {
            let mut buf = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Gif)
                .map_err(|e| CoreError::Encode(e.to_string()))?;
            writer.write_all(&buf).map_err(|e| CoreError::Encode(e.to_string()))
        }
    }
}