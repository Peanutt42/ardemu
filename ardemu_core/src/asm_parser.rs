use crate::{
	register::RegisterOrImmediate, AsmParseError, AsmParseErrorType, Instruction, Register,
};
use std::collections::HashMap;

/// removes comments (';')
fn preprocess_line(line: &str) -> String {
	let line = line.split(';').next().unwrap_or("").trim();
	line.to_string()
}

fn parse_label(line: &str) -> Option<String> {
	let line = line.trim();
	if line.ends_with(':') {
		let label = line.split(':').next()?.trim();
		Some(label.to_string())
	} else {
		None
	}
}

fn parse_number(s: &str) -> Result<u32, AsmParseErrorType> {
	if let Some(s) = s.strip_prefix("0x") {
		u32::from_str_radix(s, 16).map_err(|source| AsmParseErrorType::InvalidNumber {
			string: s.to_string(),
			source,
		})
	} else if let Some(s) = s.strip_prefix("0b") {
		u32::from_str_radix(s, 2).map_err(|source| AsmParseErrorType::InvalidNumber {
			string: s.to_string(),
			source,
		})
	} else {
		s.parse::<u32>()
			.map_err(|source| AsmParseErrorType::InvalidNumber {
				string: s.to_string(),
				source,
			})
	}
}

fn parse_register(s: &str) -> Result<Register, AsmParseErrorType> {
	match s.strip_prefix("r") {
		Some(s) => {
			let number =
				u8::from_str_radix(s, 2).map_err(|source| AsmParseErrorType::InvalidNumber {
					string: s.to_string(),
					source,
				})?;
			Register::try_from(number)
				.map_err(|_| AsmParseErrorType::InvalidRegister(s.to_string()))
		}
		None => Err(AsmParseErrorType::InvalidRegister(s.to_string())),
	}
}

fn parse_immediate_or_register(s: &str) -> Result<RegisterOrImmediate, AsmParseErrorType> {
	match parse_register(s) {
		Ok(register) => Ok(RegisterOrImmediate::Register(register)),
		Err(_) => parse_number(s).map(|immediate| RegisterOrImmediate::Immediate(immediate as u8)),
	}
}

fn split_mnemonic_operands(line: &str) -> (String, Vec<String>) {
	let parts: Vec<&str> = line.split_whitespace().collect();
	if parts.is_empty() {
		return (String::new(), Vec::new());
	}
	let mnemonic = parts[0].to_string();
	let operands = parts[1..]
		.iter()
		.flat_map(|s| s.split(','))
		.map(|s| s.trim().to_string())
		.filter(|s| !s.is_empty())
		.collect();
	(mnemonic, operands)
}

fn parse_instruction(
	mnemonic: &str,
	operands: &[String],
	symbol_table: &HashMap<String, usize>,
	current_program_counter: usize,
) -> Result<Instruction, AsmParseErrorType> {
	match mnemonic.to_uppercase().as_str() {
		"MOVE" => {
			let reg = parse_register(&operands[0])?;
			let value = parse_immediate_or_register(&operands[1])?;
			Ok(Instruction::Move { reg, value })
		}
		"ADD" => {
			let reg = parse_register(&operands[0])?;
			let value = parse_immediate_or_register(&operands[1])?;
			Ok(Instruction::Add { reg, value })
		}
		"JMP" => {
			let target = symbol_table
				.get(&operands[0])
				.copied()
				.ok_or(AsmParseErrorType::UndefinedLabel(operands[0].clone()))?;
			let offset = target as i32 - current_program_counter as i32;
			Ok(Instruction::Jmp { offset })
		}
		"STORE" => {
			let value = parse_immediate_or_register(&operands[0])?;
			let addr = parse_number(&operands[1])? as usize;
			Ok(Instruction::Store { value, addr })
		}
		_ => Err(AsmParseErrorType::InvalidInstruction(mnemonic.to_string())),
	}
}

pub fn parse_asm(asm: &str) -> Result<Vec<Instruction>, AsmParseError> {
	let lines: Vec<String> = asm
		.lines()
		.map(preprocess_line)
		.filter(|l| !l.is_empty())
		.collect();

	let mut symbol_table = HashMap::new();
	let mut program_counter = 0;
	for line in &lines {
		match parse_label(line) {
			Some(label) => {
				symbol_table.insert(label, program_counter);
			}
			None if !line.starts_with('.') => program_counter += 1,
			_ => {}
		}
	}

	let mut instructions = Vec::new();
	let mut current_program_counter = 0;
	for (i, line) in lines.iter().enumerate() {
		let line_number = i + 1;

		if parse_label(line).is_some() {
			continue;
		}

		let (mnemonic, operands) = split_mnemonic_operands(line);
		if mnemonic.is_empty() {
			continue;
		}

		let instruction =
			parse_instruction(&mnemonic, &operands, &symbol_table, current_program_counter)
				.map_err(|error| AsmParseError::new(error, line_number))?;

		instructions.push(instruction);
		current_program_counter += 1;
	}

	Ok(instructions)
}

#[cfg(test)]
mod tests {
	use crate as ardemu_core;
	use crate::{Instruction, Register};
	use ardemu_asm_parse_macro::parse_asm;

	#[test]
	fn test_parse_asm() {
		assert_eq!(
			parse_asm!(
				r"
				move r0, 0x0     ; r0 = LOW
				move r1, 0x20    ; r1 = HIGH
			loop:
				store r1, 0x25  ; turn LED on
				store r0, 0x25  ; turn LED off
				jmp loop
			"
			),
			[
				Instruction::Move {
					reg: Register::R0,
					value: 0.into()
				},
				Instruction::Move {
					reg: Register::R1,
					value: 0x20.into()
				},
				Instruction::Store {
					value: Register::R1.into(),
					addr: 0x25
				},
				Instruction::Store {
					value: Register::R0.into(),
					addr: 0x25
				},
				Instruction::Jmp { offset: -2 },
			]
		);
	}
}
