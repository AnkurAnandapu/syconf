use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use crate::compiler::{Error, Value};
use crate::compiler::value_extraction::ValueExtractor;
use crate::parse_string;
use std::collections::HashMap;

pub fn lookup(function_name: &str) -> Option<&'static dyn Fn(&[Value]) -> Result<Value, Error>> {
    Some(match function_name {
        "read_file" => &read_file,
        "getenv" => &getenv,
        "concat" => &concat,
        "merge" => &merge,
        "fold" => &fold,
        _ => return None,
    })
}

fn read_file(args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() == 1, "'read_file' expects a single string argument");
    let file_name = args[0].as_str()?;

    let mut buf = String::new();
    let mut f = File::open(file_name)
        .map_err(|e| anyhow!("Cannot open file '{}': {}", file_name, e))?;
    f.read_to_string(&mut buf)
        .map_err(|e| anyhow!("Cannot read file '{}': {}", file_name, e))?;
    Ok(Value::String(Rc::new(buf)))
}

fn getenv(args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() > 0 && args.len() <=2, "'getenv' expects a string argument with an optional default value");
    let envname = args[0].as_str()?;
    std::env::var(envname)
        .map(|x| Value::String(Rc::new(x)))
        .or_else(|e| if args.len() == 2 {
            Ok(args[1].clone())
        } else {
            Err(anyhow!("Environment variable '{}' is not set", envname))
        })
}

pub fn concat_strings(args: &[Value]) -> Result<Value, Error> {
    let mut out = String::new();
    for s in args {
        match s {
            Value::String(s) => out.push_str(s.as_str()),
            Value::Int(x) => out.push_str(x.to_string().as_str()),
            Value::Bool(x) => out.push_str(x.to_string().as_str()),
            _ => bail!("Cannot format a non-primitive type"),
        }
    }
    Ok(Value::String(Rc::new(out)))
}


#[test]
fn func_concat_strings() {
    assert_eq!(parse_string(r#"
        let name = "mike"
        in
        "Name: ${name}"
    "#).unwrap(), Value::String(Rc::new("Name: mike".to_string())));
}

fn concat(args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() > 0, "Concat requires at least one argument as a list");
    let mut out = args[0].as_list()?.clone();
    for x in &args[1..] {
        let mut li = x.as_list()?.clone();
        out.append(&mut li);
    }
    Ok(Value::List(Rc::new(out)))
}


#[test]
fn func_concat() {
    assert_eq!(parse_string(r#"concat([1],[2,3],[4])"#).unwrap(), Value::List(Rc::new(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
    ])));
}

fn merge(args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() > 0, "Merge requires at least one argument as a hashmap or a list of hashmaps");
    let hm_list = if let Value::List(list) = &args[0] {
        ensure!(args.len() == 1, "Merge expects either multiple hashmaps or a single list of hashmaps");
        list.as_slice()
    }else{
        args
    };
    let mut out = hm_list[0].as_hashmap()?.clone();
    for x in &hm_list[1..] {
        let li = x.as_hashmap()?.clone();
        out.extend(li.into_iter());
    }
    Ok(Value::HashMap(Rc::new(out)))
}


#[test]
fn func_merge() {
    let mut hm = HashMap::new();
    hm.insert("name".to_string(), Value::String(Rc::new("alexei".to_string())));
    hm.insert("age".to_string(), Value::Int(40));
    assert_eq!(parse_string(r#"merge(
        {name: "john"},
        {name: "alexei"},
        {age: 40},
    )"#).unwrap(), Value::HashMap(Rc::new(hm)));

    assert_eq!(parse_string(r#"merge([
        {name: "john"},
        {age: 40},
    ]) == {name: "john", age: 40}"#).unwrap(), Value::Bool(true));
}

fn fold(args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() == 3, "Fold requires 3 arguments (initial value, accumulation function, list or hashmap)");
    let func = args[1].as_func()?;
    match &args[2] {
        Value::List(list) => {
            let mut out = args[0].clone();
            for (ix, val) in list.iter().enumerate() {
                let args = &[
                    out.clone(),
                    Value::Int(ix as i32),
                    val.clone(),
                ];
                out = func.call(args)?;
            }
            Ok(out)
        },
        Value::HashMap(hm) => {
            let mut out = args[0].clone();
            for (ix, val) in hm.iter() {
                let args = &[
                    out.clone(),
                    Value::String(Rc::new(ix.clone())),
                    val.clone(),
                ];
                out = func.call(args)?;
            }
            Ok(out)
        }
        _ => bail!("3rd argument must be either a list or a hashmap"),
    }
}

#[test]
fn func_fold() {
    assert_eq!(parse_string(r#"fold(0, (acc, ix, val) => acc + val, [1,2,3])"#).unwrap(), Value::Int(6));
    assert_eq!(parse_string(r#"fold(0, (acc, ix, val) => acc + val, {
        aa: 1,
        bb: 2,
        cc: 3
    })"#).unwrap(), Value::Int(6));
}