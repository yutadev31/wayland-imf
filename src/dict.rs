use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

fn parse_line(line: &str) -> Option<(String, Vec<String>)> {
    if line.starts_with(";;") || line.trim().is_empty() {
        return None;
    }

    let mut parts = line.splitn(2, ' ');
    let key = parts.next()?.to_string();
    let rest = parts.next()?;

    // /で囲まれた部分を取り出す
    let candidates: Vec<String> = rest
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.split(';').next().unwrap().to_string()) // 注釈除去
        .collect();

    Some((key, candidates))
}

pub fn load_dict() -> HashMap<String, Vec<String>> {
    let file = File::open("tmp/SKK-JISYO.M").unwrap();
    let reader = BufReader::new(file);

    let mut dict = HashMap::new();
    for line in reader.lines() {
        let line = line.unwrap();
        if let Some((key, value)) = parse_line(&line) {
            dict.insert(key, value);
        }
    }
    dict
}
