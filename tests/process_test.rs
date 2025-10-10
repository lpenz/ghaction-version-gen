// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::env;
use std::fs::File;
use std::io::Write;
use std::iter;
use std::process::Command;

use color_eyre::Result;
use color_eyre::eyre::ensure;

use ghaction_version_gen::Info;
use ghaction_version_gen::git;
use ghaction_version_gen::python;
use ghaction_version_gen::rust;

#[cfg(test)]
fn environ_reset() {
    unsafe {
        env::remove_var("GITHUB_EVENT_NAME");
        env::remove_var("GITHUB_REF");
    }
}

#[test]
fn basic() -> Result<()> {
    environ_reset();
    let gitdesc = "1.3.1-20-gc5f7a99";
    let mut info = Info::default();
    info.parse_describe(gitdesc)?;
    assert_eq!(info.git_describe_tags, gitdesc);
    assert_eq!(info.tag_latest, "1.3.1");
    Ok(())
}

#[derive(Debug)]
struct TmpGit {
    pub repo: tempfile::TempDir,
}

impl TmpGit {
    pub fn new() -> Result<TmpGit> {
        let tmpgit = TmpGit {
            repo: tempfile::tempdir()?,
        };
        tmpgit.run(&["git", "init", "-b", "main"])?;
        tmpgit.run(&["git", "config", "--local", "user.name", "username"])?;
        tmpgit.run(&["git", "config", "--local", "user.email", "user@email.net"])?;
        tmpgit.run(&["git", "config", "--local", "commit.gpgsign", "false"])?;
        Ok(tmpgit)
    }

    pub fn run(&self, cmd: &[&str]) -> Result<()> {
        let status = Command::new(cmd[0])
            .current_dir(&self.repo)
            .args(&cmd[1..])
            .status()?;
        ensure!(status.success(), "error running command");
        Ok(())
    }

    pub fn file_write(&self, basename: &str, contents: &str) -> Result<()> {
        let path = self.repo.path().join(basename);
        let mut fd = File::create(path)?;
        fd.write_all(contents.as_bytes())?;
        Ok(())
    }

    fn info_get(&self) -> Result<Info> {
        let mut info = Info::from_workspace(&self.repo, iter::empty())?;
        info.is_push = None;
        info.is_tag = None;
        info.is_main = None;
        info.eval()?;
        Ok(info)
    }
}

