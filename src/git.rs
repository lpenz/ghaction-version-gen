// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::process::Command;

use anyhow::ensure;
use anyhow::Result;

pub fn run(args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).output()?;
    ensure!(output.status.success(), "error running git {:?}", args);
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(anyhow::Error::msg)
}

pub fn describe() -> Result<String> {
    run(&["describe", "--tags"])
}

pub fn head_commit() -> Result<String> {
    run(&["rev-parse", "--short", "HEAD"])
}

pub fn unshallow() -> Result<String> {
    run(&["fetch", "--unshallow", "origin"])
}
