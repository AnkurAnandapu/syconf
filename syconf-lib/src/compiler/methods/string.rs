use std::cmp::min;
use std::rc::Rc;

use crate::compiler::{Error, Value};

pub type StringMethod = dyn Fn(&str, &[Value]) -> Result<Value, Error>;

pub fn method(method_name: &str) -> Option<&'static StringMethod> {
    Some(match method_name {
        "parse_json" => &parse_json,
        "parse_yaml" => &parse_yaml,
        "parse_toml" => &parse_toml,
        "trim" => &trim,
        "contains" => &contains,
        "starts_with" => &starts_with,
        "ends_with" => &ends_with,
        "unindent" => &unindent,
        "lines" => &lines,
        _ => return None,
    })
}

fn parse_json(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'parse_json' does not take any arguments");
    let x = serde_json::from_str(string).map_err(|e| anyhow!("cannot parse JSON: {}", e))?;
    Ok(Value::HashMap(Rc::new(x)))
}

fn parse_yaml(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'parse_yaml' does not take any arguments");
    let x = serde_yaml::from_str(string).map_err(|e| anyhow!("cannot parse YAML: {}", e))?;
    Ok(Value::HashMap(Rc::new(x)))
}

fn parse_toml(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'parse_toml' does not take any arguments");
    let x = toml::de::from_str(string).map_err(|e| anyhow!("cannot parse TOML: {}", e))?;
    Ok(Value::HashMap(Rc::new(x)))
}

fn contains(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.len() == 1, "'contains' takes exactly one argument");
    let word_to_check = args[0].as_value_string()?.to_string();
    Ok(Value::Bool(string.contains(&word_to_check)).into())
}

#[test]
fn string_contains() {
    assert_eq!(
        crate::parse_string(
            r#"
            "hello".contains("ll") == true
            "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn starts_with(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.len() == 1, "'starts_with' takes exactly one argument");
    let word_to_check = args[0].as_value_string()?.to_string();
    Ok(Value::Bool(string.starts_with(&word_to_check)).into())
}

#[test]
fn string_starts_with() {
    assert_eq!(
        crate::parse_string(
            r#"
            "hello".starts_with("he") == true
            "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn ends_with(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.len() == 1, "'ends_with' takes exactly one argument");
    let word_to_check = args[0].as_value_string()?.to_string();
    Ok(Value::Bool(string.ends_with(&word_to_check)).into())
}

#[test]
fn string_ends_with() {
    assert_eq!(
        crate::parse_string(
            r#"
            "hello".ends_with("lo") == true
            "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn lines(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'lines' takes no arguments");
    let word_to_check = string
        .lines()
        .map(|x| Value::String(x.clone().into()))
        .collect::<Vec<Value>>();
    Ok(Value::List(word_to_check.into()))
}

#[test]
fn string_lines() {
    assert_eq!(
        crate::parse_string(
            r#"
            "hello
llo".lines() == ["hello", "llo"]
            "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn trim(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'trim' does not take any arguments");
    Ok(Value::String(string.trim().into()))
}

#[test]
fn trim_string() {
    assert_eq!(
        crate::parse_string(
            r#"
        "
            abc
            ".trim() == "abc"
    "#
        )
        .unwrap(),
        Value::Bool(true)
    )
}

fn unindent(string: &str, args: &[Value]) -> Result<Value, Error> {
    check!(args.is_empty(), "'unindent' does not take any arguments");
    let mut prefixed_whitespaces: Vec<&str> = string
        .lines()
        .filter(|x| !x.trim().is_empty())
        .map(|x| &x[..x.find(|s| !char::is_whitespace(s)).unwrap_or(0)])
        .collect();
    prefixed_whitespaces.sort_unstable();

    let prefix_len = match prefixed_whitespaces.len() {
        0 => return Ok(Value::String(string.into())),
        1 => prefixed_whitespaces[0].len(),
        _ => {
            let first: Vec<char> = prefixed_whitespaces[0].chars().collect();
            let last: Vec<char> = prefixed_whitespaces.last().unwrap().chars().collect();
            let mut cnt = 0;
            for ix in 0..min(first.len(), last.len()) {
                if first[ix] == last[ix] {
                    cnt += 1;
                }
            }
            cnt
        }
    };

    let out = string
        .lines()
        .map(|s| {
            if s.trim().is_empty() {
                ""
            } else {
                &s[prefix_len..]
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");

    Ok(Value::String(out.into()))
}

#[test]
fn func_unindent() {
    assert_eq!(
        crate::parse_string(
            r#"
        "

            abc
        def
                    ghk
        ".unindent()
    "#
        )
        .unwrap(),
        Value::String("\n\n    abc\ndef\n            ghk\n".into())
    )
}
