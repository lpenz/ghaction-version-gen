// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

pub mod git;

use std::collections::HashMap;
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

    fn insert_regex(&mut self, re: &Regex, s0: impl AsRef<str>) -> bool {
        let s = s0.as_ref();
        if let Some(m) = re.captures(s) {
            for name in re.capture_names().flatten() {
                self.insert(name, m.name(name).unwrap().as_str());
            }
            true
        } else {
            false
        }
    }

    pub fn get(&self, k: impl AsRef<str>) -> Option<&str> {
        self.data.get(k.as_ref()).map(|s| s.as_str())
    }

    pub fn parse_describe(&mut self, s0: impl AsRef<str>) -> Result<()> {
        let s = s0.as_ref();
        self.insert("git-describe-tags", s);
        let re = Regex::new(r"^(?P<tag_latest>.*)-(?P<distance>\d+)-g(?P<commit>[0-9a-f]+)$")?;
        let distance;
        if self.insert_regex(&re, s) {
            distance = self.get("distance").unwrap().to_string();
        } else {
            self.insert("tag_latest", s);
            self.insert("distance", "0");
            self.insert("tag_head", s);
            distance = "0".to_string();
        }
        self.insert("dash_distance", format!("-{}", distance));
        let re = Regex::new(r"^v?(?P<tag_ltrimv>.*)$")?;
        let tag_latest = self.get("tag_latest").unwrap().to_string();
        if let Some(m) = re.captures(&tag_latest) {
            self.insert("tag_latest_ltrimv", m.name("tag_ltrimv").unwrap().as_str());
        }
        let tag_head = self.get("tag_latest").unwrap().to_string();
        if let Some(m) = re.captures(&tag_head) {
            self.insert("tag_head_ltrimv", m.name("tag_ltrimv").unwrap().as_str());
        }
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

pub fn main() -> Result<()> {
    let info = Info::from_workspace()?;
    for (k, v) in &info {
        println!("Setting {}={}", k, v);
        println!("::set-output name={}::{}", k, v);
    }
    Ok(())
}
