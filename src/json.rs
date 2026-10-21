//! Parser dan penulis JSON mini — cukup untuk menyimpan progres belajar.
//!
//! Kenapa tidak memakai `serde_json`? Proyek ini sengaja tanpa dependensi
//! eksternal supaya bisa di-build di lingkungan luring. Format yang didukung:
//! object, array, string (dengan escape `\"`, `\\`, `\n`, `\t`, `\uXXXX`),
//! angka, `true`, `false`, dan `null`.
//!
//! ```
//! use korean_rust_grammar::json::Value;
//!
//! let v = Value::parse(r#"{"topics":{"hangeul":{"best":80}}}"#).unwrap();
//! assert_eq!(v.get("topics").and_then(|t| t.get("hangeul")).and_then(|h| h.get("best")).and_then(Value::as_i64), Some(80));
//! ```

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Nilai JSON.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

/// Kesalahan parsing beserta posisi byte-nya.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub offset: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (pada byte {})", self.message, self.offset)
    }
}

impl std::error::Error for ParseError {}

impl Value {
    /// Objek kosong.
    pub fn object() -> Value {
        Value::Object(BTreeMap::new())
    }

    /// Sisipkan pasangan kunci-nilai bila `self` adalah object.
    pub fn insert<K: Into<String>, V: Into<Value>>(&mut self, key: K, value: V) -> &mut Value {
        if let Value::Object(map) = self {
            map.insert(key.into(), value.into());
        }
        self
    }

    /// Ambil anggota object berdasarkan kunci.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(map) => map.get(key),
            _ => None,
        }
    }

    /// Ambil elemen array berdasarkan indeks.
    pub fn at(&self, index: usize) -> Option<&Value> {
        match self {
            Value::Array(items) => items.get(index),
            _ => None,
        }
    }

    /// Ambil anggota object, buat object kosong bila belum ada.
    pub fn entry(&mut self, key: &str) -> &mut Value {
        if !matches!(self, Value::Object(_)) {
            *self = Value::object();
        }
        if let Value::Object(map) = self {
            map.entry(key.to_string()).or_insert_with(Value::object)
        } else {
            unreachable!()
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(n) => Some(*n as i64),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        self.as_i64().and_then(|n| u64::try_from(n).ok())
    }

    pub fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|n| u32::try_from(n).ok())
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Iterasi object sebagai pasangan (kunci, nilai).
    pub fn entries(&self) -> impl Iterator<Item = (&String, &Value)> {
        let empty: &BTreeMap<String, Value> = &EMPTY_MAP;
        match self {
            Value::Object(map) => map.iter(),
            _ => empty.iter(),
        }
    }

    /// Panjang array atau jumlah anggota object.
    pub fn len(&self) -> usize {
        match self {
            Value::Array(items) => items.len(),
            Value::Object(map) => map.len(),
            Value::String(s) => s.chars().count(),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Serialisasi ke teks JSON (satu baris).
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        self.write_to(&mut out);
        out
    }

    /// Serialisasi dengan indentasi `spaces` spasi agar mudah dibaca manusia.
    pub fn to_json_pretty(&self, spaces: usize) -> String {
        let mut out = String::new();
        self.write_pretty(&mut out, 0, spaces);
        out.push('\n');
        out
    }

    fn write_to(&self, out: &mut String) {
        match self {
            Value::Null => out.push_str("null"),
            Value::Bool(true) => out.push_str("true"),
            Value::Bool(false) => out.push_str("false"),
            Value::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() && n.abs() < 9.0e15 {
                    let _ = write!(out, "{}", *n as i64);
                } else {
                    let _ = write!(out, "{}", n);
                }
            }
            Value::String(s) => write_escaped(out, s),
            Value::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.write_to(out);
                }
                out.push(']');
            }
            Value::Object(map) => {
                out.push('{');
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_escaped(out, k);
                    out.push(':');
                    v.write_to(out);
                }
                out.push('}');
            }
        }
    }

    fn write_pretty(&self, out: &mut String, depth: usize, spaces: usize) {
        let pad = |out: &mut String, level: usize| {
            out.push('\n');
            for _ in 0..level * spaces {
                out.push(' ');
            }
        };
        match self {
            Value::Array(items) if !items.is_empty() => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    pad(out, depth + 1);
                    item.write_pretty(out, depth + 1, spaces);
                }
                pad(out, depth);
                out.push(']');
            }
            Value::Object(map) if !map.is_empty() => {
                out.push('{');
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    pad(out, depth + 1);
                    write_escaped(out, k);
                    out.push_str(": ");
                    v.write_pretty(out, depth + 1, spaces);
                }
                pad(out, depth);
                out.push('}');
            }
            other => other.write_to(out),
        }
    }

    /// Parse teks JSON.
    pub fn parse(text: &str) -> Result<Value, ParseError> {
        let mut p = Parser {
            bytes: text.as_bytes(),
            pos: 0,
        };
        p.skip_ws();
        let value = p.value()?;
        p.skip_ws();
        if p.pos != p.bytes.len() {
            return Err(p.error("karakter berlebih setelah JSON"));
        }
        Ok(value)
    }
}

