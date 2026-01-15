#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, str::FromStr, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::str::FromStr;

use crate::Payload;

pub struct ParsedData {
    pub price: u64,
    pub currency_id: String,
    pub created: String,
}

impl ParsedData {
    pub fn timestamp(&self) -> Result<u64, ParsingError> {
        time::OffsetDateTime::parse(
            &self.created,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| ParsingError::InvalidTimestamp)
        .map(|t| t.unix_timestamp() as u64)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsingError {
    FieldExtractionError,
    InvalidNumberFormat,
    InvalidTimestamp,
    DeserializationError,
}

pub fn parse_data(data: &[u8]) -> Result<ParsedData, ParsingError> {
    serde_json_wasm::from_slice::<Payload>(data).unwrap();
    let payload = serde_json_wasm::from_slice::<Payload>(data)
        .map_err(|_| ParsingError::DeserializationError)?;
    let decoded_body = payload
        .decoded_body()
        .ok_or(ParsingError::DeserializationError)?;
    parse_decoded_body(&decoded_body)
}

fn parse_decoded_body(body: &str) -> Result<ParsedData, ParsingError> {
    let amount_value = extract_value(body, "\"amount\":")?;
    let currency_id_value = extract_value(body, "\"currency_id\":")?;
    let created_value = extract_value(body, "\"created\":\"")?;

    // Multiply by 100,000 and take integer part (no floats)
    // This is equivalent to moving the decimal point 5 places to the right
    let parts = amount_value.split('.').collect::<Vec<&str>>();
    let integer_part = u64::from_str(parts[0]).map_err(|_| ParsingError::InvalidNumberFormat)?;

    let adjusted_price = if parts.len() > 1 {
        let fractional_part = parts[1];

        // To multiply by 100,000, we take the integer part * 100,000
        // plus the first 5 digits of the fractional part (treating them as an integer)
        let frac_len = fractional_part.len();

        if frac_len >= 5 {
            // Take first 5 digits
            let frac_value = u64::from_str(&fractional_part[..5])
                .map_err(|_| ParsingError::InvalidNumberFormat)?;
            integer_part * 100_000 + frac_value
        } else {
            // Pad with zeros on the right
            let frac_value =
                u64::from_str(fractional_part).map_err(|_| ParsingError::InvalidNumberFormat)?;
            integer_part * 100_000 + frac_value * 10u64.pow(5 - frac_len as u32)
        }
    } else {
        // No fractional part
        integer_part * 100_000
    };

    Ok(ParsedData {
        price: adjusted_price,
        currency_id: currency_id_value.to_owned(),
        created: created_value.to_owned(),
    })
}

fn extract_value<'a>(data: &'a str, key: &'static str) -> Result<&'a str, ParsingError> {
    let parts: Vec<&str> = data.split(key).collect();
    if parts.len() != 2 {
        return Err(ParsingError::FieldExtractionError);
    }
    let tail = parts[1];

    let end_index = tail.find(|c| c == ',' || c == '}' || c == '"');
    match end_index {
        None => Err(ParsingError::FieldExtractionError),
        Some(index) => Ok(&tail[..index]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data() {
        let json_str = r#"{"currency_id":1,"amount":0.00490096,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.currency_id, "1");
        assert_eq!(parsed.created, "2026-01-08T11:19:00Z");
        assert_eq!(parsed.price, 490);

        let json_str = r#"{"currency_id":12,"amount":2.00490096,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.currency_id, "12");
        assert_eq!(parsed.price, 200_490);

        let json_str = r#"{"currency_id":1,"amount":0.10490096,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 10_490);

        let json_str = r#"{"currency_id":1,"amount":0.00000096,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 0);

        let json_str = r#"{"currency_id":1,"amount":240.23,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 24_023_000);
    }

    #[test]
    fn test_parse_edge_cases() {
        // Integer only (no decimal part)
        let json_str = r#"{"currency_id":1,"amount":42,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 4_200_000);

        // Zero
        let json_str = r#"{"currency_id":1,"amount":0,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 0);

        // Zero with decimal
        let json_str = r#"{"currency_id":1,"amount":0.0,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 0);

        // Exactly 5 decimal places
        let json_str = r#"{"currency_id":1,"amount":1.23456,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 123_456);

        // More than 5 decimal places (should truncate to 5)
        let json_str = r#"{"currency_id":1,"amount":1.2345678,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 123_456);

        // Single decimal place
        let json_str = r#"{"currency_id":1,"amount":99.9,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 9_990_000);

        // Two decimal places
        let json_str = r#"{"currency_id":1,"amount":50.25,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 5_025_000);

        // Three decimal places
        let json_str = r#"{"currency_id":1,"amount":7.125,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 712_500);

        // Four decimal places
        let json_str = r#"{"currency_id":1,"amount":3.1415,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 314_150);

        // Very small number (less than 0.00001)
        let json_str = r#"{"currency_id":1,"amount":0.000001,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 0);

        // Small fraction with trailing zeros
        let json_str = r#"{"currency_id":1,"amount":0.10000,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 10_000);

        // Large integer part
        let json_str = r#"{"currency_id":1,"amount":12345.6789,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 1_234_567_890);

        // Very large number
        let json_str =
            r#"{"currency_id":1,"amount":999999.99999,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 99_999_999_999);

        // Just above zero
        let json_str = r#"{"currency_id":1,"amount":0.00001,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 1);

        // All 5s in fractional part
        let json_str = r#"{"currency_id":1,"amount":0.55555,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 55_555);

        // All 9s in fractional part
        let json_str = r#"{"currency_id":1,"amount":0.99999,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 99_999);

        // Single digit with fractional part
        let json_str = r#"{"currency_id":1,"amount":1.00001,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 100_001);

        // Zero integer with significant fractional part
        let json_str = r#"{"currency_id":1,"amount":0.99,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 99_000);
    }
}
