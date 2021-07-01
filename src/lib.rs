// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

pub mod git;

use std::str;

use anyhow::Result;

use regex::Regex;

#[derive(Debug, Default, Clone)]
pub struct Info {
    pub is_push: Option<bool>,
    pub is_tag: Option<bool>,
    pub is_main: Option<bool>,
    pub is_push_tag: Option<bool>,
    pub is_push_main: Option<bool>,
    pub commit: String,
    pub git_describe_tags: String,
    pub tag_latest: String,
    pub distance: String,
    pub dash_distance: String,
    pub tag_distance: String,
    pub tag_head: Option<String>,
    pub tag_latest_ltrimv: String,
    pub tag_distance_ltrimv: String,
    pub tag_head_ltrimv: Option<String>,
    pub version_tagged: Option<String>,
    pub version_commit: Option<String>,
}

impl Info {
    pub fn parse_env(&mut self, enviter: impl Iterator<Item = (String, String)>) {
        for (k, v) in enviter {
            if k == "GITHUB_EVENT_NAME" {
                self.is_push = Some(v == "push");
            }
            if k == "GITHUB_REF" {
                self.is_tag = Some(v.starts_with("refs/tags/"));
                self.is_main = Some(v == "refs/heads/main" || v == "refs/heads/master");
            }
        }
        self.is_push_tag = match (self.is_push, self.is_tag) {
            (Some(a), Some(b)) => Some(a && b),
            _ => None,
        };
        self.is_push_main = match (self.is_push, self.is_main) {
            (Some(a), Some(b)) => Some(a && b),
            _ => None,
        };
    }

    pub fn parse_describe(&mut self, s0: impl AsRef<str>) -> Result<()> {
        let s = s0.as_ref();
        self.git_describe_tags = s.into();
        let re = Regex::new(r"^(?P<tag_latest>.*)-(?P<distance>\d+)-g[0-9a-f]+$")?;
        if let Some(m) = re.captures(s) {
            self.tag_latest = m.name("tag_latest").unwrap().as_str().into();
            self.distance = m.name("distance").unwrap().as_str().into();
        } else {
            self.tag_latest = s.into();
            self.distance = "0".into();
            self.tag_head = Some(s.into());
        }
        self.dash_distance = format!("-{}", self.distance);
        self.tag_distance = format!("{}{}", self.tag_latest, self.dash_distance);
        let re = Regex::new(r"^v?(?P<tag_ltrimv>.*)$")?;
        self.tag_latest_ltrimv = re.replace(&self.tag_latest, "$tag_ltrimv").into();
        self.tag_distance_ltrimv = re.replace(&self.tag_distance, "$tag_ltrimv").into();
        if let Some(ref tag_head) = self.tag_head {
            self.tag_head_ltrimv = Some(re.replace(tag_head, "$tag_ltrimv").into());
        }
        if self.is_push_tag == Some(true) {
            self.version_tagged = self.tag_head_ltrimv.clone();
        }
        if self.is_push_main == Some(true) {
            self.version_commit = Some(self.tag_distance_ltrimv.clone());
        }
        Ok(())
    }

    pub fn from_workspace() -> Result<Info> {
        let _ = git::unshallow();
        let mut info = Info {
            commit: git::head_commit()?,
            ..Info::default()
        };
        info.parse_env(std::env::vars());
        if let Ok(gitdescr) = git::describe() {
            info.parse_describe(gitdescr)?;
        }
        Ok(info)
    }
}

impl<'a> IntoIterator for &'a Info {
    type Item = (&'static str, String);
    type IntoIter = std::vec::IntoIter<(&'static str, String)>;
    fn into_iter(self) -> Self::IntoIter {
        let mut vec: Vec<(&'static str, String)> = vec![
            ("commit", self.commit.clone()),
            ("git_describe_tags", self.git_describe_tags.clone()),
            ("tag_latest", self.tag_latest.clone()),
            ("distance", self.distance.clone()),
            ("dash_distance", self.dash_distance.clone()),
            ("tag_distance", self.dash_distance.clone()),
            ("tag_latest_ltrimv", self.tag_latest_ltrimv.clone()),
            ("tag_distance_ltrimv", self.tag_latest_ltrimv.clone()),
        ];
        if let Some(v) = &self.is_push {
            vec.push(("is_push", format!("{}", v)));
        }
        if let Some(v) = &self.is_tag {
            vec.push(("is_tag", format!("{}", v)));
        }
        if let Some(v) = &self.is_main {
            vec.push(("is_main", format!("{}", v)));
        }
        if let Some(v) = &self.is_push_tag {
            vec.push(("is_push_tag", format!("{}", v)));
        }
        if let Some(v) = &self.is_push_main {
            vec.push(("is_push_main", format!("{}", v)));
        }
        if let Some(t) = &self.tag_head {
            vec.push(("tag_head", t.into()));
        }
        if let Some(t) = &self.tag_head_ltrimv {
            vec.push(("tag_head_ltrimv", t.into()));
        }
        if let Some(t) = &self.version_tagged {
            vec.push(("version_tagged", t.into()));
        }
        if let Some(t) = &self.version_commit {
            vec.push(("version_commit", t.into()));
        }
        vec.into_iter()
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
