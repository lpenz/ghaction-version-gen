// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

pub mod git;

use std::str;

use anyhow::Result;

use regex::Regex;

#[derive(Debug, Default, Clone)]
pub struct Info {
    pub commit: String,
    pub git_describe_tags: String,
    pub tag_latest: String,
    pub distance: String,
    pub tag_head: Option<String>,
    pub dash_distance: String,
    pub tag_latest_ltrimv: String,
    pub tag_head_ltrimv: Option<String>,
}

impl Info {
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
        let re = Regex::new(r"^v?(?P<tag_ltrimv>.*)$")?;
        self.tag_latest_ltrimv = re.replace(&self.tag_latest, "$tag_ltrimv").into();
        if let Some(ref tag_head) = self.tag_head {
            self.tag_head_ltrimv = Some(re.replace(tag_head, "$tag_ltrimv").into());
        }
        Ok(())
    }

    pub fn from_workspace() -> Result<Info> {
        let _ = git::unshallow();
        let mut info = Info {
            commit: git::head_commit()?,
            ..Info::default()
        };
        if let Ok(gitdescr) = git::describe() {
            info.parse_describe(gitdescr)?;
        }
        Ok(info)
    }
}

impl<'a> IntoIterator for &'a Info {
    type Item = (&'static str, &'a str);
    type IntoIter = std::vec::IntoIter<(&'static str, &'a str)>;
    fn into_iter(self) -> Self::IntoIter {
        let mut v = vec![
            ("commit", self.commit.as_str()),
            ("git_describe_tags", self.git_describe_tags.as_str()),
            ("tag_latest", self.tag_latest.as_str()),
            ("distance", self.distance.as_str()),
            ("dash_distance", self.dash_distance.as_str()),
            ("tag_latest_ltrimv", self.tag_latest_ltrimv.as_str()),
        ];
        if let Some(t) = &self.tag_head {
            v.push(("tag_head", t.as_str()));
        }
        if let Some(t) = &self.tag_head_ltrimv {
            v.push(("tag_head_ltrimv", t.as_str()));
        }
        v.into_iter()
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
