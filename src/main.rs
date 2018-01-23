// Copyright 2017-2018 Peter Williams
// Licensed under the MIT License.

/// elfx86exts helps you understand which instruction set extensions are used
/// by an x86 binary. Despite the misleading name, this crate supports both
/// ELF and MachO binary formats via the
/// [capstone](https://crates.io/crates/capstone) crate.

extern crate capstone;
#[macro_use] extern crate clap;
extern crate memmap;
extern crate object;

use std::fs::File;
use std::collections::HashSet;
use std::path::PathBuf;
use capstone::{Arch, Capstone, NO_EXTRA_MODE, Mode};
use object::{Machine, Object, ObjectSection, SectionKind};


/// These are from capstone/include/x86.h, which is super sketchy since the
/// enum values are not specificied explicitly.
fn describe_group(g: u8) -> Option<&'static str> {
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
        .version(crate_version!())
        .about("Analyze a binary to understand which instruction set extensions it uses.")
        .after_help("Despite the misleading name, this program can handle binaries in both\
                     ELF and MachO formats, and possibly others.")
        .arg(clap::Arg::with_name("FILE")
             .help("The path of the file to analyze")
             .required(true)
             .index(1))
        .get_matches();

    let inpath = PathBuf::from(matches.value_of_os("FILE").unwrap());
    let f = File::open(inpath).expect("can't open object file");
    let buf = unsafe { memmap::Mmap::map(&f).expect("can't memmap object file") };

    let obj = object::File::parse(&*buf).expect("can't parse object file");

    let (arch, mode) = match obj.machine() {
        Machine::X86 => (Arch::X86, Mode::Mode32),
        Machine::X86_64 => (Arch::X86, Mode::Mode64),
        _ => unimplemented!(),
    };

    let mut cs = Capstone::new_raw(arch, mode, NO_EXTRA_MODE, None).expect("can't initialize capstone");
    cs.set_detail(true).expect("can't enable Capstone detail mode");

    let mut seen_groups = HashSet::new();

    for sect in obj.sections() {
        if sect.kind() != SectionKind::Text {
            continue;
        }

        let insns = cs.disasm_all(sect.data(), sect.address()).expect("couldn't disassemble section");

        for insn in insns.iter() {
            for group_code in cs.insn_group_ids(&insn).unwrap() {
                if seen_groups.insert(*group_code) {
                    // If insert returned true, we hadn't seen this code before.
                    if let Some(desc) = describe_group(*group_code) {
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
