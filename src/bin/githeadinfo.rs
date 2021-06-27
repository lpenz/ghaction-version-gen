// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;

#[derive(structopt::StructOpt)]
pub struct Args {}

#[paw::main]
fn main(_: Args) -> Result<()> {
    githeadinfo::main()
}
