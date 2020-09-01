// Copyright 2017-2018 Peter Williams
// Licensed under the MIT License.

//! elfx86exts helps you understand which instruction set extensions are used
//! by an x86 binary. Despite the misleading name, this crate supports both
//! ELF and MachO binary formats via the
//! [capstone](https://crates.io/crates/capstone) crate.

use capstone::{Arch, Capstone, Mode, NO_EXTRA_MODE};
use clap::crate_version;
use object::{Object, ObjectSection, SectionKind};
use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;

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
        _ => {
            return None;
        }
    })
}

fn main() {
    // This list is taken from https://en.wikipedia.org/wiki/List_of_Intel_CPU_microarchitectures
    // The ID numbers here are internal elfx86exts identifiers; they have no meaning except
    // for their relative ordering.
    let cpu_generations: HashMap<&str, u16> = [
        ("Pentium", 100),
        ("Pentium Pro", 101),
        ("Pentium III", 102),
        ("Pentium 4", 103),
        ("Pentium M", 104),
        ("Prescott", 105),
        ("Intel Core", 106),
        ("Penryn", 107),
        ("Nehalem", 108),
        ("Bonnell", 109),
        ("Westmere", 110),
        ("Saltwell", 111),
        ("Sandy Bridge", 112),
        ("Ivy Bridge", 113),
        ("Silvermont", 114),
        ("Haswell", 115),
        ("Broadwell", 116),
        ("Airmont", 117),
        ("Skylake", 118),
        ("Goldmont", 119),
        ("Kaby Lake", 120),
        ("Coffee Lake", 121),
        ("Cannon Lake", 122),
        ("Whiskey Lake", 123),
        ("Amber Lake", 124),
        ("Cascade Lake", 125),
        ("Cooper Lake", 126),
        ("Ice Lake", 127),
        ("Unknown", 999),
    ]
    .iter()
    .cloned()
    .collect();
    let mut cpu_generations_reverse: HashMap<u16, &str> = HashMap::new();
    for (key, val) in &cpu_generations {
        cpu_generations_reverse.insert(*val, key);
    }
    // The Intel generation that introduced each instruction set
    // This list is based on Googling and Wikipedia reading
    // Many of these are approximations, since CPU development isn't strictly linear, and not
    // all models of a generation support a given instruction set.
    let instrset_to_cpu: HashMap<&str, &str> = [
        ("VT-x/AMD-V", "Intel Core"), // guess; https://en.wikipedia.org/wiki/X86_virtualization
        ("3DNow", "Unknown"), // Not supported by Intel CPUs; https://en.wikipedia.org/wiki/3DNow!
        ("AES", "Westmere"),  // https://en.wikipedia.org/wiki/AES_instruction_set
        ("ADX", "Broadwell"), // https://en.wikipedia.org/wiki/Intel_ADX
        ("AVX", "Sandy Bridge"), // https://en.wikipedia.org/wiki/Advanced_Vector_Extensions
        ("AVX2", "Haswell"),  // https://en.wikipedia.org/wiki/Advanced_Vector_Extensions
        ("AVX512", "Unknown"), // It's complicated. https://en.wikipedia.org/wiki/Advanced_Vector_Extensions
        ("BMI", "Haswell"),    // https://en.wikipedia.org/wiki/Bit_Manipulation_Instruction_Sets
        ("BMI2", "Haswell"),   // https://en.wikipedia.org/wiki/Bit_Manipulation_Instruction_Sets
        ("CMOV", "Pentium Pro"), // https://en.wikipedia.org/wiki/X86_instruction_listings
        ("F16C", "Ivy Bridge"), // https://en.wikipedia.org/wiki/F16C
        ("FMA", "Haswell"),    // https://en.wikipedia.org/wiki/FMA_instruction_set
        ("FMA4", "Unknown"), // Not supported by Intel? https://en.wikipedia.org/wiki/FMA_instruction_set
        ("FSGSBASE", "Unknown"), // ???
        ("HLE", "Haswell"), // Part of TSX - https://en.wikipedia.org/wiki/Transactional_Synchronization_Extensions
        ("MMX", "Pentium"), // https://en.wikipedia.org/wiki/MMX_(instruction_set)
        ("MODE32", "Pentium"), // Assuming all x86 CPUs support 32-bit mode
        ("MODE64", "Intel Core"), // I'm assuming this just means x86-64 support
        ("RTM", "Haswell"), // Part of TSX - https://en.wikipedia.org/wiki/Transactional_Synchronization_Extensions
        ("SHA", "Goldmont"), // https://en.wikipedia.org/wiki/Intel_SHA_extensions
        ("SSE1", "Pentium III"), // https://en.wikipedia.org/wiki/Streaming_SIMD_Extensions
        ("SSE2", "Pentium 4"), // https://en.wikipedia.org/wiki/Streaming_SIMD_Extensions
        ("SSE3", "Prescott"), // https://en.wikipedia.org/wiki/Streaming_SIMD_Extensions
        ("SSE41", "Penryn"), // https://en.wikipedia.org/wiki/SSE4
        ("SSE42", "Nehalem"), // https://en.wikipedia.org/wiki/SSE4
        ("SSE4A", "Unknown"), // AMD-only - https://en.wikipedia.org/wiki/SSE4
        ("SSSE3", "Unknown"), // Merom, but I don't know where that goes in the CPU list
        ("PCLMUL", "Intel Core"), // https://software.intel.com/en-us/articles/intel-carry-less-multiplication-instruction-and-its-usage-for-computing-the-gcm-mode/
        ("XOP", "Unknown"),       // AMD-only - https://en.wikipedia.org/wiki/XOP_instruction_set
        ("CDI", "Unknown"), // Knights Landing - https://software.intel.com/en-us/blogs/2013/avx-512-instructions
        ("ERI", "Unknown"), // Knights Landing - https://software.intel.com/en-us/blogs/2013/avx-512-instructions
        ("TBM", "Unknown"), // AMD-only - https://en.wikipedia.org/wiki/Bit_Manipulation_Instruction_Sets#TBM_(Trailing_Bit_Manipulation)
        ("16BITMODE", "Unknown"),
        ("NOT64BITMODE", "Unknown"),
        ("SGX", "Skylake"), // https://en.wikipedia.org/wiki/Software_Guard_Extensions
        ("DQI", "Unknown"), // Couldn't find a reference
        ("BWI", "Unknown"), // Looks like a Xeon-only Knights Landing+ extension? - https://reviews.llvm.org/D26306
        ("PFI", "Unknown"), // Knights Landing - https://software.intel.com/en-us/blogs/2013/avx-512-instructions
        ("VLX", "Unknown"), // Couldn't find a reference
        ("SMAP", "Broadwell"), // https://en.wikipedia.org/wiki/Supervisor_Mode_Access_Prevention
        ("NOVLX", "Unknown"), // Couldn't find a reference
    ]
    .iter()
    .cloned()
    .collect();

    let matches = clap::App::new("elfx86exts")
        .version(crate_version!())
        .about("Analyze a binary to understand which instruction set extensions it uses.")
        .after_help(
            "Despite the misleading name, this program can handle binaries in both \
             ELF and MachO formats, and possibly others.",
        )
        .arg(
            clap::Arg::with_name("FILE")
                .help("The path of the file to analyze")
                .required(true)
                .index(1),
        )
        .get_matches();

    let inpath = PathBuf::from(matches.value_of_os("FILE").unwrap());
    let f = File::open(inpath).expect("can't open object file");
    let buf = unsafe { memmap::Mmap::map(&f).expect("can't memmap object file") };

    let obj = object::File::parse(&*buf).expect("can't parse object file");

    let mode = if obj.is_64() {
        Mode::Mode64
    } else {
        Mode::Mode32
    };

    let mut cs =
        Capstone::new_raw(Arch::X86, mode, NO_EXTRA_MODE, None).expect("can't initialize capstone");
    cs.set_detail(true)
        .expect("can't enable Capstone detail mode");

    let mut seen_groups = HashSet::new();
    let mut max_gen_code = 100;

    for sect in obj.sections() {
        if sect.kind() != SectionKind::Text {
            continue;
        }

        let data = sect.data().expect("couldn't get section data");

        let insns = cs
            .disasm_all(&data, sect.address())
            .expect("couldn't disassemble section");

        for insn in insns.iter() {
            let detail = cs
                .insn_detail(&insn)
                .expect("couldn't get details of an instruction");

            for group_code in detail.groups() {
                if seen_groups.insert(group_code) {
                    // If insert returned true, we hadn't seen this code before.
                    if let Some(desc) = describe_group(group_code.0) {
                        if let Some(mnemonic) = insn.mnemonic() {
                            println!("{} ({})", desc, mnemonic);
                            match instrset_to_cpu.get(desc) {
                                Some(generation) => match cpu_generations.get(generation) {
                                    Some(gen_code) => {
                                        max_gen_code = cmp::max(max_gen_code, *gen_code);
                                    }
                                    None => unimplemented!(),
                                },
                                None => unimplemented!(),
                            }
                        } else {
                            println!("{}", desc);
                        }
                    }
                }
            }
        }
    }

    match cpu_generations_reverse.get(&max_gen_code) {
        Some(generation) => {
            println!("CPU Generation: {}", generation);
        }
        None => unimplemented!(),
    }
}
