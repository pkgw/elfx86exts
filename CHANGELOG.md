# elfx86exts 0.6.2 (2023-10-22)

- Fix excessive memory consumption (@HanabishiRecca, #2, #90, #173). For a long
  time, we have known that this program can consume surprisingly large amounts
  of memory when processing large programs (well, and small ones too). It turns
  out that if we simply disassemble one instruction at a time instead of in
  bulk, not only do the memory requirements go way down, but the program is much
  faster too! Thanks again to @HanabishiRecca for finally solving this
  longstanding issue.


# elfx86exts 0.6.1 (2023-10-21)

- Ignore invalid instructions when parsing the executable (@HanabishiRecca,
  #171, #172). This allows `elfx86exts` to handle some executables which have
  regions of "junk" data between functions, such as ones created by [mold].
  Thank you to @HanabishiRecca for reporting, investigating, and solving this
  issue!

[mold]: https://github.com/rui314/mold


# elfx86exts 0.6.0 (2023-10-12)

In the ongoing effort to make the name of this tool ever more inaccurate,
@jasonmccampbell has added preliminary support for ARM64 (aarch64) binaries in
#166! The program will now automatically detect the CPU architecture of the
binary under analysis, and choose the X86 or ARM64 analysis path depending on
what it sees. (Or if the binary is for some other architecture, it will bail
out.)

Since the last release, @dargor also added some names for AMD CPU generations
and tidied up some of the related information (#88).

There are also some general updates to the dependencies from @pkgw.


# elfx86exts 0.5.0 (2021-10-18)

This release updates this tool to do its disassembly using [capstone 0.10][cs]
and [object 0.27][obj]. In at least some cases, the previous release based on
capstone 0.7 was giving seriously incorrect output (see [#66], filed by
[@rowanworth]).

[cs]: https://github.com/capstone-rust/capstone-rs
[obj]: https://github.com/gimli-rs/object
[#66]: https://github.com/pkgw/elfx86exts/issues/66
[@rowanworth]: https://github.com/rowanworth


# elfx86exts 0.4.3 (2020-09-01)

- Bump to Rust 2018 edition
- ci: use cranko's new binary-packaging helper and publish Rust packages to
  crates.io upon deployment
- Numerous dependency updates since last release on crates.io


# elfx86exts 0.4.2 (2020-08-28)

- Add a Windows build ... mainly to demonstrate Cranko's usage in a build
  pipeline that includes Windows builds.


# elfx86exts 0.4.1 (2020-08-28)

- Migrate to Cranko for versioning and release workflow.


# 0.4.0 (2020 May 11)

- Update dependencies, including to
  [capstone](https://crates.io/crates/capstone) 0.7.0.
- Fix a missing space in the `--help` output (@bjmoran,
  [#29](https://github.com/pkgw/elfx86exts/pull/29))


# 0.3.0 (2018 Oct 16)

- The tool will now print out its best guess as to the minimum Intel CPU
  generation needed to run the analyzed binary, based on an internal table of
  which extensions were introduced in which model. This is a bit approximate
  because the evolution is not strictly linear. Thanks to
  [@apjanke](https://github.com/apjanke) for
  [the contribution](https://github.com/pkgw/elfx86exts/pull/10)! Bumping the
  minor version because this alters the output format.
- Update dependencies, including to
  [capstone](https://crates.io/crates/capstone) 0.5.0.


# 0.2.0 (2018 Jan 23)

- Support MachO/PE binaries as well! Now the name of this tool is super
  misleading! Oh well, it was worth it. Thanks to
  [@reuben](https://github.com/reuben) for
  [the contribution](https://github.com/pkgw/elfx86exts/pull/1)!


# 0.1.0 (2017 Sep 29)

- Initial release.
