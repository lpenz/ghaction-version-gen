// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::path::Path;
use std::process::Command;

use anyhow::ensure;
use anyhow::Result;

pub fn run<P: AsRef<Path>>(repo: P, args: &[&str]) -> Result<String> {
    let output = Command::new("git").current_dir(repo).args(args).output()?;
    ensure!(output.status.success(), "error running git {:?}", args);
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(anyhow::Error::msg)
}

pub fn describe<P: AsRef<Path>>(repo: P) -> Result<String> {
    run(repo, &["describe", "--tags"])
}

pub fn head_commit<P: AsRef<Path>>(repo: P) -> Result<String> {
    run(repo, &["rev-parse", "--short", "HEAD"])
}

pub fn unshallow<P: AsRef<Path>>(repo: P) -> Result<String> {
    run(repo, &["fetch", "--unshallow", "origin"])
}
