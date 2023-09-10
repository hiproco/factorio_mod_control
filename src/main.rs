use std::{
    collections::{HashMap, VecDeque},
    io::prelude::*,
};
use tinyjson::*;

fn main() {
    let mut args = std::env::args().skip(1).collect::<VecDeque<_>>();
    match args[0].as_str() {
        "init" => init(&args.make_contiguous()),
        "new" => new(args.make_contiguous()),
        "update" => update(),
        _ => {}
    }
}

fn new(args: &[String]) {
    if let Some(name) = args.get(1) {
        std::fs::create_dir(name).unwrap();
        std::env::set_current_dir(name).unwrap();
        init(args);
    }
}
fn init(args: &[String]) {
    let name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().and_then(|f| f.to_str().map(|s| s.to_owned())))
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
        *modname = name.clone().into();
    }
    if let Some(title) = map.get_mut("title") {
        *title = name.into();
    }
    if let Some(author) = map.get_mut("author") {
        let config_author = config::config().author();
        *author = config_author.to_string().into();
        // choose configuration format
        // place in ~/.config/fmc/ folder
    }

    if !std::path::Path::new("info.json")
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
                version = version_vector(&value);
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
                        version = args.next().map(|v| version_vector(v)).unwrap_or(version);
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

fn set(args: &[String]) {}

fn version_vector(version: impl AsRef<str>) -> Vec<u32> {
    version
        .as_ref()
        .split('.')
        .flat_map(|s| s.parse())
        .collect()
}

mod config {
    use std::{io::Read, sync::OnceLock};

    #[derive(Debug)]
    pub struct Config {
        author: String,
        // update zip directory?
    }
    impl Config {
        pub fn author(&self) -> &str {
            &self.author
        }
    }

    static CONFIG: OnceLock<Config> = OnceLock::new();
    pub fn config() -> &'static Config {
        let init = || {
            let mut f = std::fs::File::open("~/.config/fmc/config.config").unwrap();
            let mut author = String::new();
            f.read_to_string(&mut author).unwrap();
            Config { author }
        };
        CONFIG.get_or_init(init)
    }
}
