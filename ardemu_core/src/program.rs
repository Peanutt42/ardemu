use std::collections::HashMap;

use crate::{u8s_to_u16, Instruction, LoadProgramError, Opcode, WordAddress};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
	/// every index is the program address in words (2 bytes!)
	pub flash: Vec<u16>,
	pub debug_symbol_table: HashMap<WordAddress, String>,
}

impl Program {
	pub fn new(flash: Vec<u16>) -> Self {
		Self::with_debug_symbols(flash, HashMap::new())
	}

	pub fn with_debug_symbols(
		flash: Vec<u16>,
		debug_symbol_table: HashMap<WordAddress, String>,
	) -> Self {
		Self {
			flash,
			debug_symbol_table,
		}
	}

	pub fn load_instruction_list(instructions: &[Instruction]) -> Self {
		Self::new(Self::load_instruction_list_as_flash(instructions))
	}

	pub fn load_instruction_list_as_flash(instructions: &[Instruction]) -> Vec<u16> {
		let mut flash = Vec::<u16>::with_capacity(instructions.len());
		for instruction in instructions {
			let opcode = instruction.get_opcode();

			let first_opcode = (opcode >> 16) as u16;
			let second_opcode = (opcode & 0xFFFF) as u16;
			if instruction.is_32bit() {
				flash.push(first_opcode);
				flash.push(second_opcode);
			} else {
				flash.push(first_opcode);
				assert_eq!(second_opcode, 0, "{instruction}");
			}
		}
		flash
	}

	/// loads flash (u16's) from binary (u8's)
	pub fn load_flash_binary(code: &[u8]) -> Result<Vec<u16>, LoadProgramError> {
		if !code.len().is_multiple_of(2) {
			return Err(LoadProgramError::InvalidAlignment);
		}

		let mut flash = Vec::with_capacity(code.len() / 2);
		for low_high in code.chunks(2) {
			match low_high {
				[low, high] => {
					let word = u8s_to_u16(*low, *high);
					flash.push(word);
				}
				_ => unreachable!("chunks(2) should only return slices of length 2"),
			}
		}
		Ok(flash)
	}

	/// returns the flash size in words
	/// 32-bit instructions are two 16-bit (1 word) instruction spots: [Some(Instruction::A64BitInstr), None]
	pub fn len(&self) -> usize {
		self.flash.len()
	}
	pub fn is_empty(&self) -> bool {
		self.flash.is_empty()
	}

	/// returns None if the program address is out of bounds or
	/// if the program address is invalid, pointing at the second word of a 32-bit instruction
	pub fn get_instruction(&self, address: WordAddress) -> Option<Instruction> {
		let first_opcode_16bit = self.flash.get(address.0 as usize)?;
		let opcode_32bit = match self.flash.get(address.0 as usize + 1) {
			Some(second_opcode_16bit) => {
				((*first_opcode_16bit as u32) << 16) | (*second_opcode_16bit as u32)
			}
			None => (*first_opcode_16bit as u32) << 16,
		};

		Instruction::load(opcode_32bit)
	}

	pub fn iter(&self) -> ProgramIter<'_> {
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
		Program::new(Vec::default())
	}
}

pub struct ProgramIter<'a> {
	program: &'a Program,
	program_address: WordAddress,
}

impl Iterator for ProgramIter<'_> {
	type Item = (WordAddress, Option<Instruction>);

	fn next(&mut self) -> Option<Self::Item> {
		if self.program_address.0 as usize >= self.program.len() {
			return None;
		}

		let program_address = self.program_address;
		let instruction = self.program.get_instruction(self.program_address);
		if let Some(instruction) = &instruction {
			self.program_address += instruction.get_word_size() as u16;
		} else {
			self.program_address += 1;
		}
		Some((program_address, instruction))
	}
}
