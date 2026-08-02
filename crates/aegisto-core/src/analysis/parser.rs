use anyhow::{Context, Result};
use goblin::Object;
use std::path::Path;

use crate::types::{ImportInfo, SectionInfo};

/// Parsed binary info returned to the caller.
pub struct ParsedBinary {
    pub format: String,
    pub entry_point: u64,
    pub sections: Vec<SectionInfo>,
    pub imports: Vec<ImportInfo>,
    /// Raw bytes of the executable section (name, vaddr, bytes) chosen for disassembly.
    pub executable_section: Option<(String, u64, Vec<u8>)>,
}

pub fn parse_file(path: &Path) -> Result<ParsedBinary> {
    let data =
        std::fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;

    match Object::parse(&data).context("Failed to parse binary format")? {
        Object::Elf(elf) => parse_elf(&elf, &data),
        Object::PE(pe) => parse_pe(&pe, &data),
        Object::Mach(_mach) => {
            anyhow::bail!("Mach-O parsing is not yet implemented");
        }
        Object::Archive(_archive) => {
            anyhow::bail!("Archive format is not supported");
        }
        _ => {
            anyhow::bail!("Unrecognized or unsupported binary format");
        }
    }
}

fn parse_elf(elf: &goblin::elf::Elf, _data: &[u8]) -> Result<ParsedBinary> {
    let entry_point = elf.entry;

    let sections: Vec<SectionInfo> = elf
        .section_headers
        .iter()
        .filter_map(|sh| {
            let name = elf.shdr_strtab.get_at(sh.sh_name).unwrap_or("").to_string();
            if name.is_empty() {
                return None;
            }
            Some(SectionInfo {
                name: name.clone(),
                size: sh.sh_size,
                virtual_address: sh.sh_addr,
                flags: format_section_flags_elf(sh.sh_flags),
            })
        })
        .collect();

    let imports: Vec<ImportInfo> = elf
        .dynsyms
        .iter()
        .filter(|sym| !sym.is_import())
        .filter_map(|sym| {
            let name = elf.dynstrtab.get_at(sym.st_name)?.to_string();
            let library = elf
                .libraries
                .first()
                .map(|s| s.to_string())
                .unwrap_or_default();
            if name.is_empty() {
                return None;
            }
            Some(ImportInfo { name, library })
        })
        .collect();

    // Find the best executable section for disassembly.
    let executable_section = find_executable_section_elf(elf, _data)?;

    Ok(ParsedBinary {
        format: format!("ELF ({})", elf_bitness_str(elf)),
        entry_point,
        sections,
        imports,
        executable_section,
    })
}

fn find_executable_section_elf<'a>(
    elf: &'a goblin::elf::Elf,
    data: &'a [u8],
) -> Result<Option<(String, u64, Vec<u8>)>> {
    // Prefer .text, then any SHF_EXECINSTR section.
    let text_idx = elf.section_headers.iter().position(|sh| {
        elf.shdr_strtab
            .get_at(sh.sh_name)
            .map(|n| n == ".text")
            .unwrap_or(false)
    });

    let exec_idx = text_idx.or_else(|| {
        elf.section_headers
            .iter()
            .position(|sh| sh.sh_flags & goblin::elf::section_header::SHF_EXECINSTR as u64 != 0)
    });

    match exec_idx {
        Some(idx) => {
            let sh = &elf.section_headers[idx];
            let name = elf
                .shdr_strtab
                .get_at(sh.sh_name)
                .unwrap_or("unknown")
                .to_string();
            let start = sh.sh_offset as usize;
            let end = start + sh.sh_size as usize;
            let bytes = data
                .get(start..end)
                .context("Section data extends beyond file")?
                .to_vec();
            Ok(Some((name, sh.sh_addr, bytes)))
        }
        None => Ok(None),
    }
}

fn parse_pe(pe: &goblin::pe::PE, data: &[u8]) -> Result<ParsedBinary> {
    let entry_point = pe
        .header
        .optional_header
        .as_ref()
        .map(|oh| oh.standard_fields.address_of_entry_point as u64)
        .unwrap_or(0);

    let sections: Vec<SectionInfo> = pe
        .sections
        .iter()
        .map(|s| {
            let name = String::from_utf8_lossy(
                &s.name[..s.name.iter().position(|&b| b == 0).unwrap_or(s.name.len())],
            )
            .into_owned();
            SectionInfo {
                name: name.clone(),
                size: s.size_of_raw_data as u64,
                virtual_address: s.virtual_address as u64,
                flags: format_section_flags_pe(s.characteristics),
            }
        })
        .collect();

    let imports: Vec<ImportInfo> = pe
        .imports
        .iter()
        .map(|imp| ImportInfo {
            name: imp.name.to_string(),
            library: imp.dll.to_string(),
        })
        .collect();

    let executable_section = find_executable_section_pe(pe, data)?;

    Ok(ParsedBinary {
        format: "PE".to_string(),
        entry_point,
        sections,
        imports,
        executable_section,
    })
}

fn find_executable_section_pe<'a>(
    pe: &'a goblin::pe::PE,
    data: &'a [u8],
) -> Result<Option<(String, u64, Vec<u8>)>> {
    const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;

    // Prefer .text, then first executable section.
    let text_idx = pe.sections.iter().position(|s| {
        let name = std::str::from_utf8(
            &s.name[..s.name.iter().position(|&b| b == 0).unwrap_or(s.name.len())],
        )
        .unwrap_or("");
        name == ".text"
    });

    let exec_idx = text_idx.or_else(|| {
        pe.sections
            .iter()
            .position(|s| s.characteristics & IMAGE_SCN_MEM_EXECUTE != 0)
    });

    match exec_idx {
        Some(idx) => {
            let s = &pe.sections[idx];
            let name = String::from_utf8_lossy(
                &s.name[..s.name.iter().position(|&b| b == 0).unwrap_or(s.name.len())],
            )
            .into_owned();
            let start = s.pointer_to_raw_data as usize;
            let end = start + s.size_of_raw_data as usize;
            let bytes = data
                .get(start..end)
                .context("Section data extends beyond file")?
                .to_vec();
            Ok(Some((name, s.virtual_address as u64, bytes)))
        }
        None => Ok(None),
    }
}

fn format_section_flags_elf(flags: u64) -> String {
    use goblin::elf::section_header::*;
    let mut f = String::new();
    if flags & SHF_WRITE as u64 != 0 {
        f.push('W');
    }
    if flags & SHF_ALLOC as u64 != 0 {
        f.push('A');
    }
    if flags & SHF_EXECINSTR as u64 != 0 {
        f.push('X');
    }
    if f.is_empty() {
        f.push_str("NONE");
    }
    f
}

fn format_section_flags_pe(characteristics: u32) -> String {
    let mut f = String::new();
    if characteristics & 0x40000000 != 0 {
        f.push('R');
    }
    if characteristics & 0x80000000 != 0 {
        f.push('W');
    }
    if characteristics & 0x20000000 != 0 {
        f.push('X');
    }
    if f.is_empty() {
        f.push_str("NONE");
    }
    f
}

fn elf_bitness_str(elf: &goblin::elf::Elf<'_>) -> &'static str {
    match elf.header.e_ident[goblin::elf::header::EI_CLASS] {
        goblin::elf::header::ELFCLASS64 => "64-bit",
        goblin::elf::header::ELFCLASS32 => "32-bit",
        _ => "unknown-bit",
    }
}
