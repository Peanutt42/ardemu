use ihex::{Reader, Record};

use crate::{LoadIHexError, Program};

pub fn load_ihex_str(ihex_content: &str) -> Result<Program, LoadIHexError> {
	let mut flash_binary = Vec::new();
	let ihex_reader = Reader::new(ihex_content);
	for record in ihex_reader {
		if let Record::Data { mut value, offset } = record? {
			if flash_binary.len() < offset as usize + value.len() {
				flash_binary.resize(offset as usize + value.len(), 0);
			}
			value.swap_with_slice(match flash_binary.get_mut(offset as usize..) {
				Some(slice) => slice,
				None => unreachable!("should have been resized enough to not panic!"),
			});
		}
	}

	let flash = Program::load_flash_binary(&flash_binary)?;

	Ok(Program::new(flash))
}