static EMPTY_MAP: BTreeMap<String, Value> = BTreeMap::new();

fn write_escaped(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            offset: self.pos,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek();
        if b.is_some() {
            self.pos += 1;
        }
        b
    }

    fn skip_ws(&mut self) {
        while matches!(
            self.peek(),
            Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')
        ) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), ParseError> {
        if self.peek() == Some(byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.error(&format!("diharapkan '{}'", byte as char)))
        }
    }

    fn value(&mut self) -> Result<Value, ParseError> {
        match self.peek() {
            None => Err(self.error("JSON tidak lengkap")),
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Value::String(self.string()?)),
            Some(b't') => self.literal("true", Value::Bool(true)),
            Some(b'f') => self.literal("false", Value::Bool(false)),
            Some(b'n') => self.literal("null", Value::Null),
            Some(_) => self.number(),
        }
    }

    fn literal(&mut self, text: &str, value: Value) -> Result<Value, ParseError> {
        let end = self.pos + text.len();
        if end <= self.bytes.len() && &self.bytes[self.pos..end] == text.as_bytes() {
            self.pos = end;
            Ok(value)
        } else {
            Err(self.error("literal tidak dikenal"))
        }
    }

    fn number(&mut self) -> Result<Value, ParseError> {
        let start = self.pos;
        if matches!(self.peek(), Some(b'-') | Some(b'+')) {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.peek() == Some(b'.') {
            self.pos += 1;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'-') | Some(b'+')) {
                self.pos += 1;
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if start == self.pos {
            return Err(self.error("angka tidak valid"));
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| self.error("UTF-8 tidak valid"))?;
        text.parse::<f64>()
            .map(Value::Number)
            .map_err(|_| self.error("angka tidak valid"))
    }

    fn string(&mut self) -> Result<String, ParseError> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return Err(self.error("string tidak ditutup")),
                Some(b'"') => return Ok(out),
                Some(b'\\') => match self.bump() {
                    Some(b'"') => out.push('"'),
                    Some(b'\\') => out.push('\\'),
                    Some(b'/') => out.push('/'),
                    Some(b'b') => out.push('\u{0008}'),
                    Some(b'f') => out.push('\u{000C}'),
                    Some(b'n') => out.push('\n'),
                    Some(b'r') => out.push('\r'),
                    Some(b't') => out.push('\t'),
                    Some(b'u') => {
                        let mut code = 0u32;
                        for _ in 0..4 {
                            let digit = self
                                .bump()
                                .and_then(|c| (c as char).to_digit(16))
                                .ok_or_else(|| self.error("escape \\u tidak valid"))?;
                            code = code * 16 + digit;
                        }
                        // Pasangan surrogate perlu digabung.
                        if (0xD800..0xDC00).contains(&code)
                            && self.peek() == Some(b'\\')
                            && self.bytes.get(self.pos + 1) == Some(&b'u')
                        {
                            self.pos += 2;
                            let mut low = 0u32;
                            for _ in 0..4 {
                                let digit = self
                                    .bump()
                                    .and_then(|c| (c as char).to_digit(16))
                                    .ok_or_else(|| self.error("surrogate tidak valid"))?;
                                low = low * 16 + digit;
                            }
                            if (0xDC00..0xE000).contains(&low) {
                                code = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                            }
                        }
                        out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                    }
                    _ => return Err(self.error("escape tidak dikenal")),
                },
                Some(b) => {
                    // Salin UTF-8 apa adanya.
                    let start = self.pos - 1;
                    let width = utf8_width(b);
                    let end = start + width;
                    if end > self.bytes.len() {
                        return Err(self.error("UTF-8 terpotong"));
                    }
                    match std::str::from_utf8(&self.bytes[start..end]) {
                        Ok(s) => out.push_str(s),
                        Err(_) => return Err(self.error("UTF-8 tidak valid")),
                    }
                    self.pos = end;
                }
            }
        }
    }

    fn array(&mut self) -> Result<Value, ParseError> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Value::Array(items));
        }
        loop {
            self.skip_ws();
            items.push(self.value()?);
            self.skip_ws();
            match self.bump() {
                Some(b',') => continue,
                Some(b']') => return Ok(Value::Array(items)),
                _ => return Err(self.error("array tidak ditutup")),
            }
        }
    }

    fn object(&mut self) -> Result<Value, ParseError> {
        self.expect(b'{')?;
        let mut map = BTreeMap::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Value::Object(map));
        }
        loop {
            self.skip_ws();
            let key = self.string()?;
            self.skip_ws();
            self.expect(b':')?;
            self.skip_ws();
            let value = self.value()?;
            map.insert(key, value);
            self.skip_ws();
            match self.bump() {
                Some(b',') => continue,
                Some(b'}') => return Ok(Value::Object(map)),
                _ => return Err(self.error("object tidak ditutup")),
            }
        }
    }
}

