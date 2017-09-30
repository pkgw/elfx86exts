// Copyright 2017 Peter Williams
// Licensed under the MIT License.

/// elfx86exts helps you understand which instruction set extensions are used
/// by an x86 ELF binary.

extern crate capstone3;
extern crate clap;
extern crate libc;
extern crate xmas_elf;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use xmas_elf::ElfFile;
use xmas_elf::sections::ShType;


/// Helper function to open a file and read it into a buffer.
/// Allocates the buffer.
///
/// Copied from xmas-elf/src/bin/main.rs
fn open_file<P: AsRef<Path>>(name: P) -> Vec<u8> {
    use std::fs::File;
    use std::io::Read;

    let mut f = File::open(name).expect("couldn't open file");
    let mut buf = Vec::new();
    assert!(f.read_to_end(&mut buf).expect("couldn't read file") > 0);
    buf
}


/// These are from capstone/include/x86.h, which is super sketchy since the
/// enum values are not specificied explicitly.
fn describe_group(g: libc::uint8_t) -> Option<&'static str> {
    Some(match g {
        128 => "VT-x/AMD-V", // https://github.com/aquynh/capstone/blob/master/include/x86.h#L1583
        129 => "3DNow",
        130 => "AES",
        131 => "ADX",
        132 => "AVX",
        133 => "AVX2",
        134 => "AVX512",
        135 => "BMI",
        136 => "BMI2",
        137 => "CMOV",
        138 => "F16C", // line 1593
        139 => "FMA",
        140 => "FMA4",
        141 => "FSGSBASE",
        142 => "HLE",
        143 => "MMX",
        144 => "MODE32",
        145 => "MODE64",
        146 => "RTM",
        147 => "SHA",
        148 => "SSE1", // line 1603
        149 => "SSE2",
        150 => "SSE3",
        151 => "SSE41",
        152 => "SSE42",
        153 => "SSE4A",
        154 => "SSSE3",
        155 => "PCLMUL",
        156 => "XOP",
        157 => "CDI",
        158 => "ERI", // line 1613
        159 => "TBM",
        160 => "16BITMODE",
        161 => "NOT64BITMODE",
        162 => "SGX",
        163 => "DQI",
        164 => "BWI",
        165 => "PFI",
        166 => "VLX",
        167 => "SMAP",
        168 => "NOVLX", // line 1623
        _ => { return None; },
    })
}


fn main() {
    let matches = clap::App::new("elfx86exts")
        .version("0.1.0")
        .about("Analyze an ELF/x86 binary to understand which instruction set extensions it uses.")
        .arg(clap::Arg::with_name("FILE")
             .help("The path of the file to analyze")
             .required(true)
             .index(1))
        .get_matches();

    let inpath = PathBuf::from(matches.value_of_os("FILE").unwrap());
    let contents = open_file(inpath);
    let elf_file = ElfFile::new(&contents).expect("couldn't parse as an ELF file");

    let cs = capstone3::Capstone::new(capstone3::Arch::X86).expect("couldn\'t set up Capstone");
    cs.detail().expect("couldn't turn on Capstone detail mode");

    let mut seen_groups = HashSet::new();

    let mut sect_iter = elf_file.section_iter();
    // Skip the first (dummy) section
    sect_iter.next();

    for sect in sect_iter {
        //let name = sect.get_name(&elf_file).expect("couldn\'t get a section name");
        match sect.get_type() {
            Ok(ShType::ProgBits) => {},
            _ => { continue; },
        };

        let insns = cs.disassemble(sect.raw_data(&elf_file), sect.offset()).expect("couldn't disassemble section");

        for insn in insns.iter() {
            let detail = match insn.detail() {
                Some(d) => d,
                None => { continue; },
            };

            let groups = detail.groups();

            for i in 0..detail.groups_count() {
                let group_code = groups[i as usize];

                if seen_groups.insert(group_code) {
                    // If insert returned true, we hadn't seen this code before.
                    if let Some(desc) = describe_group(group_code) {
                        if let Some(mnemonic) = insn.mnemonic() {
                            println!("{} ({})", desc, mnemonic);
                        } else {
                            println!("{}", desc);
                        }
                    }
                }
            }
        }
    }
}
