// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::path::Path;

use anyhow::Result;
use anyhow::anyhow;
use configparser::ini::Ini;

pub fn module_version<P: AsRef<Path>>(repo: P) -> Result<Option<String>> {
    let setupcfgfile = repo.as_ref().join("setup.cfg");
    let mut config = Ini::new();
    let result = config.load(setupcfgfile);
    if let Err(e) = result {
        // Could have used a proper error...
        if e.contains("No such file or directory") {
            return Ok(None);
        } else {
            return Err(anyhow!("parsing setup.cfg: {}", e));
        }
    }
    let version = config
        .get("metadata", "version")
        .ok_or_else(|| anyhow!("could not find metadata.version"))?;
    Ok(Some(version))
}
