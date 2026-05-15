// ZON (Zig Object Notation) parser & serializer
// Equivalent to Python zon_parser.py

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZonEnum(pub String);

impl std::fmt::Display for ZonEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ".{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ZonValue {
    Null,
    Bool(bool),
    Int(i64),
    String(String),
    Enum(ZonEnum),
    Array(Vec<ZonValue>),
    Object(BTreeMap<String, ZonValue>),
}

impl ZonValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ZonValue::String(s) => Some(s),
            ZonValue::Enum(e) => Some(&e.0),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ZonValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ZonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum Token {
    LBrace,
    RBrace,
    DotId(String),
    Int(i64),
    Bool(bool),
    Null,
    String(String),
}

fn tokenize(text: &str) -> Result<Vec<(Token, usize)>, String> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < n {
        let ch = chars[i];

        // Whitespace
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        // Comments
        if ch == '/' && i + 1 < n && chars[i + 1] == '/' {
            while i < n && chars[i] != '\n' && chars[i] != '\r' {
                i += 1;
            }
            continue;
        }

        // Strings
        if ch == '"' {
            i += 1;
            let mut s = String::new();
            while i < n && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < n {
                    i += 1;
                    match chars[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        other => {
                            s.push('\\');
                            s.push(other);
                        }
                    }
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if i >= n {
                return Err("Unterminated string".to_string());
            }
            i += 1; // skip closing "
            tokens.push((Token::String(s), i));
            continue;
        }

        // Numbers
        if ch == '-' && i + 1 < n && chars[i + 1].is_ascii_digit() {
            i += 1;
            let start = i;
            while i < n && chars[i].is_ascii_digit() {
                i += 1;
            }
            let num_str: String = chars[start - 1..i].iter().collect();
            let num: i64 = num_str.parse().map_err(|_| "Invalid number")?;
            tokens.push((Token::Int(num), i));
            continue;
        }
        if ch.is_ascii_digit() {
            let start = i;
            while i < n && chars[i].is_ascii_digit() {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            let num: i64 = num_str.parse().map_err(|_| "Invalid number")?;
            tokens.push((Token::Int(num), i));
            continue;
        }

        // Keywords
        if ch == 't' && text[i..].starts_with("true") && keyword_boundary(&chars, i, 4) {
            tokens.push((Token::Bool(true), i));
            i += 4;
            continue;
        }
        if ch == 'f' && text[i..].starts_with("false") && keyword_boundary(&chars, i, 5) {
            tokens.push((Token::Bool(false), i));
            i += 5;
            continue;
        }
        if ch == 'n' && text[i..].starts_with("null") && keyword_boundary(&chars, i, 4) {
            tokens.push((Token::Null, i));
            i += 4;
            continue;
        }

        // Dot followed by brace (struct/array start) — skip the dot
        if ch == '.' && i + 1 < n && chars[i + 1] == '{' {
            i += 1;
            continue;
        }

        // Dot ID
        if ch == '.' && i + 1 < n && (chars[i + 1].is_alphabetic() || chars[i + 1] == '_') {
            i += 1;
            let start = i;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let id: String = chars[start..i].iter().collect();
            tokens.push((Token::DotId(id), i));
            continue;
        }

        // Braces
        if ch == '{' {
            tokens.push((Token::LBrace, i));
            i += 1;
            continue;
        }
        if ch == '}' {
            tokens.push((Token::RBrace, i));
            i += 1;
            continue;
        }

        // Comma
        if ch == ',' {
            i += 1;
            continue;
        }

        // Equals sign (assignment operator, skip it)
        if ch == '=' {
            i += 1;
            continue;
        }

        // Bare dot (not followed by { or letter) — skip it
        if ch == '.' {
            i += 1;
            continue;
        }

        return Err(format!("Unexpected character '{}' at position {}", ch, i));
    }

    Ok(tokens)
}

fn keyword_boundary(chars: &[char], start: usize, len: usize) -> bool {
    let end = start + len;
    end >= chars.len() || !(chars[end].is_alphanumeric() || chars[end] == '_')
}

pub fn parse_zon(text: &str) -> Result<ZonValue, String> {
    let tokens = tokenize(text)?;
    let mut parser = ZonParser::new(tokens);
    parser.parse_value()
}

const MAX_DEPTH: usize = 64;

struct ZonParser {
    tokens: Vec<(Token, usize)>,
    pos: usize,
    depth: usize,
}

impl ZonParser {
    fn new(tokens: Vec<(Token, usize)>) -> Self {
        ZonParser { tokens, pos: 0, depth: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn parse_value(&mut self) -> Result<ZonValue, String> {
        match self.peek() {
            None => Err("Unexpected end of input".into()),
            Some(Token::LBrace) => self.parse_brace(),
            Some(Token::DotId(_)) => self.parse_enum(),
            Some(&Token::Int(v)) => { self.advance(); Ok(ZonValue::Int(v)) }
            Some(Token::Bool(v)) => { let v = *v; self.advance(); Ok(ZonValue::Bool(v)) }
            Some(Token::Null) => { self.advance(); Ok(ZonValue::Null) }
            Some(Token::String(_)) => self.parse_string(),
            Some(tok) => Err(format!("Unexpected token {:?}", tok)),
        }
    }

    fn parse_string(&mut self) -> Result<ZonValue, String> {
        if let Some((Token::String(s), _)) = self.tokens.get(self.pos) {
            let s = s.clone();
            self.advance();
            Ok(ZonValue::String(s))
        } else {
            Err("Expected string".into())
        }
    }

    fn parse_enum(&mut self) -> Result<ZonValue, String> {
        if let Some((Token::DotId(id), _)) = self.tokens.get(self.pos) {
            let id = id.clone();
            self.advance();
            Ok(ZonValue::Enum(ZonEnum(id)))
        } else {
            Err("Expected .identifier".into())
        }
    }

    fn parse_brace(&mut self) -> Result<ZonValue, String> {
        if self.depth >= MAX_DEPTH {
            return Err(format!("嵌套深度超过限制 ({})", MAX_DEPTH));
        }
        self.depth += 1;
        self.advance(); // skip {
        let result = match self.peek() {
            Some(Token::RBrace) => {
                self.advance();
                Ok(ZonValue::Object(BTreeMap::new()))
            }
            Some(Token::LBrace) => self.parse_array(),
            Some(Token::DotId(_)) => {
                let saved_pos = self.pos;
                self.advance();
                let is_struct = match self.peek() {
                    Some(Token::LBrace) | Some(Token::DotId(_)) | Some(Token::String(_))
                    | Some(Token::Int(_)) | Some(Token::Bool(_)) | Some(Token::Null) => true,
                    Some(Token::RBrace) | None => false,
                };
                self.pos = saved_pos;
                if is_struct {
                    self.parse_struct()
                } else {
                    self.parse_array()
                }
            }
            _ => self.parse_array(),
        };
        self.depth -= 1;
        result
    }

    fn parse_array(&mut self) -> Result<ZonValue, String> {
        let mut items = Vec::new();
        while self.peek().is_some_and(|t| !matches!(t, Token::RBrace)) {
            items.push(self.parse_value()?);
            // skip optional comma
            if self.peek().is_some_and(|t| matches!(t, Token::RBrace)) {
                break;
            }
        }
        if let Some(Token::RBrace) = self.peek() {
            self.advance();
        }
        Ok(ZonValue::Array(items))
    }

    fn parse_struct(&mut self) -> Result<ZonValue, String> {
        let mut obj = BTreeMap::new();
        loop {
            match self.peek() {
                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }
                None => return Err("Unexpected end of input, expected '}'".into()),
                _ => {}
            }
            let key = match self.peek() {
                Some(Token::DotId(id)) => id.clone(),
                other => return Err(format!("Expected field name, got {:?}", other)),
            };
            self.advance();
            if matches!(self.peek(), Some(Token::LBrace) | Some(Token::DotId(_)) | Some(Token::String(_)) | Some(Token::Int(_)) | Some(Token::Bool(_)) | Some(Token::Null)) {
                let value = self.parse_value()?;
                obj.insert(key, value);
            } else {
                return Err(format!("Expected value after '{}'", key));
            }
        }
        Ok(ZonValue::Object(obj))
    }
}

pub fn serialize_zon(value: &ZonValue) -> String {
    serialize_zon_with_indent(value, 0)
}

/// Serialize a BTreeMap as a ZON Object directly, avoiding clone.
pub fn serialize_zon_object(obj: &BTreeMap<String, ZonValue>) -> String {
    if obj.is_empty() {
        ".{}".into()
    } else {
        let pad = "";
        let inner_pad = "    ";
        let fields: Vec<String> = obj.iter().map(|(k, v)| {
            format!("{}.{} = {},", inner_pad, k, serialize_zon_with_indent(v, 1))
        }).collect();
        format!(".{{\n{}\n{}}}", fields.join("\n"), pad)
    }
}

fn serialize_zon_with_indent(value: &ZonValue, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let inner_pad = "    ".repeat(indent + 1);
    match value {
        ZonValue::Null => "null".into(),
        ZonValue::Bool(b) => if *b { "true" } else { "false" }.into(),
        ZonValue::Int(i) => i.to_string(),
        ZonValue::String(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\t', "\\t").replace('\r', "\\r");
            format!("\"{}\"", escaped)
        }
        ZonValue::Enum(e) => format!(".{}", e.0),
        ZonValue::Array(arr) => {
            if arr.is_empty() {
                ".{}".into()
            } else {
                let items: Vec<String> = arr.iter().map(|v| format!("{}{},", inner_pad, serialize_zon_with_indent(v, indent + 1))).collect();
                format!(".{{\n{}\n{}}}", items.join("\n"), pad)
            }
        }
        ZonValue::Object(obj) => {
            if obj.is_empty() {
                ".{}".into()
            } else {
                let fields: Vec<String> = obj.iter().map(|(k, v)| {
                    format!("{}.{} = {},", inner_pad, k, serialize_zon_with_indent(v, indent + 1))
                }).collect();
                format!(".{{\n{}\n{}}}", fields.join("\n"), pad)
            }
        }
    }
}
