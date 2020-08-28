// Copyright 2020 Peter Williams <peter@newton.cx> and collaborators
// Licensed under the MIT License.

//! Build infrastructure.

use std::{
    fs::File,
    io::{Error, Write},
    path::Path,
};

fn main() -> Result<(), Error> {
    let outdir = std::env::var_os("OUT_DIR").unwrap();
    std::fs::create_dir_all(&outdir)?;
    let target_file_path = Path::new(&outdir).join("cargo-target.txt");
    let mut target_file = File::create(&target_file_path)?;
    writeln!(target_file, "{}", std::env::var("TARGET").unwrap())?;
    Ok(())
}