#[test]
fn gitrepo() -> Result<()> {
    environ_reset();
    let repo = TmpGit::new()?;
    repo.file_write("foo.txt", "Hello, world!")?;
    repo.run(&["git", "add", "foo.txt"])?;
    repo.run(&["git", "commit", "-m", "first commit"])?;
    // Check with no tag is present
    let commit1 = git::head_commit(&repo.repo)?;
    let info = repo.info_get()?;
    assert_eq!(info.commit, commit1.as_str());
    assert_eq!(info.commit_main.clone().unwrap(), commit1);
    assert_eq!(info.is_main_here, Some(true));
    assert_eq!(info.tag_latest, "");
    assert_eq!(info.tag_head, None);
    assert_eq!(info.distance, None);
    assert_eq!(info.version_docker_ci, "null");
    // Check tag on HEAD
    repo.run(&["git", "tag", "v1.0.0"])?;
    let mut info = repo.info_get()?;
    info.is_push = Some(true);
    info.is_tag = Some(true);
    info.is_main = Some(true);
    info.eval()?;
    assert_eq!(info.commit, commit1.as_str());
    assert_eq!(info.git_describe_tags, "v1.0.0");
    assert_eq!(info.tag_latest, "v1.0.0");
    assert_eq!(info.distance, Some("0".to_string()));
    assert_eq!(info.dash_distance, Some("-0".to_string()));
    assert_eq!(info.tag_distance, Some("v1.0.0-0".to_string()));
    assert_eq!(info.tag_head, Some("v1.0.0".to_string()));
    assert_eq!(info.tag_latest_ltrimv, Some("1.0.0".to_string()));
    assert_eq!(info.tag_distance_ltrimv, Some("1.0.0-0".to_string()));
    assert_eq!(info.tag_head_ltrimv, Some("1.0.0".to_string()));
    assert_eq!(info.version_tagged, Some("1.0.0".to_string()));
    assert_eq!(info.version_commit, Some("1.0.0".to_string()));
    assert_eq!(info.version_docker_ci, "1.0.0");
    // Check tag behind HEAD
    repo.file_write("bar.txt", "Hello again!")?;
    repo.run(&["git", "add", "bar.txt"])?;
    repo.run(&["git", "commit", "-m", "second commit"])?;
    let commit2 = git::head_commit(&repo.repo)?;
    let mut info = repo.info_get()?;
    info.is_push = Some(true);
    info.is_tag = Some(false);
    info.is_main = Some(true);
    info.parse_files(&repo.repo)?;
    info.eval()?;
    assert_eq!(info.commit, commit2.as_str());
    assert_eq!(info.git_describe_tags, format!("v1.0.0-1-g{}", commit2));
    assert_eq!(info.tag_latest, "v1.0.0");
    assert_eq!(info.distance, Some("1".to_string()));
    assert_eq!(info.dash_distance, Some("-1".to_string()));
    assert_eq!(info.tag_distance, Some("v1.0.0-1".to_string()));
    assert_eq!(info.tag_head, None);
    assert_eq!(info.tag_latest_ltrimv, Some("1.0.0".to_string()));
    assert_eq!(info.tag_distance_ltrimv, Some("1.0.0-1".to_string()));
    assert_eq!(info.tag_head_ltrimv, None);
    assert_eq!(info.version_tagged, None);
    assert_eq!(info.version_commit, Some("1.0.0-1".to_string()));
    assert_eq!(info.version_docker_ci, "latest");
    // Check overrides
    info.parse_env(
        vec![
            ("OVERRIDE_VERSION_TAGGED", "a"),
            ("OVERRIDE_VERSION_COMMIT", "b"),
            ("OVERRIDE_VERSION_DOCKER_CI", "c"),
            ("PWD", &repo.repo.path().display().to_string()),
        ]
        .into_iter()
        .map(|(a, b)| (String::from(a), String::from(b))),
    );
    info.is_tag = Some(true);
    info.eval()?;
    assert_eq!(
        info.pwd_basename,
        repo.repo.path().file_name().unwrap().display().to_string()
    );
    assert_eq!(info.name, info.pwd_basename);
    assert_eq!(info.override_version_tagged, Some(String::from("a")));
    assert_eq!(info.override_version_commit, Some(String::from("b")));
    assert_eq!(info.override_version_docker_ci, Some(String::from("c")));
    assert_eq!(info.version_tagged, Some(String::from("a")));
    assert_eq!(info.version_commit, Some(String::from("b")));
    assert_eq!(info.version_docker_ci, String::from("c"));
    // Check new tag, on HEAD
    repo.run(&["git", "tag", "7.5"])?;
    repo.file_write("baz.txt", "Hello again again!")?;
    repo.run(&["git", "add", "baz.txt"])?;
    repo.run(&["git", "commit", "-m", "third commit"])?;
    let commit3 = git::head_commit(&repo.repo)?;
    let info = repo.info_get()?;
    assert_eq!(info.commit, commit3.as_str());
    assert_eq!(info.tag_latest, "7.5");
    assert_eq!(info.tag_latest_ltrimv, Some("7.5".to_string()));
    assert_eq!(info.distance, Some("1".to_string()));
    assert_eq!(info.version_docker_ci, "null");
    ghaction_version_gen::process_repo(Some(repo.repo.as_ref()))?;
    Ok(())
}

#[test]
fn gitrepo_tag_before_branch() -> Result<()> {
    environ_reset();
    let repo = TmpGit::new()?;
    repo.file_write("foo.txt", "Hello, world!")?;
    repo.run(&["git", "add", "foo.txt"])?;
    repo.run(&["git", "commit", "-m", "first commit"])?;
    repo.run(&["git", "tag", "v1.0.0"])?;
    repo.run(&["git", "checkout", "-b", "devel"])?;
    repo.file_write("bar.txt", "Hello, world!")?;
    repo.run(&["git", "add", "bar.txt"])?;
    repo.run(&["git", "commit", "-m", "second commit"])?;
    repo.run(&["git", "tag", "v1.1.0"])?;
    let mut info = repo.info_get()?;
    info.is_push = Some(true);
    info.is_tag = Some(true);
    info.eval()?;
    assert_eq!(info.tag_distance, Some("v1.1.0-0".to_string()));
    assert_eq!(info.tag_distance_ltrimv, Some("1.1.0-0".to_string()));
    assert_eq!(info.version_tagged.unwrap(), "1.1.0");
    assert_eq!(info.version_commit.unwrap(), "1.1.0");
    // Bring main after the tag:
    repo.run(&["git", "branch", "-f", "main", "HEAD"])?;
    repo.run(&["git", "checkout", "main"])?;
    let mut info = repo.info_get()?;
    info.is_push = Some(true);
    info.is_main = Some(true);
    info.eval()?;
    assert_eq!(info.tag_distance, Some("v1.1.0-0".to_string()));
    assert_eq!(info.tag_distance_ltrimv, Some("1.1.0-0".to_string()));
    assert_eq!(info.version_tagged, None);
    assert_eq!(info.version_commit, None);
    assert_eq!(info.rpm_basename, format!("-1.1.0-0-{}", info.commit));
    assert_eq!(info.deb_basename, format!("_1.1.0-0-{}", info.commit));
    Ok(())
}