fn utf8_width(first: u8) -> usize {
    match first {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        _ => 4,
    }
}

/* ---------------------------------------------------------------- konversi */

impl From<bool> for Value {
    fn from(v: bool) -> Value {
        Value::Bool(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Value {
        Value::Number(v as f64)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Value {
        Value::Number(v as f64)
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Value {
        Value::Number(v as f64)
    }
}

impl From<usize> for Value {
    fn from(v: usize) -> Value {
        Value::Number(v as f64)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Value {
        Value::Number(v as f64)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Value {
        Value::Number(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Value {
        Value::String(v.to_string())
    }
}

impl From<String> for Value {
    fn from(v: String) -> Value {
        Value::String(v)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(items: Vec<T>) -> Value {
        Value::Array(items.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_object() {
        let v = Value::parse(r#"{"a":1,"b":"dua","c":true,"d":null}"#).unwrap();
        assert_eq!(v.get("a").and_then(Value::as_i64), Some(1));
        assert_eq!(v.get("b").and_then(Value::as_str), Some("dua"));
        assert_eq!(v.get("c").and_then(Value::as_bool), Some(true));
        assert_eq!(v.get("d"), Some(&Value::Null));
    }

    #[test]
    fn parses_nested_and_arrays() {
        let text = r#"{"topics":{"hangeul":{"best":80,"history":[1,2,3]}}}"#;
        let v = Value::parse(text).unwrap();
        let h = v.get("topics").unwrap().get("hangeul").unwrap();
        assert_eq!(h.get("best").and_then(Value::as_i64), Some(80));
        assert_eq!(
            h.get("history")
                .and_then(|a| a.at(2))
                .and_then(Value::as_i64),
            Some(3)
        );
    }

    #[test]
    fn roundtrip_korean_strings() {
        let original = Value::String("한국어 문법 — belajar".to_string());
        let json = original.to_json();
        assert_eq!(Value::parse(&json).unwrap(), original);
    }

    #[test]
    fn escapes_are_handled() {
        let v = Value::parse(r#""baris1\nbaris2\t\"kutip\"""#).unwrap();
        assert_eq!(v.as_str(), Some("baris1\nbaris2\t\"kutip\""));
    }

    #[test]
    fn unicode_escape_pairs() {
        // \ud55c\ud55c = 한한
        let v = Value::parse(r#""\ud55c\ud55c""#).unwrap();
        assert_eq!(v.as_str(), Some("한한"));
    }

    #[test]
    fn serializes_numbers_without_decimals() {
        assert_eq!(Value::from(42).to_json(), "42");
        assert_eq!(Value::from(3.5).to_json(), "3.5");
    }

    #[test]
    fn pretty_roundtrip() {
        let mut obj = Value::object();
        obj.insert("topics", Value::object())
            .entry("topics")
            .insert("hangeul", Value::object())
            .entry("hangeul")
            .insert("best", 90u32);
        let pretty = obj.to_json_pretty(2);
        assert!(pretty.contains('\n'));
        assert_eq!(Value::parse(&pretty).unwrap(), obj);
    }

    #[test]
    fn rejects_broken_input() {
        assert!(Value::parse("{").is_err());
        assert!(Value::parse("{\"a\":}").is_err());
        assert!(Value::parse("").is_err());
        assert!(Value::parse("{} sisa").is_err());
    }
}
