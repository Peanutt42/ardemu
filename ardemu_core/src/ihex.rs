use ihex::{Reader, Record};

use crate::{Instruction, LoadIHex, Opcode, Program};

pub fn load_ihex_str(ihex_content: &str) -> Result<Program, LoadIHex> {
	let mut opcodes_binary = Vec::new();
	let ihex_reader = Reader::new(ihex_content);
	for record in ihex_reader {
		if let Record::Data { value, offset } = record? {
			if opcodes_binary.len() < offset as usize + value.len() {
				opcodes_binary.resize(offset as usize + value.len(), 0);
			}
			opcodes_binary[offset as usize..offset as usize + value.len()].copy_from_slice(&value);
		}
	}

	if opcodes_binary.len() % 2 != 0 {
		return Err(LoadIHex::InvalidAlignment);
	}

	let mut program_address = 0;
	let mut instructions = Vec::new();
	while program_address + std::mem::size_of::<u16>() <= opcodes_binary.len() {
		let first_opcode_16bit = u16::from_le_bytes([
			opcodes_binary[program_address],
			opcodes_binary[program_address + 1],
		]);

		let opcode_32bit = if opcodes_binary.len() > program_address + std::mem::size_of::<u32>() {
			let second_opcode_16bit = u16::from_le_bytes([
				opcodes_binary[program_address + 2],
				opcodes_binary[program_address + 3],
			]);

			((first_opcode_16bit as u32) << 16) | (second_opcode_16bit as u32)
		} else {
			(first_opcode_16bit as u32) << 16
		};

		match Instruction::load(opcode_32bit) {
			Some(instruction) => {
				program_address += instruction.get_byte_size() as usize;
				instructions.push(instruction);
			}
			None => {
				return Err(LoadIHex::UnsupportedInstruction {
					opcode_32bit,
					program_address: program_address as u16,
				})
			}
		}
	}

	Ok(Program::new(&instructions))
}
