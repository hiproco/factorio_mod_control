use std::{
    collections::{HashMap, VecDeque},
    io::prelude::*,
};
use tinyjson::*;

fn main() {
    let mut args = std::env::args().skip(1).collect::<VecDeque<_>>();
    match args[1].as_str() {
        "init" => init(&args.make_contiguous()),
        "new" => new(args.make_contiguous()),
        "update" => update(),
        _ => {}
    }
}

fn new(args: &[String]) {
    if let Some(name) = args.iter().next() {
        std::fs::create_dir(name).unwrap();
        std::env::set_current_dir(name).unwrap();
        init(args);
    }
}
fn init(args: &[String]) {
    let name: JsonValue = std::env::current_dir()
        .ok()
        .and_then(|p| {
            p.file_name()
                .and_then(|f| f.to_str().and_then(|s| s.parse().ok()))
        })
        .unwrap();
    const TEMPLATE: &str = r#"{
        "name": "temp",
        "version": "0.0.1",
        "title": "temp",
        "author": "temp",
        "factorio_version": "1.1",
        "dependencies": ["base"]
    }"#;
    let mut info = TEMPLATE.parse::<JsonValue>().unwrap();
    let map: &mut HashMap<_, _> = info.get_mut().unwrap();
    if let Some(modname) = map.get_mut("name") {
        *modname = name.clone();
    }
    if let Some(title) = map.get_mut("title") {
        *title = name;
    }
    if let Some(author) = map.get_mut("author") {
        // use some config
    }

    if std::path::Path::new("info.json")
        .try_exists()
        .unwrap_or(false)
    {
        std::fs::write("info.json", info.format().unwrap()).unwrap();
        std::fs::File::options()
            .create_new(true)
            .open("data.lua")
            .ok();
    }
}
fn update() {
    let mut json = std::fs::read_to_string("info.json")
        .ok()
        .and_then(|s| s.parse::<JsonValue>().ok())
        .unwrap();
    let mut name = String::new();
    let mut version = vec![];
    let info: &mut HashMap<_, _> = json.get_mut().unwrap();
    for (key, value) in info {
        match key.as_str() {
            "name" => {
                let JsonValue::String(value) = value else {
                    return;
                };
                name.clone_from(value);
            }
            "factorio_version" => {
                let JsonValue::String(value) = value else {
                    return;
                };
            }
            "version" => {
                let JsonValue::String(value) = value else {
                    return;
                };
                version = value
                    .split('.')
                    .filter_map(|v| v.parse().ok())
                    .collect::<Vec<u32>>();
                let mut args = std::env::args();
                let arg = args.nth(2).unwrap_or("".to_string());
                match arg.as_str() {
                    "minor" => {
                        version[2] += 1;
                    }
                    "middle" => {
                        version[1] += 1;
                    }
                    "major" => {
                        version[0] += 1;
                    }
                    "set-version" => {
                        version = args
                            .next()
                            .map(|v| v.split('.').filter_map(|v| v.parse().ok()).collect())
                            .unwrap_or(version);
                    }
                    _ => {}
                }
                version.truncate(3);
                version
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(".")
                    .clone_into(value);
            }
            "dependencies" => {
                let JsonValue::Array(deps) = value else {
                    return;
                };
                let deps = deps
                    .into_iter()
                    .filter_map(|dep| {
                        let JsonValue::String(dep) = dep else {
                            return None;
                        };
                        let mut dep = dep.split_whitespace().peekable();
                        let prefix = dep.next_if(|p| ["!", "?", "(?)", "~"].contains(p));
                        let modname = dep.next().unwrap();
                        let support_version = dep.next().zip(dep.next());
                        Some((prefix, modname, support_version))
                    })
                    .collect::<Vec<_>>();
            }
            _ => {
                let JsonValue::String(str) = value else {
                    return;
                };
            }
        }
    }
    let version = json.get::<HashMap<_, _>>().unwrap().get("version").unwrap();
    eprintln!("{version:?}");
    std::fs::write("info.json", json.format().unwrap().as_bytes());
}
fn add() {}
