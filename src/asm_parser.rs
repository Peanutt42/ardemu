use crate::{
	error::{AsmParseError, AsmParseErrorType},
	Instruction,
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
	} else if let Some(s) = s.strip_prefix("r") {
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
		"LDI" => {
			let reg = parse_number(&operands[0][1..])? as usize;
			let value = parse_number(&operands[1])? as u8;
			Ok(Instruction::Ldi { reg, value })
		}
		"ADD" => {
			let rd = parse_number(&operands[0])? as usize;
			let rs = parse_number(&operands[1])? as usize;
			Ok(Instruction::Add { rd, rs })
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
			let reg = parse_number(&operands[0])? as usize;
			let addr = parse_number(&operands[1])? as usize;
			Ok(Instruction::Store { reg, addr })
		}
		"NOP" => Ok(Instruction::Nop),
		_ => Err(AsmParseErrorType::InvalidInstruction),
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
				.map_err(|error| AsmParseError::new(line_number, error))?;

		instructions.push(instruction);
		current_program_counter += 1;
	}

	Ok(instructions)
}

#[macro_export]
macro_rules! include_asm {
	($file:literal) => {{
		ardemu::parse_asm(include_str!($file)).expect(concat!("failed to parse asm file: ", $file))
	}};
}

#[macro_export]
macro_rules! include_asm_str {
	($str:literal) => {{
		ardemu::parse_asm($str).expect("failed to parse inline asm string")
	}};
}

#[cfg(test)]
mod tests {
	use crate::{self as ardemu, Instruction};

	#[test]
	fn test_parse_asm() {
		let asm = include_asm_str!(
			r"
	ldi r0, 0x0     ; r0 = LOW
	ldi r1, 0x20    ; r1 = HIGH
loop:
	store r1, 0x25  ; turn LED on
	store r0, 0x25  ; turn LED off
	jmp loop
			"
		);

		assert_eq!(
			asm,
			vec![
				Instruction::Ldi { reg: 0, value: 0 },
				Instruction::Ldi {
					reg: 1,
					value: 0x20
				},
				Instruction::Store { reg: 1, addr: 0x25 },
				Instruction::Store { reg: 0, addr: 0x25 },
				Instruction::Jmp { offset: -2 },
			]
		);
	}
}