#[test]
fn gitrepo_no_tag() -> Result<()> {
    environ_reset();
    let repo = TmpGit::new()?;
    repo.file_write("foo.txt", "Hello, world!")?;
    repo.run(&["git", "add", "foo.txt"])?;
    repo.run(&["git", "commit", "-m", "first commit"])?;
    let mut info = repo.info_get()?;
    info.is_push = Some(true);
    info.is_tag = Some(false);
    info.is_main = Some(true);
    info.eval()?;
    Ok(())
}

#[test]
fn gitrepo_rust() -> Result<()> {
    environ_reset();
    let repo = TmpGit::new()?;
    repo.file_write(
        "Cargo.toml",
        "[package]\nname = \"test\"\nversion = \"9.7\"\n",
    )?;
    repo.run(&["git", "add", "Cargo.toml"])?;
    repo.run(&["git", "commit", "-m", "first commit"])?;
    repo.run(&["git", "tag", "v1.0.0"])?;
    let mut info = repo.info_get()?;
    info.parse_files(&repo.repo)?;
    info.is_push = Some(true);
    info.is_tag = Some(true);
    info.is_main = Some(true);
    info.eval()?;
    assert_eq!(info.name, "test");
    assert_eq!(info.rust_crate_version, Some("9.7".to_string()));
    assert_eq!(
        info.version_mismatch,
        Some("file=Cargo.toml::Version mismatch: tag 1.0.0 != 9.7 from Cargo.toml".to_string())
    );
    assert_eq!(info.version_commit, Some("1.0.0".to_string()));
    assert_eq!(info.rpm_basename, "test-1.0.0");
    assert_eq!(info.deb_basename, "test_1.0.0");
    ghaction_version_gen::process_repo(Some(repo.repo.as_ref()))?;
    Ok(())
}

#[test]
fn gitrepo_no_tag_rust() -> Result<()> {
    environ_reset();
    let repo = TmpGit::new()?;
    repo.file_write(
        "Cargo.toml",
        "[package]\nname = \"test\"\nversion = \"9.7\"\n",
    )?;
    repo.run(&["git", "add", "Cargo.toml"])?;
    repo.run(&["git", "commit", "-m", "first commit"])?;
    let mut info = repo.info_get()?;
    info.is_push = Some(true);
    info.is_tag = Some(false);
    info.is_main = Some(true);
    info.eval()?;
    assert_eq!(info.rust_crate_version, Some("9.7".to_string()));
    assert_eq!(info.version_mismatch, None);
    assert_eq!(info.dash_distance, None);
    assert_eq!(info.tag_distance, None);
    assert_eq!(info.tag_latest_ltrimv, None);
    assert_eq!(info.tag_distance_ltrimv, None);
    assert_eq!(info.rpm_basename, "test");
    assert_eq!(info.deb_basename, "test");
    Ok(())
}

#[test]
fn gitrepo_after_tag_rust() -> Result<()> {
    environ_reset();
    let repo = TmpGit::new()?;
    repo.file_write(
        "Cargo.toml",
        "[package]\nname = \"test\"\nversion = \"9.7\"\n",
    )?;
    repo.run(&["git", "add", "Cargo.toml"])?;
    repo.run(&["git", "commit", "-m", "first commit"])?;
    repo.run(&["git", "tag", "v1.0.0"])?;
    repo.file_write("new", "\n")?;
    repo.run(&["git", "add", "new"])?;
    repo.run(&["git", "commit", "-m", "second commit"])?;
    let mut info = repo.info_get()?;
    info.parse_files(&repo.repo)?;
    info.is_push = Some(true);
    info.is_tag = Some(false);
    info.is_main = Some(false);
    info.eval()?;
    assert_eq!(info.name, "test");
    assert_eq!(info.rust_crate_version, Some("9.7".to_string()));
    assert_eq!(info.version_mismatch, None);
    assert_eq!(info.version_commit, None);
    assert_eq!(info.rpm_basename, format!("test-1.0.0-1-{}", info.commit));
    assert_eq!(info.deb_basename, format!("test_1.0.0-1-{}", info.commit));
    ghaction_version_gen::process_repo(Some(repo.repo.as_ref()))?;
    Ok(())
}

