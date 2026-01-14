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
    let price_factor = 100_000_000u64;
    let amount_value = extract_value(body, "\"amount\":")?;
    let currency_id_value = extract_value(body, "\"currency_id\":")?;
    let created_value = extract_value(body, "\"created\":\"")?;

    let adjusted_price = if amount_value.starts_with("0.") {
        u64::from_str(&amount_value[2..]).map_err(|_| ParsingError::InvalidNumberFormat)?
    } else {
        let num = amount_value.split(".").collect::<Vec<&str>>();
        let fractional_part = if num.len() > 1 { num[1] } else { "0" };
        let fractional_length = fractional_part.len() as u32;
        let price =
            u64::from_str(num.join("").as_str()).map_err(|_| ParsingError::InvalidNumberFormat)?;
        price * price_factor / 10u64.pow(fractional_length)
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
        assert_eq!(parsed.price, 490096);

        let json_str = r#"{"currency_id":12,"amount":2.00490096,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.currency_id, "12");
        assert_eq!(parsed.price, 200490096);

        let json_str = r#"{"currency_id":1,"amount":0.10490096,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 10490096);

        let json_str = r#"{"currency_id":1,"amount":0.00000096,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 96);

        let json_str = r#"{"currency_id":1,"amount":240.23,"created":"2026-01-08T11:19:00Z"}"#;
        let parsed = parse_decoded_body(json_str).unwrap();
        assert_eq!(parsed.price, 24023000000);
    }
}
