use std::collections::HashMap;

use crate::{Instruction, Opcode, WordAddress};

/// Stores map of program address in words and instructions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
	/// program is a list of instructions where the index is the word program address
	/// -> None -> previous instruction is 32-bit
	pub program_address_instruction_map: Vec<Option<Instruction>>,
	pub debug_symbol_table: HashMap<WordAddress, String>,
}

impl Program {
	pub fn new(instructions: &[Instruction]) -> Self {
		Self::with_debug_symbols(instructions, HashMap::new())
	}

	pub fn with_debug_symbols(
		instructions: &[Instruction],
		debug_symbol_table: HashMap<WordAddress, String>,
	) -> Self {
		let mut program_address_instruction_map = Vec::with_capacity(instructions.len());
		for instruction in instructions {
			let is_32bit = instruction.is_32bit();
			program_address_instruction_map.push(Some(*instruction));
			if is_32bit {
				program_address_instruction_map.push(None);
			}
		}

		Self {
			program_address_instruction_map,
			debug_symbol_table,
		}
	}

	pub fn len(&self) -> usize {
		self.program_address_instruction_map.len()
	}
	pub fn is_empty(&self) -> bool {
		self.program_address_instruction_map.is_empty()
	}

	/// returns None if the program address is out of bounds or
	/// if the program address is invalid, pointing at the second word of a 32-bit instruction
	pub fn get(&self, address: WordAddress) -> Option<Instruction> {
		self.program_address_instruction_map
			.get(address.0 as usize)
			.copied()
			.and_then(|opt_instruction| opt_instruction)
	}

	pub fn iter(&self) -> ProgramIter {
		ProgramIter {
			program: self,
			program_address: WordAddress(0),
		}
	}

	pub fn get_debug_symbol(&self, address: WordAddress) -> Option<&String> {
		self.debug_symbol_table.get(&address)
	}
}

impl Default for Program {
	fn default() -> Self {
		Program::new(&[])
	}
}

pub struct ProgramIter<'a> {
	program: &'a Program,
	program_address: WordAddress,
}

impl Iterator for ProgramIter<'_> {
	type Item = (WordAddress, Instruction);

	fn next(&mut self) -> Option<Self::Item> {
		let program_address = self.program_address;
		let opt_instruction = self
			.program
			.program_address_instruction_map
			.get(self.program_address.0 as usize)?;
		match opt_instruction {
			Some(instruction) => {
				self.program_address += instruction.get_word_size() as u16;
				Some((program_address, *instruction))
			}
			None => None,
		}
	}
}
