use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

pub trait CandidateProvider {
    fn candidates_for(&self, text: &str) -> Vec<String>;
}

fn parse_line(line: &str) -> Option<(String, Vec<String>)> {
    if line.starts_with(";;") || line.trim().is_empty() {
        return None;
    }

    let mut parts = line.splitn(2, ' ');
    let key = parts.next()?.to_string();
    let rest = parts.next()?;

    let candidates = rest
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.split(';').next().unwrap_or_default().to_string())
        .collect();

    Some((key, candidates))
}

pub struct StaticDictionary {
    entries: HashMap<String, Vec<String>>,
}

impl StaticDictionary {
    pub fn empty() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut entries = HashMap::new();
        for line in reader.lines() {
            let line = line?;
            if let Some((key, value)) = parse_line(&line) {
                entries.insert(key, value);
            }
        }

        Ok(Self { entries })
    }
}

impl CandidateProvider for StaticDictionary {
    fn candidates_for(&self, text: &str) -> Vec<String> {
        self.entries.get(text).cloned().unwrap_or_default()
    }
}