#[test]
fn gitrepo_after_tag_rust_main() -> Result<()> {
    environ_reset();
    let repo = TmpGit::new()?;
    repo.file_write(
        "Cargo.toml",
        "[package]\nname = \"test\"\nversion = \"9.7\"\n",
    )?;
    repo.run(&["git", "add", "Cargo.toml"])?;
    repo.run(&["git", "commit", "-m", "first commit"])?;
    repo.run(&["git", "tag", "v1.0.0"])?;
    repo.file_write("new", "\n")?;
    repo.run(&["git", "add", "new"])?;
    repo.run(&["git", "commit", "-m", "second commit"])?;
    let mut info = repo.info_get()?;
    info.parse_files(&repo.repo)?;
    info.is_push = Some(true);
    info.is_tag = Some(false);
    info.is_main = Some(true);
    info.eval()?;
    assert_eq!(info.name, "test");
    assert_eq!(info.rust_crate_version, Some("9.7".to_string()));
    assert_eq!(
        info.version_mismatch,
        Some("file=Cargo.toml::Version mismatch: tag 1.0.0 != 9.7 from Cargo.toml".to_string())
    );
    assert_eq!(info.version_commit, Some("1.0.0-1".into()));
    assert_eq!(info.rpm_basename, format!("test-1.0.0-1"));
    assert_eq!(info.deb_basename, format!("test_1.0.0-1"));
    ghaction_version_gen::process_repo(Some(repo.repo.as_ref()))?;
    Ok(())
}

#[test]
fn toml1() -> Result<()> {
    environ_reset();
    let repo = TmpGit::new()?;
    assert_eq!(rust::crate_data(&repo.repo)?, None);
    repo.file_write("Cargo.toml", "")?;
    assert!(rust::crate_data(&repo.repo).is_err());
    repo.file_write("Cargo.toml", "[package]\n")?;
    assert!(rust::crate_data(&repo.repo).is_err());
    repo.file_write(
        "Cargo.toml",
        "[package]\nname = \"test\"\nversion = \"1.0\"\n",
    )?;
    let data = rust::crate_data(&repo.repo)?;
    assert!(data.is_some());
    let data = data.unwrap();
    assert_eq!(data.name, "test".to_string());
    assert_eq!(data.version, "1.0".to_string());
    repo.file_write("Cargo.toml", "[workspace]\nmembers = [ \"abc\" ]\n")?;
    assert!(rust::crate_data(&repo.repo)?.is_none());
    Ok(())
}

#[test]
fn gitrepo_python() -> Result<()> {
    environ_reset();
    let repo = TmpGit::new()?;
    repo.file_write(
        "setup.cfg",
        "[metadata]
name = myname
version = 9.7

[options]
package_dir =
    = src
",
    )?;
    repo.run(&["git", "add", "setup.cfg"])?;
    repo.run(&["git", "commit", "-m", "first commit"])?;
    repo.run(&["git", "tag", "v1.0.0"])?;
    let mut info = repo.info_get()?;
    info.parse_files(&repo.repo)?;
    info.is_push = Some(true);
    info.is_tag = Some(true);
    info.is_main = Some(true);
    info.eval()?;
    assert_eq!(info.python_module_name, Some("myname".to_string()));
    assert_eq!(info.python_module_version, Some("9.7".to_string()));
    assert_eq!(
        info.version_mismatch,
        Some("file=setup.cfg::Version mismatch: tag 1.0.0 != 9.7 from setup.cfg".to_string())
    );
    ghaction_version_gen::process_repo(Some(repo.repo.as_ref()))?;
    Ok(())
}

#[test]
fn setupcfg() -> Result<()> {
    environ_reset();
    let repo = TmpGit::new()?;
    assert_eq!(python::module_data(&repo.repo)?, None);
    repo.file_write("setup.cfg", "")?;
    assert!(python::module_data(&repo.repo).is_err());
    repo.file_write("setup.cfg", "[metadata]\n")?;
    assert!(python::module_data(&repo.repo).is_err());
    repo.file_write("setup.cfg", "[metadata]\nname = abcde\n")?;
    assert!(python::module_data(&repo.repo).is_err());
    repo.file_write("setup.cfg", "[metadata]\nversion = 1.0\n")?;
    repo.file_write("setup.cfg", "[metadata]\nname = abcde\n")?;
    repo.file_write("setup.cfg", "[metadata]\nname = abcde\nversion = 1.0\n")?;
    assert_eq!(
        python::module_data(&repo.repo)?,
        Some(python::Data {
            name: "abcde".to_string(),
            version: "1.0".to_string()
        })
    );
    Ok(())
}
