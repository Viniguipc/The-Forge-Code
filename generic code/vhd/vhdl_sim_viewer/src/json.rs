//! Lightweight, 100% Safe Rust JSON parser and serializer for the IDE HTTP API.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            JsonValue::Number(n) if *n >= 0.0 => Some(*n as u64),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(a) => Some(a.as_slice()),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(o) => Some(o),
            _ => None,
        }
    }
}

pub fn escape_json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_ws();
        let b = self.peek().ok_or_else(|| "Unexpected end of JSON".to_string())?;
        match b {
            b'{' => self.parse_object(),
            b'[' => self.parse_array(),
            b'"' => Ok(JsonValue::String(self.parse_string()?)),
            b't' => {
                if self.bytes[self.pos..].starts_with(b"true") {
                    self.pos += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err("Invalid literal in JSON".to_string())
                }
            }
            b'f' => {
                if self.bytes[self.pos..].starts_with(b"false") {
                    self.pos += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err("Invalid literal in JSON".to_string())
                }
            }
            b'n' => {
                if self.bytes[self.pos..].starts_with(b"null") {
                    self.pos += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err("Invalid literal in JSON".to_string())
                }
            }
            b'-' | b'0'..=b'9' => self.parse_number(),
            other => Err(format!("Unexpected byte '{}' in JSON", other as char)),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.next(); // '{'
        let mut map = HashMap::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.next();
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err("Expected string key in JSON object".to_string());
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.next() != Some(b':') {
                return Err("Expected ':' in JSON object".to_string());
            }
            let val = self.parse_value()?;
            map.insert(key, val);
            self.skip_ws();
            match self.next() {
                Some(b',') => continue,
                Some(b'}') => break,
                _ => return Err("Expected ',' or '}' in JSON object".to_string()),
            }
        }
        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.next(); // '['
        let mut arr = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.next();
            return Ok(JsonValue::Array(arr));
        }

        loop {
            let val = self.parse_value()?;
            arr.push(val);
            self.skip_ws();
            match self.next() {
                Some(b',') => continue,
                Some(b']') => break,
                _ => return Err("Expected ',' or ']' in JSON array".to_string()),
            }
        }
        Ok(JsonValue::Array(arr))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.next(); // '"'
        let mut out = String::new();
        while self.pos < self.bytes.len() {
            // Read UTF-8 char safely using str slice
            let remaining = std::str::from_utf8(&self.bytes[self.pos..])
                .map_err(|_| "Invalid UTF-8 in JSON string".to_string())?;
            let ch = remaining
                .chars()
                .next()
                .ok_or_else(|| "Unterminated JSON string".to_string())?;
            self.pos += ch.len_utf8();

            if ch == '"' {
                return Ok(out);
            } else if ch == '\\' {
                let esc = self.next().ok_or_else(|| "Unterminated escape in JSON".to_string())?;
                match esc {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{0008}'),
                    b'f' => out.push('\u{000C}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => {
                        if self.pos + 4 > self.bytes.len() {
                            return Err("Invalid \\u escape in JSON".to_string());
                        }
                        let hex = std::str::from_utf8(&self.bytes[self.pos..self.pos + 4])
                            .map_err(|_| "Invalid \\u hex".to_string())?;
                        let code = u32::from_str_radix(hex, 16)
                            .map_err(|_| "Invalid \\u hex digits".to_string())?;
                        self.pos += 4;
                        if let Some(c) = char::from_u32(code) {
                            out.push(c);
                        }
                    }
                    other => out.push(other as char),
                }
            } else {
                out.push(ch);
            }
        }
        Err("Unterminated JSON string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_digit() || matches!(b, b'-' | b'+' | b'.' | b'e' | b'E') {
                self.pos += 1;
            } else {
                break;
            }
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| "Invalid number in JSON".to_string())?;
        let num: f64 = s.parse().map_err(|_| format!("Invalid JSON number: {s}"))?;
        Ok(JsonValue::Number(num))
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(input);
    parser.parse_value()
}
