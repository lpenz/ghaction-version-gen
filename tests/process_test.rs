// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use anyhow::ensure;
use anyhow::Result;
use serial_test::serial;

use ::ghaction_version_gen::git;
use ::ghaction_version_gen::rust;
use ::ghaction_version_gen::Info;

#[test]
#[serial]
fn basic() -> Result<()> {
    let gitdesc = "1.3.1-20-gc5f7a99";
    let mut info = Info::default();
    info.parse_describe(gitdesc)?;
    assert_eq!(info.git_describe_tags, gitdesc);
    assert_eq!(info.tag_latest, "1.3.1");
    Ok(())
}

fn run(cmd: &[&str]) -> Result<()> {
    let status = Command::new(cmd[0]).args(&cmd[1..]).status()?;
    ensure!(status.success(), "error running command");
    Ok(())
}

fn file_write(filename: &str, contents: &str) -> Result<()> {
    let mut fd = File::create(filename)?;
    fd.write_all(contents.as_bytes())?;
    Ok(())
}

fn info_get() -> Result<Info> {
    let mut info = Info::from_workspace()?;
    info.is_push = None;
    info.is_tag = None;
    info.is_main = None;
    info.eval()?;
    Ok(info)
}

#[test]
#[serial]
fn gitrepo() -> Result<()> {
    let tmpdir = tempfile::tempdir().unwrap();
    env::set_current_dir(&tmpdir).unwrap();
    run(&["git", "init"])?;
    run(&["git", "config", "user.name", "username"])?;
    run(&["git", "config", "user.email", "user@email.net"])?;
    file_write("foo.txt", "Hello, world!")?;
    run(&["git", "add", "foo.txt"])?;
    run(&["git", "commit", "-m", "first commit"])?;
    // Check with no tag is present
    let commit1 = git::head_commit()?;
    let info = info_get()?;
    assert_eq!(info.commit, commit1.as_str());
    assert_eq!(info.tag_latest, "");
    assert_eq!(info.tag_head, None);
    assert_eq!(info.distance, "");
    assert_eq!(info.version_docker_ci, "null");
    // Check tag on HEAD
    run(&["git", "tag", "v1.0.0"])?;
    let mut info = info_get()?;
    info.is_push = Some(true);
    info.is_tag = Some(true);
    info.is_main = Some(true);
    info.eval()?;
    assert_eq!(info.commit, commit1.as_str());
    assert_eq!(info.git_describe_tags, "v1.0.0");
    assert_eq!(info.tag_latest, "v1.0.0");
    assert_eq!(info.distance, "0");
    assert_eq!(info.dash_distance, "-0");
    assert_eq!(info.tag_distance, "v1.0.0-0");
    assert_eq!(info.tag_head, Some("v1.0.0".to_string()));
    assert_eq!(info.tag_latest_ltrimv, "1.0.0");
    assert_eq!(info.tag_distance_ltrimv, "1.0.0-0");
    assert_eq!(info.tag_head_ltrimv, Some("1.0.0".to_string()));
    assert_eq!(info.version_tagged, Some("1.0.0".to_string()));
    assert_eq!(info.version_commit, Some("1.0.0".to_string()));
    assert_eq!(info.version_docker_ci, "1.0.0");
    // Check tag behind HEAD
    file_write("bar.txt", "Hello again!")?;
    run(&["git", "add", "bar.txt"])?;
    run(&["git", "commit", "-m", "second commit"])?;
    let commit2 = git::head_commit()?;
    let mut info = info_get()?;
    info.is_push = Some(true);
    info.is_tag = Some(false);
    info.is_main = Some(true);
    file_write("Cargo.toml", "[package]\nversion = \"9.7\"\n")?;
    info.parse_files()?;
    info.eval()?;
    assert_eq!(info.commit, commit2.as_str());
    assert_eq!(info.git_describe_tags, format!("v1.0.0-1-g{}", commit2));
    assert_eq!(info.tag_latest, "v1.0.0");
    assert_eq!(info.distance, "1");
    assert_eq!(info.dash_distance, "-1");
    assert_eq!(info.tag_distance, "v1.0.0-1");
    assert_eq!(info.tag_head, None);
    assert_eq!(info.tag_latest_ltrimv, "1.0.0");
    assert_eq!(info.tag_distance_ltrimv, "1.0.0-1");
    assert_eq!(info.tag_head_ltrimv, None);
    assert_eq!(info.version_tagged, None);
    assert_eq!(info.version_commit, Some("1.0.0-1".to_string()));
    assert_eq!(info.version_docker_ci, "latest");
    assert_eq!(info.rust_crate_version, Some("9.7".to_string()));
    assert_eq!(
        info.version_mismatch,
        Some("file=Cargo.toml::Version mismatch: tag 1.0.0 != 9.7 from Cargo.toml".to_string())
    );
    fs::remove_file("Cargo.toml")?;
    // Check new tag, on HEAD
    run(&["git", "tag", "7.5"])?;
    file_write("baz.txt", "Hello again again!")?;
    run(&["git", "add", "baz.txt"])?;
    run(&["git", "commit", "-m", "third commit"])?;
    let commit3 = git::head_commit()?;
    let info = info_get()?;
    assert_eq!(info.commit, commit3.as_str());
    assert_eq!(info.tag_latest, "7.5");
    assert_eq!(info.tag_latest_ltrimv, "7.5");
    assert_eq!(info.distance, "1");
    assert_eq!(info.version_docker_ci, "null");
    ghaction_version_gen::main()?;
    Ok(())
}

#[test]
#[serial]
fn toml1() -> Result<()> {
    let tmpdir = tempfile::tempdir()?;
    env::set_current_dir(&tmpdir)?;
    assert_eq!(rust::crate_version()?, None);
    file_write("Cargo.toml", "")?;
    assert!(rust::crate_version().is_err());
    file_write("Cargo.toml", "[package]\n")?;
    assert!(rust::crate_version().is_err());
    file_write("Cargo.toml", "[package]\nversion = \"1.0\"\n")?;
    assert_eq!(rust::crate_version()?, Some("1.0".to_string()));
    Ok(())
}
