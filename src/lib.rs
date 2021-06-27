// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

pub mod git;

use std::collections::HashMap;
use std::fmt;
use std::str;

use anyhow::Result;

use regex::Regex;

#[derive(Debug, Default, Clone)]
pub struct Info {
    data: HashMap<String, String>,
}

impl Info {
    fn insert(&mut self, k: &str, v: impl AsRef<str>) {
        self.data.insert(k.to_string(), v.as_ref().to_string());
    }

    pub fn get(&self, k: impl AsRef<str>) -> Option<&str> {
        self.data.get(k.as_ref()).map(|s| s.as_str())
    }

    pub fn parse_describe(&mut self, s0: impl AsRef<str>) -> Result<()> {
        let s = s0.as_ref();
        self.insert("git-describe-tags", s);
        let re = Regex::new(r"^(?P<tag_latest>.*)-(?P<distance>\d+)-g(?P<commit>[0-9a-f]+)$")?;
        let distance;
        if let Some(m) = re.captures(s) {
            for label in &["tag_latest", "distance", "commit"] {
                self.insert(label, m.name(label).unwrap().as_str());
            }
            distance = m.name("distance").unwrap().as_str();
        } else {
            self.insert("tag_latest", s);
            self.insert("distance", "0");
            self.insert("tag_head", s);
            distance = "0";
        }
        self.insert("dash_distance", format!("-{}", distance));
        Ok(())
    }

    pub fn from_workspace() -> Result<Info> {
        let mut info = Info::default();
        if let Ok(gitdescr) = git::describe() {
            info.parse_describe(gitdescr)?;
        }
        if info.get("commit").is_none() {
            info.insert("commit", &git::head_commit()?);
        }
        Ok(info)
    }
}

impl<'a> IntoIterator for &'a Info {
    type Item = (&'a String, &'a String);
    type IntoIter = std::collections::hash_map::Iter<'a, String, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (k, v) in &self.data {
            writeln!(f, "::set-output name={}::{}", k, v)?;
        }
        Ok(())
    }
}

pub fn main() -> Result<()> {
    let info = Info::from_workspace()?;
    print!("{}", info);
    Ok(())
}
