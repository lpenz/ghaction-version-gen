// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::anyhow;
use anyhow::Result;
use toml::Value;

pub fn crate_version<P: AsRef<Path>>(repo: P) -> Result<Option<String>> {
    let cargofile = repo.as_ref().join("Cargo.toml");
    let result = fs::read_to_string(cargofile);
    if let Err(e) = result {
        return if e.kind() == ErrorKind::NotFound {
            Ok(None)
        } else {
            Err(e.into())
        };
    }
    let contents = result.unwrap();
    let info = contents.parse::<Value>()?;
    let package = &info
        .get("package")
        .ok_or_else(|| anyhow!("could not find package section"))?;
    let version_value = &package
        .get("version")
        .ok_or_else(|| anyhow!("could not find version in package section"))?;
    let version_str = version_value
        .as_str()
        .ok_or_else(|| anyhow!("could not find convert version to string"))?;
    Ok(Some(version_str.to_string()))
}
