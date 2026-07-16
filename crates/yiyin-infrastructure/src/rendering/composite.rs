use image::{GrayImage, Luma, Rgba, RgbaImage, imageops::FilterType};
use yiyin_domain::{Rect, RenderPlan};

pub fn composite_main(canvas: &mut RgbaImage, main: &RgbaImage, plan: &RenderPlan) {
    if plan.shadow_blur > 0.0 {
        composite_shadow(canvas, plan);
    }
    composite_rounded(canvas, main, plan.main_rect, plan.corner_radius);
}

pub fn overlay_rgba(canvas: &mut RgbaImage, overlay: &RgbaImage, x: i64, y: i64) {
    for (overlay_x, overlay_y, top) in overlay.enumerate_pixels() {
        let target_x = x + i64::from(overlay_x);
        let target_y = y + i64::from(overlay_y);
        let (Ok(target_x), Ok(target_y)) = (u32::try_from(target_x), u32::try_from(target_y))
        else {
            continue;
        };
        if target_x >= canvas.width() || target_y >= canvas.height() {
            continue;
        }
        blend_pixel(canvas.get_pixel_mut(target_x, target_y), *top);
    }
}

fn composite_shadow(canvas: &mut RgbaImage, plan: &RenderPlan) {
    let scale = (512.0 / f64::from(plan.canvas.width.max(plan.canvas.height))).min(1.0);
    let width = scaled_dimension(plan.canvas.width, scale);
    let height = scaled_dimension(plan.canvas.height, scale);
    let mut mask = GrayImage::from_pixel(width, height, Luma([0]));
    let rect = Rect {
        x: scaled_coordinate(plan.main_rect.x, scale),
        y: scaled_coordinate(plan.main_rect.y, scale),
        width: scaled_dimension(plan.main_rect.width, scale),
        height: scaled_dimension(plan.main_rect.height, scale),
    };
    let radius = plan.corner_radius * scale;
    for y in rect.y..rect.y.saturating_add(rect.height).min(height) {
        for x in rect.x..rect.x.saturating_add(rect.width).min(width) {
            if inside_rounded_rect(x - rect.x, y - rect.y, rect.width, rect.height, radius) {
                mask.put_pixel(x, y, Luma([255]));
            }
        }
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the bounded shadow sigma is intentionally converted for image::blur"
    )]
    let sigma = (plan.shadow_blur * scale).max(0.1) as f32;
    let blurred = image::imageops::blur(&mask, sigma);
    let full = image::imageops::resize(
        &blurred,
        plan.canvas.width,
        plan.canvas.height,
        FilterType::Triangle,
    );
    for (x, y, alpha) in full.enumerate_pixels() {
        let alpha = u8::try_from((u16::from(alpha[0]) * 166 + 127) / 255).unwrap_or(u8::MAX);
        if alpha != 0 {
            blend_pixel(canvas.get_pixel_mut(x, y), Rgba([0, 0, 0, alpha]));
        }
    }
}

fn composite_rounded(canvas: &mut RgbaImage, main: &RgbaImage, rect: Rect, radius: f64) {
    let resized;
    let main = if main.width() == rect.width && main.height() == rect.height {
        main
    } else {
        resized = image::imageops::resize(main, rect.width, rect.height, FilterType::Lanczos3);
        &resized
    };
    for (x, y, pixel) in main.enumerate_pixels() {
        if inside_rounded_rect(x, y, rect.width, rect.height, radius) {
            let target_x = rect.x + x;
            let target_y = rect.y + y;
            if target_x < canvas.width() && target_y < canvas.height() {
                blend_pixel(canvas.get_pixel_mut(target_x, target_y), *pixel);
            }
        }
    }
}

fn inside_rounded_rect(x: u32, y: u32, width: u32, height: u32, radius: f64) -> bool {
    if radius <= 0.0 {
        return true;
    }
    let radius = radius.min(f64::from(width.min(height)) / 2.0);
    let x = f64::from(x) + 0.5;
    let y = f64::from(y) + 0.5;
    let width = f64::from(width);
    let height = f64::from(height);
    let center_x = if x < radius {
        radius
    } else if x > width - radius {
        width - radius
    } else {
        x
    };
    let center_y = if y < radius {
        radius
    } else if y > height - radius {
        height - radius
    } else {
        y
    };
    let dx = x - center_x;
    let dy = y - center_y;
    dx.mul_add(dx, dy * dy) <= radius * radius
}

fn blend_pixel(bottom: &mut Rgba<u8>, top: Rgba<u8>) {
    let alpha = u16::from(top[3]);
    if alpha == 0 {
        return;
    }
    let inverse = 255 - alpha;
    for channel in 0..3 {
        let value = u16::from(top[channel]) * alpha + u16::from(bottom[channel]) * inverse;
        bottom[channel] = u8::try_from((value + 127) / 255).unwrap_or(u8::MAX);
    }
    bottom[3] = 255;
}

fn scaled_coordinate(value: u32, scale: f64) -> u32 {
    u32::try_from(scaled(value, scale).min(u64::from(u32::MAX))).unwrap_or(u32::MAX)
}

fn scaled_dimension(value: u32, scale: f64) -> u32 {
    u32::try_from(scaled(value, scale).clamp(1, u64::from(u32::MAX))).unwrap_or(u32::MAX)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "positive bounded image coordinates are rounded before conversion"
)]
fn scaled(value: u32, scale: f64) -> u64 {
    (f64::from(value) * scale).round() as u64
}
