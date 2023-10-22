// Copyright 2017-2023 elfx86exts contributors
// Licensed under the MIT License.

//! elfx86exts helps you understand which instruction set extensions are used
//! by a binary. Despite the misleading name, this crate supports both
//! ELF and MachO binary formats and x86 as well as Arm processor architectures via the
//! [capstone](https://crates.io/crates/capstone) crate.

use capstone::{Arch as CapArch, Capstone, Mode, NO_EXTRA_MODE};
use clap::Parser;
use object::{Architecture as ObjArch, Object, ObjectSection, SectionKind};
use std::{
    cmp,
    collections::{HashMap, HashSet},
    fs::File,
    path::PathBuf,
};

/// These are from capstone/include/x86.h, which is super sketchy since the
/// enum values are not specificied explicitly.
fn describe_group_x86(g: &u8) -> Option<&'static str> {
    Some(match *g {
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

/// These are from capstone/include/arm64.h
fn describe_group_aarch64(g: &u8) -> Option<&'static str> {
    Some(match *g {
        128 => "CRYPTO",  // https://github.com/aquynh/capstone/blob/master/include/aarch64.h
        129 => "FPARMV8", // Appears to map to both fp and fp16 instruction sets
        130 => "NEON",
        131 => "CRC",
        132 => "AES",
        133 => "DOTPROD",
        134 => "FULLFP16",
        135 => "LSE",
        136 => "RCPC",
        137 => "RDM",
        138 => "SHA2",
        139 => "SHA3",
        140 => "SM4",
        141 => "SVE",
        142 => "SVE2",
        143 => "SVE2AES",
        144 => "SVE2BitPerm",
        145 => "SVE2SHA3",
        146 => "SV#2SM4",
        147 => "SME",
        148 => "SMEF64",
        149 => "SMEI64",
        150 => "MatMulFP32",
        151 => "MatMulFP64",
        152 => "MatMulInt8",
        153 => "V8_1A",
        154 => "V8_3A",
        155 => "V8_4A",
        _ => {
            return None;
        }
    })
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path of the file to analyze
    path: PathBuf,
}

fn main() {
    // This list is taken from https://en.wikipedia.org/wiki/List_of_Intel_CPU_microarchitectures
    // The ID numbers here are internal elfx86exts identifiers; they have no meaning except
    // for their relative ordering.
    let cpu_generations: HashMap<&str, u16> = [
        // Intel: 1xx
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
        // AMD: 2xx
        ("K6-2", 200),
        ("Bulldozer", 201),
        ("K10", 202),
        ("Piledriver", 203),
        // Unknown
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
        ("3DNow", "K6-2"), // Not supported by Intel CPUs, nor AMD since 2010; https://en.wikipedia.org/wiki/3DNow!
        ("AES", "Westmere"), // https://en.wikipedia.org/wiki/AES_instruction_set
        ("ADX", "Broadwell"), // https://en.wikipedia.org/wiki/Intel_ADX
        ("AVX", "Sandy Bridge"), // https://en.wikipedia.org/wiki/Advanced_Vector_Extensions
        ("AVX2", "Haswell"), // https://en.wikipedia.org/wiki/Advanced_Vector_Extensions
        ("AVX512", "Unknown"), // It's complicated. https://en.wikipedia.org/wiki/Advanced_Vector_Extensions
        ("BMI", "Haswell"),    // https://en.wikipedia.org/wiki/Bit_Manipulation_Instruction_Sets
        ("BMI2", "Haswell"),   // https://en.wikipedia.org/wiki/Bit_Manipulation_Instruction_Sets
        ("CMOV", "Pentium Pro"), // https://en.wikipedia.org/wiki/X86_instruction_listings
        ("F16C", "Ivy Bridge"), // https://en.wikipedia.org/wiki/F16C
        ("FMA", "Haswell"),    // https://en.wikipedia.org/wiki/FMA_instruction_set
        ("FMA4", "Bulldozer"), // Not supported by Intel? https://en.wikipedia.org/wiki/FMA_instruction_set
        ("FSGSBASE", "Ivy Bridge"), // https://lwn.net/Articles/821723/
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
        ("SSE4A", "K10"),   // AMD-only - https://en.wikipedia.org/wiki/SSE4
        ("SSSE3", "Intel Core"), // https://en.wikipedia.org/wiki/Intel_Core_(microarchitecture)
        ("PCLMUL", "Intel Core"), // https://software.intel.com/en-us/articles/intel-carry-less-multiplication-instruction-and-its-usage-for-computing-the-gcm-mode/
        ("XOP", "Bulldozer"),     // AMD-only - https://en.wikipedia.org/wiki/XOP_instruction_set
        ("CDI", "Unknown"), // Knights Landing - https://software.intel.com/en-us/blogs/2013/avx-512-instructions
        ("ERI", "Unknown"), // Knights Landing - https://software.intel.com/en-us/blogs/2013/avx-512-instructions
        ("TBM", "Piledriver"), // AMD-only - https://en.wikipedia.org/wiki/Bit_Manipulation_Instruction_Sets#TBM_(Trailing_Bit_Manipulation)
        ("16BITMODE", "Unknown"),
        ("NOT64BITMODE", "Unknown"),
        ("SGX", "Skylake"), // https://en.wikipedia.org/wiki/Software_Guard_Extensions
        ("DQI", "Cannon Lake"), // AVX-512 Doubleword and Quadword Instructions
        ("BWI", "Cannon Lake"), // AVX-512 Byte and Word Instructions
        ("PFI", "Unknown"), // AVX-512 Prefetch Instructions, implemented by Knights Landing - https://software.intel.com/en-us/blogs/2013/avx-512-instructions
        ("VLX", "Cannon Lake"), // AVX-512 Vector Length Extensions
        ("SMAP", "Broadwell"), // https://en.wikipedia.org/wiki/Supervisor_Mode_Access_Prevention
        ("NOVLX", "Unknown"), // References in LLVM sources, associated mostly with AVX and AVX2 when VLX are not available
    ]
    .iter()
    .cloned()
    .collect();

    let args = Args::parse();

    // Read the file
    let f = File::open(&args.path).expect("can't open object file");
    let buf = unsafe { memmap::Mmap::map(&f).expect("can't memmap object file") };
    let obj = object::File::parse(&*buf).expect("can't parse object file");
    let obj_arch = obj.architecture();

    println!(
        "File format and CPU architecture: {:?}, {obj_arch:?}",
        obj.format()
    );

    // Figure out the capstone / analysis settings.
    //
    // Capstone apparently doesn't do any validation, so if we initialize it
    // with the wrong architecture, it will just plunge on ahead and parse
    // everything as best it can. So in principle instead of exiting early with
    // unrecognized arches, we could print a warning and YOLO it. Right now we
    // don't do that, but we do try to be generous about accepting any `object`
    // architecture that might be consistent with a supported `capstone`
    // architecture.
    let (cap_arch, mode, describe_group): (CapArch, Mode, fn(&u8) -> Option<&str>) = match obj_arch
    {
        ObjArch::X86_64 | ObjArch::X86_64_X32 => {
            if obj.is_64() {
                (CapArch::X86, Mode::Mode64, describe_group_x86)
            } else {
                (CapArch::X86, Mode::Mode32, describe_group_x86)
            }
        }

        ObjArch::Aarch64 | ObjArch::Aarch64_Ilp32 => {
            (CapArch::ARM64, Mode::Arm, describe_group_aarch64)
        }

        _ => {
            // This could plausibly be an error exit, but it doesn't seem
            // unreasonable to exit with success either.
            println!(
                "CPU architecture `{obj_arch:?}` of input file `{}` is not currently supported for analysis",
                args.path.display()
            );
            return;
        }
    };

    // Disassemble the file
    let mut cs =
        Capstone::new_raw(cap_arch, mode, NO_EXTRA_MODE, None).expect("can't initialize capstone");
    cs.set_detail(true)
        .expect("can't enable Capstone detail mode");
    cs.set_skipdata(true)
        .expect("can't enable Capstone skip data mode");

    let mut seen_groups = HashSet::new();

    for sect in obj.sections() {
        if sect.kind() != SectionKind::Text {
            continue;
        }

        let data = sect.data().expect("couldn't get section data");
        let mut offset = 0;

        loop {
            let rest = &data[offset..];

            if rest.is_empty() {
                break;
            }

            let insns = cs
                .disasm_count(rest, 0, 1)
                .expect("couldn't disassemble section");

            for insn in insns.iter() {
                offset += insn.bytes().len();

                let Ok(detail) = cs.insn_detail(insn) else {
                    continue;
                };

                for group_code in detail.groups() {
                    if seen_groups.insert(group_code.0) {
                        if let Some(mnemonic) = insn.mnemonic() {
                            if let Some(desc) = describe_group(&group_code.0) {
                                println!("{} ({})", desc, mnemonic);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut proc_features = seen_groups
        .iter()
        .filter_map(describe_group)
        .collect::<Vec<&str>>();
    proc_features.sort();

    println!(
        "Instruction set extensions used: {}",
        proc_features.join(", ")
    );

    // For x86 processors, we can map the instruction set features to specific CPU generations
    if cap_arch == CapArch::X86 {
        let mut max_gen_code = 100;
        for group_code in seen_groups.iter() {
            if let Some(desc) = describe_group(group_code) {
                match instrset_to_cpu.get(desc) {
                    Some(generation) => match cpu_generations.get(generation) {
                        Some(gen_code) => {
                            max_gen_code = cmp::max(max_gen_code, *gen_code);
                        }
                        None => unimplemented!(),
                    },
                    None => unimplemented!(),
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
}
