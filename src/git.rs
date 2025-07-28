// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::path::Path;
use std::process::Command;

use color_eyre::Result;
use color_eyre::eyre::Report;
use color_eyre::eyre::ensure;

pub fn run<P: AsRef<Path>>(repo: P, args: &[&str]) -> Result<String> {
    let result = Command::new("git")
        .current_dir(repo.as_ref())
        .args(args)
        .output()?;
    ensure!(
        result.status.success(),
        "error running git {:?}: {:?}; in {:?}",
        args,
        result,
        repo.as_ref().display(),
    );
    String::from_utf8(result.stdout)
        .map(|s| s.trim().to_string())
        .map_err(Report::from)
}

pub fn describe<P: AsRef<Path>>(repo: P) -> Result<String> {
    run(repo, &["describe", "--tags"])
}

pub fn ref_commit<P: AsRef<Path>>(repo: P, reference: &str) -> Result<String> {
    run(repo, &["rev-parse", "--short", reference])
}

pub fn head_commit<P: AsRef<Path>>(repo: P) -> Result<String> {
    ref_commit(repo, "HEAD")
}

pub fn unshallow<P: AsRef<Path>>(repo: P) -> Result<String> {
    run(repo, &["fetch", "--unshallow", "origin"])
}
