// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

pub mod git;
pub mod rust;

use std::env;
use std::fs;
use std::io::Write;
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
    pub commit_main: Option<String>,
    pub is_main_here: Option<bool>,
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
    pub override_version_tagged: Option<String>,
    pub override_version_commit: Option<String>,
    pub override_version_docker_ci: Option<String>,
}

impl Info {
    pub fn parse_env(&mut self, enviter: impl Iterator<Item = (String, String)>) {
        for (k, v) in enviter {
            match k.as_str() {
                "GITHUB_EVENT_NAME" => {
                    self.is_push = Some(v == "push");
                }
                "GITHUB_REF" => {
                    self.is_tag = Some(v.starts_with("refs/tags/"));
                    self.is_main = Some(v == "refs/heads/main" || v == "refs/heads/master");
                }
                "OVERRIDE_VERSION_TAGGED" => {
                    self.override_version_tagged = Some(v);
                }
                "OVERRIDE_VERSION_COMMIT" => {
                    self.override_version_commit = Some(v);
                }
                "OVERRIDE_VERSION_DOCKER_CI" => {
                    self.override_version_docker_ci = Some(v);
                }
                _ => {}
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
            self.version_tagged = self
                .override_version_tagged
                .as_ref()
                .or(self.tag_head_ltrimv.as_ref())
                .cloned();
            self.version_commit = self
                .override_version_commit
                .as_ref()
                .or(Some(&self.tag_latest_ltrimv))
                .cloned();
            self.version_docker_ci = self
                .override_version_docker_ci
                .as_ref()
                .unwrap_or(&self.tag_latest_ltrimv)
                .clone();
        } else if self.is_push_main == Some(true) {
            self.version_commit = self
                .override_version_commit
                .as_ref()
                .or(Some(&self.tag_distance_ltrimv))
                .cloned();
            self.version_docker_ci = self
                .override_version_docker_ci
                .as_ref()
                .unwrap_or(&String::from("latest"))
                .clone();
        } else {
            self.version_docker_ci = self
                .override_version_docker_ci
                .as_ref()
                .unwrap_or(&String::from("null"))
                .clone();
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
        if self.is_push_tag == Some(true) && self.is_main_here != Some(true) {
            self.version_mismatch = Some(format!(
                "Version tag {} pushed over {}, but main branch is at {:?}",
                self.tag_latest_ltrimv, self.commit, self.commit_main
            ));
        }
        Ok(())
    }

    pub fn from_workspace<P: AsRef<Path>>(
        repo: P,
        enviter: impl Iterator<Item = (String, String)>,
    ) -> Result<Info> {
        let _ = git::unshallow(&repo);
        let commit = git::head_commit(&repo)?;
        let commit_main = git::ref_commit(&repo, "refs/remotes/origin/main")
            .or_else(|_| git::ref_commit(&repo, "refs/remotes/origin/master"))
            .or_else(|_| git::ref_commit(&repo, "refs/heads/main"))
            .or_else(|_| git::ref_commit(&repo, "refs/heads/master"))
            .ok();
        let is_main_here = commit_main.as_ref().map(|c| c == &commit);
        let mut info = Info {
            commit,
            commit_main,
            is_main_here,
            ..Info::default()
        };
        info.parse_env(enviter);
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
        if let Some(ref v) = self.commit_main {
            vec.push(("commit_main", v));
        }
        if let Some(ref v) = self.is_main_here {
            vec.push(("is_main_here", bool2str(*v)));
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
        if let Some(ref t) = self.override_version_tagged {
            vec.push(("override_version_tagged", t));
        }
        if let Some(ref t) = self.override_version_commit {
            vec.push(("override_version_commit", t));
        }
        if let Some(ref t) = self.override_version_docker_ci {
            vec.push(("override_version_docker_ci", t));
        }
        vec.into_iter()
    }
}

fn write_github_output(output_filename: &Path, info: &Info) -> Result<()> {
    let mut output = fs::File::options().append(true).open(output_filename)?;
    for (k, v) in info {
        writeln!(output, "{}={}", k, v)?;
    }
    Ok(())
}

pub fn main() -> Result<()> {
    let info = Info::from_workspace(env::current_dir()?, env::vars())?;
    for (k, v) in &info {
        println!("Setting {}={}", k, v);
    }
    if let Ok(output_filename) = env::var("GITHUB_OUTPUT") {
        write_github_output(Path::new(&output_filename), &info)?;
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
