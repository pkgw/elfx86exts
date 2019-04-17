// Copyright 2019 Peter Williams
// Licensed under the MIT License.

//! Test the CLI command.

extern crate assert_cmd;
extern crate escargot;

use assert_cmd::prelude::*;
use std::process::Command;

/// Test that the command runs successfully on itself. In principle I think
/// this might be somewhat limiting: there's no reason you couldn't compile
/// this tool on a platform whose executable format is not ELF or Mach-O, and
/// in that case this test would fail. Somehow I'm not very worried about that
/// possibility, though.
#[test]
fn run_on_self() {
    let cmd_run = escargot::CargoBuild::new()
        .bin("elfx86exts")
        .current_release()
        .current_target()
        .run()
        .unwrap();
    let cmd_path = cmd_run.path();

    let mut cmd = Command::new(cmd_path);
    cmd.arg(cmd_path);

    cmd.assert().success();
}
