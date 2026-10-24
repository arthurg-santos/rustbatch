pub mod watermark;

use image::{DynamicImage, ImageReader, ImageDecoder}; // ImageDecoder adicionado aqui
use crate::error::CoreError;
use crate::model::*;
use std::path::Path;

pub fn decode_with_orientation(path: &Path) -> Result<DynamicImage, CoreError> {
    let reader = ImageReader::open(path).map_err(|e| CoreError::io(path, e))?
        .with_guessed_format().map_err(|e| CoreError::io(path, e))?;
    let mut decoder = reader.into_decoder().map_err(|e| CoreError::Decode(e.to_string()))?;
    let orientation = decoder.orientation().map_err(|e| CoreError::Decode(e.to_string()))?;
    let mut img = DynamicImage::from_decoder(decoder).map_err(|e| CoreError::CorruptFile {
        path: path.to_path_buf(), source: e,
    })?;
    img.apply_orientation(orientation);
    Ok(img)
}

pub fn process_image(source: &Path, cfg: &ProcessConfig) -> Result<DynamicImage, CoreError> {
    let mut img = decode_with_orientation(source)?;
    if cfg.rotation != Rotation::None { img = apply_rotation(img, cfg.rotation); }
    if let Some(resize) = &cfg.resize { img = apply_resize(img, resize)?; }
    for filter in &cfg.filters { img = apply_filter(img, *filter); }
    if let Some(wm) = &cfg.watermark { img = watermark::apply_watermark(img, wm)?; }
    Ok(img)
}

fn apply_rotation(img: DynamicImage, rot: Rotation) -> DynamicImage {
    match rot {
        Rotation::None => img, Rotation::R90 => img.rotate90(),
        Rotation::R180 => img.rotate180(), Rotation::R270 => img.rotate270(),
        Rotation::FlipH => img.fliph(), Rotation::FlipV => img.flipv(),
    }
}

fn apply_resize(img: DynamicImage, cfg: &ResizeConfig) -> Result<DynamicImage, CoreError> {
    let (w, h) = (img.width(), img.height());
    let (target_w, target_h) = match cfg.mode {
        ResizeMode::Exact { width, height } => (width.get(), height.get()),
        ResizeMode::Percentage(pct) => {
            let factor = pct / 100.0;
            (((w as f32 * factor).round() as u32).max(1), ((h as f32 * factor).round() as u32).max(1))
        }
        ResizeMode::FitWithin { width, height } => {
            let (mw, mh) = (width.get(), height.get());
            let ratio = (mw as f32 / w as f32).min(mh as f32 / h as f32);
            (((w as f32 * ratio).round() as u32).max(1), ((h as f32 * ratio).round() as u32).max(1))
        }
    };

    let (target_w, target_h) = if cfg.no_upscale && (target_w > w || target_h > h) { (w, h) } else { (target_w, target_h) };
    validate_dimensions(target_w, target_h)?;

    if cfg.preserve_aspect {
        let ratio = (target_w as f32 / w as f32).min(target_h as f32 / h as f32);
        let rw = ((w as f32 * ratio).round() as u32).max(1);
        let rh = ((h as f32 * ratio).round() as u32).max(1);
        Ok(img.resize_exact(rw, rh, image::imageops::FilterType::Lanczos3))
    } else {
        Ok(img.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3))
    }
}

fn validate_dimensions(w: u32, h: u32) -> Result<(), CoreError> {
    if w == 0 || h == 0 || w > MAX_DIMENSION || h > MAX_DIMENSION {
        return Err(CoreError::InvalidDimensions { width: w, height: h, max: MAX_DIMENSION });
    }
    if (w as u64 * h as u64) > MAX_PIXELS { return Err(CoreError::TooLarge(w as u64 * h as u64)); }
    Ok(())
}

fn apply_filter(img: DynamicImage, filter: Filter) -> DynamicImage {
    match filter {
        Filter::Brightness(v) => img.brighten((v as f32 / 100.0 * 255.0 / 2.55) as i32),
        Filter::Contrast(c) => img.adjust_contrast(c),
        Filter::Saturation(s) => {
            let mut out = img.to_rgba8(); // PRECISA de mut
            for px in out.pixels_mut() {
                let r = px[0] as f32; let g = px[1] as f32; let b = px[2] as f32;
                let gray = 0.299 * r + 0.587 * g + 0.114 * b;
                px[0] = (gray + (r - gray) * s).clamp(0.0, 255.0) as u8;
                px[1] = (gray + (g - gray) * s).clamp(0.0, 255.0) as u8;
                px[2] = (gray + (b - gray) * s).clamp(0.0, 255.0) as u8;
            }
            DynamicImage::ImageRgba8(out)
        },
        Filter::Grayscale => img.grayscale(),
        Filter::Sepia => {
            let mut out = img.to_rgba8(); // PRECISA de mut
            for px in out.pixels_mut() {
                let r = px[0] as f32; let g = px[1] as f32; let b = px[2] as f32;
                px[0] = (0.393 * r + 0.769 * g + 0.189 * b).min(255.0) as u8;
                px[1] = (0.349 * r + 0.686 * g + 0.168 * b).min(255.0) as u8;
                px[2] = (0.272 * r + 0.534 * g + 0.131 * b).min(255.0) as u8;
            }
            DynamicImage::ImageRgba8(out)
        }
        Filter::Blur(sigma) => DynamicImage::ImageRgba8(imageproc::filter::gaussian_blur_f32(&img.to_rgba8(), sigma)),
        Filter::Sharpen => {
            let out = img.to_rgba8(); // NÃO PRECISA de mut (usado como read-only na clonagem)
            let (w, h) = out.dimensions();
            let mut result = out.clone();
            for y in 1..h-1 {
                for x in 1..w-1 {
                    let mut sum = [0i32; 3];
                    for dy in 0..3 {
                        for dx in 0..3 {
                            let px = out.get_pixel(x + dx - 1, y + dy - 1);
                            let weight = if dx == 1 && dy == 1 { 5 } else { -1 };
                            sum[0] += px[0] as i32 * weight;
                            sum[1] += px[1] as i32 * weight;
                            sum[2] += px[2] as i32 * weight;
                        }
                    }
                    let alpha = out.get_pixel(x, y)[3];
                    result.put_pixel(x, y, image::Rgba([
                        sum[0].clamp(0, 255) as u8,
                        sum[1].clamp(0, 255) as u8,
                        sum[2].clamp(0, 255) as u8,
                        alpha
                    ]));
                }
            }
            DynamicImage::ImageRgba8(result)
        },
        Filter::Invert => {
            let mut out = img.to_rgba8(); // Adicionado 'mut' aqui
            for px in out.pixels_mut() {  // Trocado para pixels_mut() e removido o 'mut' do px
                px[0] = 255 - px[0]; 
                px[1] = 255 - px[1]; 
                px[2] = 255 - px[2];
            }
            DynamicImage::ImageRgba8(out)
        }
    }
}