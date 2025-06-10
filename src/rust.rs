// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::Result;
use anyhow::anyhow;
use toml::Value;

#[derive(Debug, PartialEq, Eq)]
pub struct Crate {
    pub name: String,
    pub version: String,
}

pub fn crate_data<P: AsRef<Path>>(repo: P) -> Result<Option<Crate>> {
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
    if info.get("workspace").is_some() {
        return Ok(None);
    }
    let package = &info
        .get("package")
        .ok_or_else(|| anyhow!("could not find package section"))?;
    let name = &package
        .get("name")
        .ok_or_else(|| anyhow!("could not find name in package section"))?;
    let version_value = &package
        .get("version")
        .ok_or_else(|| anyhow!("could not find version in package section"))?;
    let version_str = version_value
        .as_str()
        .ok_or_else(|| anyhow!("could not find convert version to string"))?;
    Ok(Some(Crate {
        name: name.to_string(),
        version: version_str.to_string(),
    }))
}
