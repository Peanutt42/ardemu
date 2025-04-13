use std::collections::HashMap;

use elf::{
	abi::{STB_GLOBAL, STB_LOCAL, STB_WEAK, STT_FUNC, STT_NOTYPE, STT_OBJECT},
	endian::AnyEndian,
	parse::ParsingTable,
	section::SectionHeader,
	string_table::StringTable,
	ElfBytes,
};

use crate::{LoadElfError, Program, WordAddress};

fn find_section(
	name: &'static str,
	shdrs: &ParsingTable<AnyEndian, SectionHeader>,
	strtab: &StringTable,
) -> Option<(usize, SectionHeader)> {
	for (section_index, section) in shdrs.iter().enumerate() {
		if let Ok(section_name) = strtab.get(section.sh_name as usize) {
			if section_name == name {
				return Some((section_index, section));
			}
		}
	}
	None
}

/// returns (code_section_header_index, instructions)
fn load_instructions(elf: &ElfBytes<AnyEndian>) -> Result<(usize, Vec<u16>), LoadElfError> {
	let (Some(shdrs), Some(strtab)) = elf.section_headers_with_strtab()? else {
		return Err(LoadElfError::CouldNotFindCodeSection);
	};

	let Some((code_section_header_index, code_section)) = find_section(".text", &shdrs, &strtab)
	else {
		return Err(LoadElfError::CouldNotFindCodeSection);
	};

	if code_section.sh_addr != 0 {
		return Err(LoadElfError::NonZeroBaseCodeAddress);
	}

	let (flash_code_binary, flash_code_compression) = elf
		.section_data(&code_section)
		.map_err(|_| LoadElfError::CouldNotFindCodeSection)?;

	if flash_code_compression.is_some() {
		return Err(LoadElfError::CompressedNotSupported);
	}

	let mut flash = flash_code_binary.to_vec();

	if let Some((_index, data_section)) = find_section(".data", &shdrs, &strtab) {
		if let Ok((flash_data_binary, None)) = elf.section_data(&data_section) {
			if !flash_data_binary.is_empty() {
				flash.append(&mut flash_data_binary.to_vec());
			}
		}
	}

	Ok((
		code_section_header_index,
		Program::load_flash_binary(&flash)?,
	))
}

fn load_debug_symbol_table(
	elf: &ElfBytes<AnyEndian>,
	code_section_index: usize,
) -> HashMap<WordAddress, String> {
	// Value: (name, bind)
	let mut debug_symbol_table = HashMap::<WordAddress, (String, u8)>::new();
	if let Ok(Some((symbol_table, string_table))) = elf.symbol_table() {
		for symbol in symbol_table.iter() {
			if symbol.st_shndx as usize != code_section_index {
				continue;
			}

			let st_type = symbol.st_symtype();

			if st_type == STT_NOTYPE || st_type == STT_OBJECT || st_type == STT_FUNC {
				if let Ok(name) = string_table.get(symbol.st_name as usize) {
					// 1 word = 2 bytes
					#[allow(clippy::cast_possible_truncation)]
					let address = WordAddress((symbol.st_value / 2) as u32);
					let st_bind = symbol.st_bind();
					match debug_symbol_table.get_mut(&address) {
						Some((_prev_symbol_name, prev_symbol_bind)) => {
							let should_override = match st_bind {
								STB_GLOBAL => true,
								STB_LOCAL => *prev_symbol_bind == STB_WEAK,
								_ => false,
							};

							if should_override {
								debug_symbol_table.insert(address, (name.to_string(), st_bind));
							}
						}
						None => {
							debug_symbol_table.insert(address, (name.to_string(), st_bind));
						}
					}
				}
			}
		}
	}
	debug_symbol_table
		.into_iter()
		.map(|(address, (name, _bind))| (address, name))
		.collect()
}

pub fn load_elf(elf_content: &[u8]) -> Result<Program, LoadElfError> {
	let elf = ElfBytes::<AnyEndian>::minimal_parse(elf_content)?;

	let (code_section_index, flash) = load_instructions(&elf)?;

	let debug_symbol_table = load_debug_symbol_table(&elf, code_section_index);

	Ok(Program::with_debug_symbols(flash, debug_symbol_table))
}
