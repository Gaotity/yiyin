use image::{Rgba, RgbaImage, imageops::FilterType};
use yiyin_domain::{RenderOptions, RenderPlan};

pub fn render_background(
    source: &RgbaImage,
    plan: &RenderPlan,
    options: &RenderOptions,
) -> RgbaImage {
    if options.solid_background {
        return RgbaImage::from_pixel(
            plan.canvas.width,
            plan.canvas.height,
            parse_color(options.solid_color.as_str()),
        );
    }

    let square = image::imageops::resize(source, 128, 128, FilterType::Triangle);
    let configured = options.background_blur.get();
    let compatibility_value = if configured == 0 { 100 } else { configured };
    let sigma = 7.0 * (f32::from(compatibility_value) / 100.0);
    let blurred = image::imageops::blur(&square, sigma.max(0.1));
    let mut background = image::imageops::resize(
        &blurred,
        plan.canvas.width,
        plan.canvas.height,
        FilterType::Triangle,
    );
    apply_brightness_overlay(&mut background);
    background
}

fn apply_brightness_overlay(image: &mut RgbaImage) {
    let total = image.pixels().fold(0_u64, |sum, pixel| {
        sum + (u64::from(pixel[0]) + u64::from(pixel[1]) + u64::from(pixel[2])) / 3
    });
    let pixel_count = u64::from(image.width()) * u64::from(image.height());
    let average = total.checked_div(pixel_count).unwrap_or(0);
    let overlay = if average < 15 {
        180
    } else if average < 20 {
        158
    } else if average < 40 {
        128
    } else {
        0
    };
    for pixel in image.pixels_mut() {
        for channel in &mut pixel.0[..3] {
            *channel = blend_channel(*channel, overlay, 51);
        }
    }
}

fn blend_channel(bottom: u8, top: u8, alpha: u8) -> u8 {
    let inverse = u16::from(255 - alpha);
    let value = u16::from(bottom) * inverse + u16::from(top) * u16::from(alpha);
    u8::try_from((value + 127) / 255).unwrap_or(u8::MAX)
}

fn parse_color(value: &str) -> Rgba<u8> {
    let digits = value.strip_prefix('#').unwrap_or(value);
    let expanded;
    let digits = if digits.len() == 3 {
        expanded = digits
            .chars()
            .flat_map(|character| [character, character])
            .collect::<String>();
        expanded.as_str()
    } else {
        digits
    };
    if digits.len() != 6 {
        return Rgba([255, 255, 255, 255]);
    }
    let channel = |offset| u8::from_str_radix(&digits[offset..offset + 2], 16).unwrap_or(255);
    Rgba([channel(0), channel(2), channel(4), 255])
}
