# elfx86exts

Disassemble a binary containing x86 instructions and print out which
extensions it uses. Despite the utterly misleading name, **this tool supports
ELF and MachO binaries**, and perhaps PE-format ones as well. (It used to be
more limited.)

I have no idea what I'm doing here, but it seems to work. There are several
Rust crates that make this pretty easy to do.

# License

Licensed under the MIT License.

# Installation

To build and install system-wide:

```
make
make install
```

Or you can build using `cargo` directly:

```
cargo build
```
