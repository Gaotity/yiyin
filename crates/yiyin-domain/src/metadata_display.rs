const ROMAN_NUMERALS: &[(u32, &str)] = &[
    (1_000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

#[must_use]
pub fn format_shutter(numerator: u32, denominator: u32) -> String {
    if numerator == 0 || denominator == 0 {
        return String::new();
    }
    let seconds = f64::from(numerator) / f64::from(denominator);
    if seconds < 1.0 {
        let reciprocal = 1.0 / seconds;
        if reciprocal < 1.5 {
            decimal(seconds)
        } else {
            format!("1/{:.0}", reciprocal.round())
        }
    } else {
        decimal(seconds)
    }
}

#[must_use]
pub fn normalize_nikon_model(make: &str, model: &str) -> String {
    let make = make.to_uppercase();
    let without_make = model.replace(&make, "");
    let with_logo = without_make
        .chars()
        .map(|character| match character {
            'z' | 'Z' => 'ℤ',
            other => other,
        })
        .collect::<String>();
    let mut parts = with_logo.split('_').collect::<Vec<_>>();
    if parts.len() <= 1 {
        return with_logo;
    }
    let suffix = parts.pop().unwrap_or_default();
    let suffix = suffix
        .parse::<u32>()
        .ok()
        .and_then(to_roman)
        .unwrap_or_else(|| suffix.to_owned());
    format!("{} {suffix}", parts.join(" "))
}

#[must_use]
pub fn normalize_sony_model(model: &str) -> String {
    model.replacen("ILCE-", "α", 1).to_lowercase()
}

#[must_use]
pub fn normalize_make(make: &str) -> String {
    let make = make.replace("CORPORATION", "");
    let make = make.trim();
    let mut characters = make.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    format!("{first}{}", characters.as_str().to_lowercase())
}

#[must_use]
pub fn normalize_model_for_templates(make: &str, model: &str) -> String {
    let lookup_make = make.replace("CORPORATION", "").trim().to_owned();
    match lookup_make.as_str() {
        "NIKON" => normalize_nikon_model(&lookup_make, model),
        "SONY" => normalize_sony_model(model),
        _ => model.to_lowercase(),
    }
}

#[must_use]
pub fn normalize_date_time(value: &str) -> String {
    let Some((date, time)) = value.trim().split_once(' ') else {
        return String::new();
    };
    let time = time.split(['+', '-', '.']).next().unwrap_or_default();
    let date_parts = date.split(':').collect::<Vec<_>>();
    let time_parts = time.split(':').collect::<Vec<_>>();
    if date_parts.len() != 3
        || time_parts.len() != 3
        || date_parts
            .iter()
            .chain(&time_parts)
            .any(|part| part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return String::new();
    }
    format!(
        "{}/{}/{} {}:{}:{}",
        date_parts[0], date_parts[1], date_parts[2], time_parts[0], time_parts[1], time_parts[2]
    )
}

#[must_use]
pub fn normalize_white_balance(value: u32) -> &'static str {
    match value {
        0 => "Auto",
        1 => "手动",
        _ => "",
    }
}

#[must_use]
pub fn normalize_exposure_program(value: u32) -> &'static str {
    match value {
        0 => "Auto",
        1 => "M",
        2 => "P",
        3 => "A",
        4 => "S",
        _ => "",
    }
}

#[must_use]
pub fn normalize_metering_mode(value: u32) -> &'static str {
    match value {
        1 => "平均测光",
        2 => "中央重点测光",
        3 => "点测光",
        5 => "评价测光",
        6 => "局部测光",
        _ => "",
    }
}

#[must_use]
pub fn rounded_display(value: f64) -> String {
    format!("{:.0}", value.round())
}

#[must_use]
pub fn signed_decimal(value: f64) -> String {
    let value = decimal(value);
    if value.is_empty() || value == "0" || value.starts_with('-') {
        value
    } else {
        format!("+{value}")
    }
}

#[must_use]
pub fn decimal(value: f64) -> String {
    if !value.is_finite() {
        return String::new();
    }
    let mut result = format!("{value:.6}");
    while result.contains('.') && result.ends_with('0') {
        result.pop();
    }
    if result.ends_with('.') {
        result.pop();
    }
    result
}

fn to_roman(value: u32) -> Option<String> {
    if value == 0 || value > 3_999 {
        return None;
    }
    if let Some(numeral) = ["Ⅰ", "Ⅱ", "Ⅲ", "Ⅳ", "Ⅴ", "Ⅵ", "Ⅶ", "Ⅷ", "Ⅸ", "Ⅹ", "Ⅺ", "Ⅻ"]
        .get(usize::try_from(value - 1).ok()?)
    {
        return Some((*numeral).to_owned());
    }
    let mut remaining = value;
    let mut result = String::new();
    for &(number, numeral) in ROMAN_NUMERALS {
        while remaining >= number {
            result.push_str(numeral);
            remaining -= number;
        }
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::{format_shutter, normalize_nikon_model, normalize_sony_model};

    #[test]
    fn vendor_model_formatting_matches_the_legacy_template_formatter() {
        assert_eq!(normalize_nikon_model("NIKON", "NIKON Z 7_2"), " ℤ 7 Ⅱ");
        assert_eq!(normalize_sony_model("ILCE-7RM5"), "α7rm5");
    }

    #[test]
    fn shutter_formatting_preserves_fractional_and_whole_seconds() {
        assert_eq!(format_shutter(1, 125), "1/125");
        assert_eq!(format_shutter(2, 1), "2");
        assert_eq!(format_shutter(3, 2), "1.5");
        assert_eq!(format_shutter(0, 1), "");
        assert_eq!(format_shutter(1, 0), "");
    }

    #[test]
    fn sub_second_shutter_above_two_thirds_displays_decimal_seconds() {
        assert_eq!(format_shutter(7, 10), "0.7");
        assert_eq!(format_shutter(4, 5), "0.8");
        assert_eq!(format_shutter(3, 4), "0.75");
        // At and below the two-thirds boundary the legacy reciprocal stays.
        assert_eq!(format_shutter(2, 3), "1/2");
        assert_eq!(format_shutter(5, 8), "1/2");
        // One second and above keep the seconds display.
        assert_eq!(format_shutter(1, 1), "1");
    }
}
