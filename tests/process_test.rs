// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use anyhow::ensure;
use anyhow::Result;

use ::githeadinfo::git;
use ::githeadinfo::Info;

#[test]
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

#[test]
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
    let info = Info::from_workspace()?;
    assert_eq!(info.commit, commit1.as_str());
    assert_eq!(info.tag_latest, "");
    assert_eq!(info.tag_head, None);
    assert_eq!(info.distance, "");
    // Check tag on HEAD
    run(&["git", "tag", "v1.0.0"])?;
    let info = Info::from_workspace()?;
    assert_eq!(info.tag_latest, "v1.0.0");
    assert_eq!(info.tag_latest_ltrimv, "1.0.0");
    assert_eq!(info.tag_head, Some("v1.0.0".to_string()));
    assert_eq!(info.tag_head_ltrimv, Some("1.0.0".to_string()));
    assert_eq!(info.distance, "0");
    assert_eq!(info.dash_distance, "-0");
    assert_eq!(info.commit, commit1.as_str());
    // Check tag behind HEAD
    file_write("bar.txt", "Hello again!")?;
    run(&["git", "add", "bar.txt"])?;
    run(&["git", "commit", "-m", "second commit"])?;
    let commit2 = git::head_commit()?;
    let info = Info::from_workspace()?;
    assert_eq!(info.tag_latest, "v1.0.0");
    assert_eq!(info.tag_head, None);
    assert_eq!(info.distance, "1");
    assert_eq!(info.dash_distance, "-1");
    assert_eq!(info.commit, commit2.as_str());
    // Check new tag, on HEAD
    run(&["git", "tag", "7.5"])?;
    file_write("baz.txt", "Hello again again!")?;
    run(&["git", "add", "baz.txt"])?;
    run(&["git", "commit", "-m", "third commit"])?;
    let commit3 = git::head_commit()?;
    let info = Info::from_workspace()?;
    assert_eq!(info.tag_latest, "7.5");
    assert_eq!(info.tag_latest_ltrimv, "7.5");
    assert_eq!(info.distance, "1");
    assert_eq!(info.commit, commit3.as_str());
    githeadinfo::main()?;
    Ok(())
}
