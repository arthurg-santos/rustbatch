use ab_glyph::{FontArc, PxScale, ScaleFont, Font}; // <-- Adicionado 'Font' aqui
use image::{DynamicImage, Rgba, RgbaImage, imageops};
use std::path::Path;
use crate::error::CoreError;
use crate::model::{Watermark, WatermarkPosition};

pub const DEFAULT_FONT: &[u8] = include_bytes!("../../assets/DejaVuSans.ttf");

pub fn apply_watermark(
    img: DynamicImage,
    wm: &Watermark,
) -> Result<DynamicImage, CoreError> {
    match wm {
        Watermark::Text { .. } => apply_text_watermark(img, wm),
        Watermark::Image { .. } => apply_image_watermark(img, wm),
    }
}

fn apply_text_watermark(
    img: DynamicImage,
    wm: &Watermark,
) -> Result<DynamicImage, CoreError> {
    let Watermark::Text {
        text,
        font_size,
        color,
        opacity,
        margin,
        position,
    } = wm
    else {
        unreachable!()
    };

    if text.is_empty() {
        return Ok(img);
    }

    let font = FontArc::try_from_slice(DEFAULT_FONT)
        .map_err(|e| CoreError::InvalidWatermark(format!("fonte inválida: {e}")))?;

    let scale = PxScale::from(*font_size);
    let scaled = font.as_scaled(scale);

    // Medir o texto
    let mut width = 0.0f32;
    let mut prev: Option<ab_glyph::GlyphId> = None;
    for c in text.chars() {
        let glyph_id = scaled.glyph_id(c);
        if let Some(p) = prev {
            width += scaled.kern(p, glyph_id);
        }
        width += scaled.h_advance(glyph_id);
        prev = Some(glyph_id);
    }
    let height = scaled.ascent() - scaled.descent();

    let pad = 2u32;
    let canvas_w = width.ceil() as u32 + pad * 2;
    let canvas_h = height.ceil() as u32 + pad * 2;
    let mut canvas = RgbaImage::new(canvas_w, canvas_h);

    // Rasterizar glifos
    let mut caret_x = pad as f32;
    let baseline_y = pad as f32 + scaled.ascent();
    prev = None;

    for c in text.chars() {
        let glyph_id = scaled.glyph_id(c);
        if let Some(p) = prev {
            caret_x += scaled.kern(p, glyph_id);
        }

        let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(caret_x, baseline_y));

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, coverage| {
                let px = bounds.min.x as u32 + gx;
                let py = bounds.min.y as u32 + gy;
                if px < canvas_w && py < canvas_h {
                    let alpha = (coverage * 255.0 * *opacity) as u8;
                    let existing = canvas.get_pixel(px, py);
                    let blended = blend_pixel(*existing, Rgba([color[0], color[1], color[2], alpha]));
                    canvas.put_pixel(px, py, blended);
                }
            });
        }

        caret_x += scaled.h_advance(glyph_id);
        prev = Some(glyph_id);
    }

    let (ox, oy) = anchor_position(
        img.width(),
        img.height(),
        canvas_w,
        canvas_h,
        *margin,
        *position,
    );

    let mut base = img.to_rgba8();
    // Correção: ox e oy precisam ser i64
    imageops::overlay(&mut base, &canvas, ox as i64, oy as i64);
    Ok(DynamicImage::ImageRgba8(base))
}

fn apply_image_watermark(
    img: DynamicImage,
    wm: &Watermark,
) -> Result<DynamicImage, CoreError> {
    let Watermark::Image {
        path,
        scale,
        opacity,
        margin,
        position,
    } = wm
    else {
        unreachable!()
    };

    let logo = image::open(Path::new(path))
        .map_err(|e| CoreError::InvalidWatermark(format!("marca d'água: {e}")))?;

    let logo_w = (img.width() as f32 * *scale).round() as u32;
    let logo_h = (logo.height() as f32 * (*scale * img.width() as f32 / logo.width() as f32))
        .round() as u32;
    let mut logo_rgba = logo.resize_exact(logo_w, logo_h, imageops::FilterType::Lanczos3).to_rgba8();

    // Aplicar opacidade multiplicando o canal alfa
    if *opacity < 1.0 {
        for px in logo_rgba.pixels_mut() {
            px[3] = (px[3] as f32 * *opacity).round() as u8;
        }
    }

    let (ox, oy) = anchor_position(
        img.width(),
        img.height(),
        logo_w,
        logo_h,
        *margin,
        *position,
    );

    let mut base = img.to_rgba8();
    // Correção: ox e oy precisam ser i64
    imageops::overlay(&mut base, &logo_rgba, ox as i64, oy as i64);
    Ok(DynamicImage::ImageRgba8(base))
}

fn anchor_position(
    base_w: u32,
    base_h: u32,
    wm_w: u32,
    wm_h: u32,
    margin: u32,
    pos: WatermarkPosition,
) -> (u32, u32) {
    let left = margin;
    let center_x = (base_w.saturating_sub(wm_w)) / 2;
    let right = base_w.saturating_sub(wm_w).saturating_sub(margin);
    let top = margin;
    let center_y = (base_h.saturating_sub(wm_h)) / 2;
    let bottom = base_h.saturating_sub(wm_h).saturating_sub(margin);

    match pos {
        WatermarkPosition::TopLeft => (left, top),
        WatermarkPosition::TopCenter => (center_x, top),
        WatermarkPosition::TopRight => (right, top),
        WatermarkPosition::MiddleLeft => (left, center_y),
        WatermarkPosition::Center => (center_x, center_y),
        WatermarkPosition::MiddleRight => (right, center_y),
        WatermarkPosition::BottomLeft => (left, bottom),
        WatermarkPosition::BottomCenter => (center_x, bottom),
        WatermarkPosition::BottomRight => (right, bottom),
    }
}

fn blend_pixel(bottom: Rgba<u8>, top: Rgba<u8>) -> Rgba<u8> {
    let a = top[3] as f32 / 255.0;
    let inv = 1.0 - a;
    Rgba([
        (top[0] as f32 * a + bottom[0] as f32 * inv) as u8,
        (top[1] as f32 * a + bottom[1] as f32 * inv) as u8,
        (top[2] as f32 * a + bottom[2] as f32 * inv) as u8,
        (top[3] as f32 + bottom[3] as f32 * inv) as u8,
    ])
}