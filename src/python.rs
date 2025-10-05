// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::path::Path;

use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::eyre;

use configparser::ini::Ini;

pub fn module_version<P: AsRef<Path>>(repo: P) -> Result<Option<String>> {
    let setupcfgfile = repo.as_ref().join("setup.cfg");
    let content = match std::fs::read_to_string(setupcfgfile) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(e) => {
            return Err(eyre!(e));
        }
    };
    // Pre-parse the file to get rid of empty keys which are not
    // supported by configparser:
    let setupcfg = content
        .lines()
        .map(|line| {
            if line.trim_start().starts_with('=') {
                // replace "= src" with "__empty__ = src"
                format!("__empty__{}", line)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut config = Ini::new();
    let result = config.read(setupcfg);
    if let Err(e) = result {
        // Could have used a proper error...
        if e.contains("No such file or directory") {
            return Ok(None);
        } else {
            return Err(eyre!("parsing setup.cfg: {}", e));
        }
    }
    let version = config
        .get("metadata", "version")
        .ok_or_eyre("could not find metadata.version")?;
    Ok(Some(version))
}
