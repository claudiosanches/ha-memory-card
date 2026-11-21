//! Prices are stored as whole minor units (cents) with an ISO 4217 currency
//! code, and travel as decimal text so clients need no exponent table.

/// Largest price accepted, in minor units.
const MAX_MINOR: i64 = 1_000_000_000_000;

/// Currencies whose minor unit is not a hundredth (ISO 4217).
const EXPONENTS: [(&str, u32); 24] = [
    ("BIF", 0),
    ("CLP", 0),
    ("DJF", 0),
    ("GNF", 0),
    ("ISK", 0),
    ("JPY", 0),
    ("KMF", 0),
    ("KRW", 0),
    ("PYG", 0),
    ("RWF", 0),
    ("UGX", 0),
    ("UYI", 0),
    ("VND", 0),
    ("VUV", 0),
    ("XAF", 0),
    ("XOF", 0),
    ("XPF", 0),
    ("BHD", 3),
    ("IQD", 3),
    ("JOD", 3),
    ("KWD", 3),
    ("LYD", 3),
    ("OMR", 3),
    ("TND", 3),
];

/// An uppercased three-letter code, or `None` when `value` is not one.
pub fn currency(value: &str) -> Option<String> {
    let value = value.trim();
    (value.len() == 3 && value.bytes().all(|b| b.is_ascii_alphabetic()))
        .then(|| value.to_ascii_uppercase())
}

pub fn exponent(currency: &str) -> u32 {
    EXPONENTS
        .iter()
        .find(|(code, _)| *code == currency)
        .map_or(2, |(_, exponent)| *exponent)
}

/// Reads an amount such as `59.99`, `59,99`, `R$ 1.299,90` or `1,299.90`.
/// The last `.` or `,` is the decimal mark when the currency's minor unit
/// fits the digits after it; other marks and symbols are ignored.
pub fn parse(text: &str, currency: &str) -> Option<i64> {
    let kept: String = text
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .collect();
    if !kept.bytes().any(|b| b.is_ascii_digit()) || text.contains('-') {
        return None;
    }
    let exponent = exponent(currency);
    let (whole, fraction) = match kept.rfind(['.', ',']) {
        Some(at) => {
            let after = &kept[at + 1..];
            if !after.is_empty() && after.len() <= exponent as usize {
                (&kept[..at], after)
            } else if after.len() == 3 {
                (kept.as_str(), "")
            } else {
                return None;
            }
        }
        None => (kept.as_str(), ""),
    };
    let whole: String = whole.chars().filter(char::is_ascii_digit).collect();
    let whole: i64 = if whole.is_empty() {
        0
    } else {
        whole.parse().ok()?
    };
    let scale = 10i64.pow(exponent);
    let fraction: i64 = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<i64>().ok()? * 10i64.pow(exponent - fraction.len() as u32)
    };
    let minor = whole.checked_mul(scale)?.checked_add(fraction)?;
    (minor <= MAX_MINOR).then_some(minor)
}

/// Minor units as plain decimal text, such as `59.99` or `5980`.
pub fn format(minor: i64, currency: &str) -> String {
    let exponent = exponent(currency);
    if exponent == 0 {
        return minor.to_string();
    }
    let scale = 10i64.pow(exponent);
    format!(
        "{}.{:0width$}",
        minor / scale,
        minor % scale,
        width = exponent as usize
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_common_spellings() {
        assert_eq!(parse("59.99", "USD"), Some(5999));
        assert_eq!(parse("59,9", "EUR"), Some(5990));
        assert_eq!(parse("R$ 1.299,90", "BRL"), Some(129_990));
        assert_eq!(parse("1,299.90", "USD"), Some(129_990));
        assert_eq!(parse("1,299", "USD"), Some(129_900));
        assert_eq!(parse("60", "USD"), Some(6000));
        assert_eq!(parse("¥5,980", "JPY"), Some(5980));
        assert_eq!(parse("0", "USD"), Some(0));
        assert_eq!(parse("1.250", "KWD"), Some(1250));
    }

    #[test]
    fn rejects_what_is_not_an_amount() {
        assert_eq!(parse("", "USD"), None);
        assert_eq!(parse("free", "USD"), None);
        assert_eq!(parse("-5", "USD"), None);
        assert_eq!(parse("5.999", "USD"), Some(599_900), "a thousands mark");
        assert_eq!(parse("5.9999", "USD"), None);
        assert_eq!(parse("5.5", "JPY"), None);
        assert_eq!(parse("99999999999999", "USD"), None);
    }

    #[test]
    fn formats_minor_units() {
        assert_eq!(format(5999, "USD"), "59.99");
        assert_eq!(format(5, "EUR"), "0.05");
        assert_eq!(format(5980, "JPY"), "5980");
        assert_eq!(format(1250, "KWD"), "1.250");
    }

    #[test]
    fn currency_codes() {
        assert_eq!(currency(" brl "), Some("BRL".into()));
        assert_eq!(currency("R$"), None);
        assert_eq!(currency("EURO"), None);
    }
}
