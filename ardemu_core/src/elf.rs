use std::collections::HashMap;

use elf::{endian::AnyEndian, ElfBytes};

use crate::{Instruction, LoadElfError, Program, WordAddress};

/// returns (section_header_index, instructions)
fn load_instructions(elf: &ElfBytes<AnyEndian>) -> Result<(usize, Vec<Instruction>), LoadElfError> {
	match elf.section_headers_with_strtab()? {
		(Some(shdrs), Some(strtab)) => {
			for (section_index, section) in shdrs.iter().enumerate() {
				if let Ok(section_name) = strtab.get(section.sh_name as usize) {
					if section_name == ".text" {
						if section.sh_addr != 0 {
							return Err(LoadElfError::NonZeroBaseCodeAddress);
						}

						return match elf.section_data(&section) {
							Ok((code, None)) => {
								Ok((section_index, Program::load_instructions(code)?))
							}
							Ok((_code, Some(_compression_header))) => {
								Err(LoadElfError::CompressedNotSupported)
							}
							Err(_) => Err(LoadElfError::CouldNotFindCodeSection),
						};
					}
				}
			}
			Err(LoadElfError::CouldNotFindCodeSection)
		}
		_ => Err(LoadElfError::CouldNotFindCodeSection),
	}
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
			let st_bind = symbol.st_bind();

			// 0: NoType
			// 2: Function
			if st_type == 0 || st_type == 2 {
				if let Ok(name) = string_table.get(symbol.st_name as usize) {
					// 1 word = 2 bytes
					let address = WordAddress((symbol.st_value / 2) as u32);
					match debug_symbol_table.get_mut(&address) {
						Some((_prev_symbol_name, prev_symbol_bind)) => {
							// use the latest symbol for the same address
							// bind: global (1), local (2), etc.
							// priorities global over local
							if st_bind <= *prev_symbol_bind {
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

	let (code_section_index, instructions) = load_instructions(&elf)?;

	let debug_symbol_table = load_debug_symbol_table(&elf, code_section_index);

	Ok(Program::with_debug_symbols(
		&instructions,
		debug_symbol_table,
	))
}
