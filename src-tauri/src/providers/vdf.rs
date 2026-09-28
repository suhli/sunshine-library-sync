//! Minimal KeyValues parser: nested objects, quoted/bare tokens, comments and
//! escaped quotes/backslashes. Fails closed on incomplete launcher writes.
use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq)]
pub enum Value { Text(String), Object(BTreeMap<String, Value>) }
impl Value {
    pub fn object(&self) -> Option<&BTreeMap<String, Value>> { if let Self::Object(x) = self { Some(x) } else { None } }
    pub fn text(&self) -> Option<&str> { if let Self::Text(x) = self { Some(x) } else { None } }
}
#[derive(Debug, PartialEq)]
enum Token { Text(String), Open, Close }
pub fn parse(input: &str) -> Result<BTreeMap<String, Value>> {
    let mut chars = input.trim_start_matches('\u{feff}').chars().peekable();
    let mut tokens = Vec::new();
    while let Some(c) = chars.next() {
        if c.is_whitespace() { continue; }
        match c {
            '/' if chars.peek() == Some(&'/') => { for x in chars.by_ref() { if x == '\n' { break; } } },
            '{' => tokens.push(Token::Open), '}' => tokens.push(Token::Close),
            '"' => {
                let mut s = String::new(); let mut closed = false;
                while let Some(x) = chars.next() {
                    if x == '"' { closed = true; break; }
                    if x == '\\' {
                        match chars.peek() { Some('\\' | '"') => s.push(chars.next().unwrap()), _ => s.push(x) }
                    } else { s.push(x); }
                }
                if !closed { bail!("Unterminated VDF string"); }
                tokens.push(Token::Text(s));
            },
            _ => { let mut s = c.to_string(); while let Some(x) = chars.peek() { if x.is_whitespace() || *x == '{' || *x == '}' { break; } s.push(chars.next().unwrap()); } tokens.push(Token::Text(s)); }
        }
    }
    fn object(tokens: &[Token], cursor: &mut usize, nested: bool, depth: usize) -> Result<BTreeMap<String, Value>> {
        if depth > 32 { bail!("VDF nesting exceeds limit"); }
        let mut out = BTreeMap::new();
        while *cursor < tokens.len() {
            if tokens[*cursor] == Token::Close { if !nested { bail!("Unexpected closing brace"); } *cursor += 1; return Ok(out); }
            let Token::Text(key) = &tokens[*cursor] else { bail!("Expected VDF key"); }; let key = key.clone(); *cursor += 1;
            let v = match tokens.get(*cursor).context("Missing VDF value")? {
                Token::Text(x) => { *cursor += 1; Value::Text(x.clone()) },
                Token::Open => { *cursor += 1; Value::Object(object(tokens, cursor, true, depth + 1)?) },
                Token::Close => bail!("Missing VDF value"),
            };
            if out.insert(key, v).is_some() { bail!("Duplicate VDF key"); }
        }
        if nested { bail!("Unclosed VDF object"); } Ok(out)
    }
    object(&tokens, &mut 0, false, 0)
}
pub fn text<'a>(map: &'a BTreeMap<String, Value>, key: &str) -> Result<&'a str> { map.get(key).and_then(Value::text).with_context(|| format!("Missing VDF field: {key}")) }
