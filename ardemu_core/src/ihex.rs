use ihex::{Reader, Record};

use crate::{LoadIHexError, Program};

pub fn load_ihex_str(ihex_content: &str) -> Result<Program, LoadIHexError> {
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

	let instructions = Program::load_instructions(&opcodes_binary)?;

	Ok(Program::new(&instructions))
}
