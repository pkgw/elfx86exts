# elfx86exts

Disassemble a binary containing x86 instructions and print out which
extensions it uses. Despite the utterly misleading name, **this tool supports
ELF and MachO binaries**, and perhaps PE-format ones as well. (It used to be
more limited.)

I have no idea what I'm doing here, but it seems to work. There are several
Rust crates that make this pretty easy to do.

## Installation

### Prepackaged

This tool is installable through a few package managers:

- [Arch Linux AUR](https://aur.archlinux.org/packages/elfx86exts/)
- [conda-forge](https://anaconda.org/conda-forge/elfx86exts) (Linux only right now)

If you are interested in packaging `elfx86exts` in a new packaging system, or
have already done so, please submit a PR to add it to this list.

### Compiling the Latest Release

If a package is not available, in most cases it will be straightforward to
build `elfx86exts` yourself. Dependencies are:

- A [Rust](https://www.rust-lang.org/) toolchain
- The [Capstone](http://www.capstone-engine.org/) disassembly engine

Both of these dependencies are available through a wide variety of package
managers. Once they’re set up, you don’t even need to check out this
repository to install the latest release. Simply run:

```
cargo install elfx86exts
```

… and the tool will be installed in your Cargo binary directory, usually
`~/.cargo/bin/`. When using this method, you need to add the `--force` flag to
upgrade from one version to the next.

### Compiling the Code From Git

This is hardly any more difficult than the above. Check out this repository,
then run:

```
cargo install --path .
```

To develop the program, use the `cargo build` and `cargo run` commands. For
more information, see
[The Cargo Book](https://doc.rust-lang.org/cargo/index.html).


## Contributions

Contributions are welcome! Please submit PRs against this repository, or file
issues for discussion. The only important rule is that all participants are
expected to abide by the spirit of a standard
[Contributor Covenant code of conduct](https://www.contributor-covenant.org/).
All contributions will be assumed to be licensed under the terms described
below unless you explicitly state otherwise.


## Licensing

Licensed under the [MIT License](https://opensource.org/licenses/MIT).
