// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

pub mod git;
pub mod rust;

use std::env;
use std::path::Path;
use std::str;

use anyhow::bail;
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
    pub rust_crate_version: Option<String>,
    pub version_mismatch: Option<String>,
    pub version_tagged: Option<String>,
    pub version_commit: Option<String>,
    pub version_docker_ci: String,
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
    }

    pub fn parse_files<P: AsRef<Path>>(&mut self, repo: P) -> Result<()> {
        if let Some(version) = rust::crate_version(repo)? {
            self.rust_crate_version = Some(version);
        }
        Ok(())
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
        Ok(())
    }

    pub fn eval(&mut self) -> Result<()> {
        // Evaluate trivial parameters:
        self.is_push_tag = match (self.is_push, self.is_tag) {
            (Some(a), Some(b)) => Some(a && b),
            _ => None,
        };
        self.is_push_main = match (self.is_push, self.is_main) {
            (Some(a), Some(b)) => Some(a && b),
            _ => None,
        };
        self.dash_distance = format!("-{}", self.distance);
        self.tag_distance = format!("{}{}", self.tag_latest, self.dash_distance);
        let re = Regex::new(r"^v?(?P<tag_ltrimv>.*)$")?;
        self.tag_latest_ltrimv = re.replace(&self.tag_latest, "$tag_ltrimv").into();
        self.tag_distance_ltrimv = re.replace(&self.tag_distance, "$tag_ltrimv").into();
        if let Some(ref tag_head) = self.tag_head {
            self.tag_head_ltrimv = Some(re.replace(tag_head, "$tag_ltrimv").into());
        }
        // Evaluate version outputs, correlating the previous variables
        if self.is_push_tag == Some(true) {
            self.version_tagged = self.tag_head_ltrimv.clone();
            self.version_commit = Some(self.tag_latest_ltrimv.clone());
            self.version_docker_ci = self.tag_latest_ltrimv.clone();
        } else if self.is_push_main == Some(true) {
            self.version_commit = Some(self.tag_distance_ltrimv.clone());
            self.version_docker_ci = "latest".to_string();
        } else {
            self.version_docker_ci = "null".to_string();
        }
        if self.is_push_tag == Some(true) || self.is_push_main == Some(true) {
            if let Some(ref version) = self.rust_crate_version {
                if version != &self.tag_latest_ltrimv {
                    self.version_mismatch = Some(format!(
                        "file=Cargo.toml::Version mismatch: tag {} != {} from Cargo.toml",
                        self.tag_latest_ltrimv, version
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn from_workspace<P: AsRef<Path>>(repo: P) -> Result<Info> {
        let _ = git::unshallow(&repo);
        let mut info = Info {
            commit: git::head_commit(&repo)?,
            ..Info::default()
        };
        info.parse_env(std::env::vars());
        info.parse_files(&repo)?;
        if let Ok(gitdescr) = git::describe(&repo) {
            info.parse_describe(gitdescr)?;
        }
        info.eval()?;
        Ok(info)
    }
}

pub fn bool2str(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

impl<'a> IntoIterator for &'a Info {
    type Item = (&'static str, &'a str);
    type IntoIter = std::vec::IntoIter<(&'static str, &'a str)>;
    fn into_iter(self) -> Self::IntoIter {
        let mut vec: Vec<(&'static str, &'a str)> = vec![
            ("commit", &self.commit),
            ("git_describe_tags", &self.git_describe_tags),
            ("tag_latest", &self.tag_latest),
            ("distance", &self.distance),
            ("dash_distance", &self.dash_distance),
            ("tag_distance", &self.dash_distance),
            ("tag_latest_ltrimv", &self.tag_latest_ltrimv),
            ("tag_distance_ltrimv", &self.tag_latest_ltrimv),
            ("version_docker_ci", &self.version_docker_ci),
        ];
        if let Some(ref v) = self.is_push {
            vec.push(("is_push", bool2str(*v)));
        }
        if let Some(ref v) = self.is_tag {
            vec.push(("is_tag", bool2str(*v)));
        }
        if let Some(ref v) = self.is_main {
            vec.push(("is_main", bool2str(*v)));
        }
        if let Some(ref v) = self.is_push_tag {
            vec.push(("is_push_tag", bool2str(*v)));
        }
        if let Some(ref v) = self.is_push_main {
            vec.push(("is_push_main", bool2str(*v)));
        }
        if let Some(ref t) = self.tag_head {
            vec.push(("tag_head", t));
        }
        if let Some(ref t) = self.tag_head_ltrimv {
            vec.push(("tag_head_ltrimv", t));
        }
        if let Some(ref t) = self.rust_crate_version {
            vec.push(("rust_crate_version", t));
        }
        if let Some(ref t) = self.version_mismatch {
            vec.push(("version_mismatch", t));
        }
        if let Some(ref t) = self.version_tagged {
            vec.push(("version_tagged", t));
        }
        if let Some(ref t) = self.version_commit {
            vec.push(("version_commit", t));
        }
        vec.into_iter()
    }
}

pub fn main() -> Result<()> {
    let info = Info::from_workspace(env::current_dir()?)?;
    for (k, v) in &info {
        println!("Setting {}={}", k, v);
        println!("::set-output name={}::{}", k, v);
    }
    if let Some(ref message) = info.version_mismatch {
        if info.is_push_tag == Some(true) {
            println!("::error {}", message);
            bail!("Version mismatch while pushing tag");
        } else {
            println!("::warning {}", message);
        }
    }
    Ok(())
}
