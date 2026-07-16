use std::{fs, path::Path};

use image::{Rgb, RgbImage};

#[derive(Clone, Copy, Debug)]
pub struct PerceptualMetrics {
    pub ssim: f64,
    pub changed_pixel_ratio: f64,
}

#[allow(
    clippy::cast_precision_loss,
    reason = "fixture pixel counts are far below the exact integer range of f64"
)]
pub fn compare(actual: &RgbImage, expected: &RgbImage) -> PerceptualMetrics {
    assert_eq!(actual.dimensions(), expected.dimensions());
    let count = f64::from(actual.width()) * f64::from(actual.height());
    let mut changed = 0_u64;
    for (actual, expected) in actual.pixels().zip(expected.pixels()) {
        if actual
            .0
            .iter()
            .zip(expected.0)
            .any(|(actual, expected)| actual.abs_diff(expected) > 24)
        {
            changed += 1;
        }
    }
    let ssim = windowed_ssim(actual, expected, 8);
    PerceptualMetrics {
        ssim,
        changed_pixel_ratio: changed as f64 / count,
    }
}

#[allow(
    clippy::cast_precision_loss,
    reason = "fixture window sample counts are far below the exact integer range of f64"
)]
fn windowed_ssim(actual: &RgbImage, expected: &RgbImage, window: u32) -> f64 {
    let mut weighted_sum = 0.0;
    let mut total_weight = 0_u64;
    let c1 = (0.01_f64 * 255.0).powi(2);
    let c2 = (0.03_f64 * 255.0).powi(2);
    for top in (0..actual.height()).step_by(window as usize) {
        for left in (0..actual.width()).step_by(window as usize) {
            let right = left.saturating_add(window).min(actual.width());
            let bottom = top.saturating_add(window).min(actual.height());
            let samples = u64::from(right - left) * u64::from(bottom - top);
            let samples_f64 = samples as f64;
            let mut sum_actual = 0.0;
            let mut sum_expected = 0.0;
            for y in top..bottom {
                for x in left..right {
                    sum_actual += luminance(*actual.get_pixel(x, y));
                    sum_expected += luminance(*expected.get_pixel(x, y));
                }
            }
            let mean_actual = sum_actual / samples_f64;
            let mean_expected = sum_expected / samples_f64;
            let mut variance_actual = 0.0;
            let mut variance_expected = 0.0;
            let mut covariance = 0.0;
            for y in top..bottom {
                for x in left..right {
                    let actual_delta = luminance(*actual.get_pixel(x, y)) - mean_actual;
                    let expected_delta = luminance(*expected.get_pixel(x, y)) - mean_expected;
                    variance_actual += actual_delta * actual_delta;
                    variance_expected += expected_delta * expected_delta;
                    covariance += actual_delta * expected_delta;
                }
            }
            let divisor = (samples_f64 - 1.0).max(1.0);
            variance_actual /= divisor;
            variance_expected /= divisor;
            covariance /= divisor;
            let score = ((2.0 * mean_actual * mean_expected + c1) * (2.0 * covariance + c2))
                / ((mean_actual.powi(2) + mean_expected.powi(2) + c1)
                    * (variance_actual + variance_expected + c2));
            weighted_sum += score * samples_f64;
            total_weight += samples;
        }
    }
    weighted_sum / total_weight as f64
}

pub fn write_failure_artifacts(
    directory: &Path,
    actual: &RgbImage,
    expected: &RgbImage,
) -> image::ImageResult<()> {
    fs::create_dir_all(directory).map_err(image::ImageError::IoError)?;
    actual.save(directory.join("actual.png"))?;
    expected.save(directory.join("expected.png"))?;
    let mut diff = RgbImage::new(actual.width(), actual.height());
    for ((target, actual), expected) in diff
        .pixels_mut()
        .zip(actual.pixels())
        .zip(expected.pixels())
    {
        *target = Rgb([
            actual[0].abs_diff(expected[0]).saturating_mul(4),
            actual[1].abs_diff(expected[1]).saturating_mul(4),
            actual[2].abs_diff(expected[2]).saturating_mul(4),
        ]);
    }
    diff.save(directory.join("diff.png"))
}

fn luminance(pixel: Rgb<u8>) -> f64 {
    0.2126_f64.mul_add(
        f64::from(pixel[0]),
        0.7152_f64.mul_add(f64::from(pixel[1]), 0.0722 * f64::from(pixel[2])),
    )
}
