use crate::{
	register::{HlOrImm16, RegisterOrImm8},
	AsmParseError, AsmParseErrorType, Instruction, Register,
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

/// parses different number formats like "0x123" or "0b10101" and normal "42"
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
	match s {
		"a" => Ok(Register::A),
		"b" => Ok(Register::B),
		"c" => Ok(Register::C),
		"d" => Ok(Register::D),
		"l" => Ok(Register::L),
		"h" => Ok(Register::H),
		"z" => Ok(Register::Z),
		"f" => Ok(Register::F),
		_ => Err(AsmParseErrorType::InvalidRegister(s.to_string())),
	}
}

fn parse_imm8_or_register(s: &str) -> Result<RegisterOrImm8, AsmParseErrorType> {
	match parse_register(s) {
		Ok(register) => Ok(RegisterOrImm8::Register(register)),
		Err(_) => parse_number(s).map(|immediate| RegisterOrImm8::Imm8(immediate as u8)),
	}
}

fn split_mnemonic_operands(line: &str) -> (String, Vec<&str>) {
	let parts: Vec<&str> = line.split_whitespace().collect();
	if parts.is_empty() {
		return (String::new(), Vec::new());
	}
	let mnemonic = parts[0].to_string();
	let operands = parts[1..]
		.iter()
		.flat_map(|s| s.split(','))
		.map(|s| s.trim())
		.filter(|s| !s.is_empty())
		.collect();
	(mnemonic, operands)
}

fn consume_operands<'a, 'b, const N: usize>(
	operands: &'a [&'b str],
) -> Result<&'a [&'b str; N], AsmParseErrorType> {
	match operands.try_into() {
		Ok(operands) => Ok(operands),
		Err(_) => Err(AsmParseErrorType::InvalidArgumentCount {
			expected_count: N,
			actual_count: operands.len(),
		}),
	}
}

fn parse_instruction(
	mnemonic: &str,
	operands: &[&str],
	symbol_table: &HashMap<String, usize>,
) -> Result<Instruction, AsmParseErrorType> {
	match mnemonic.to_uppercase().as_str() {
		"MW" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			Ok(Instruction::Mw { reg, value })
		}
		"LW" => match operands.len() {
			1 | 2 => {
				let register = parse_register(operands[0])?;
				let address = match operands.get(1) {
					Some(address) => (parse_number(address)? as u16).into(),
					None => HlOrImm16::Hl,
				};
				Ok(Instruction::Lw { register, address })
			}
			_ => Err(AsmParseErrorType::InvalidDynamicArgumentCount {
				allowed_counts: vec![1, 2],
				actual_count: operands.len(),
			}),
		},
		"SW" => match operands.len() {
			1 => {
				let address = HlOrImm16::Hl;
				let register = parse_register(operands[0])?;
				Ok(Instruction::Sw { address, register })
			}
			2 => {
				let address = (parse_number(operands[0])? as u16).into();
				let register = parse_register(operands[1])?;
				Ok(Instruction::Sw { address, register })
			}
			_ => Err(AsmParseErrorType::InvalidDynamicArgumentCount {
				allowed_counts: vec![1, 2],
				actual_count: operands.len(),
			}),
		},
		"LDA" => {
			let operands = consume_operands::<1>(operands)?;
			let address = symbol_table
				.get(operands[0])
				.copied()
				.ok_or(AsmParseErrorType::UndefinedLabel(operands[0].to_string()))?
				as u16;
			Ok(Instruction::Lda {
				address: crate::register::Imm16(address),
			})
		}
		"JNZ" => {
			let operands = consume_operands::<1>(operands)?;
			let value = parse_imm8_or_register(operands[0])?;
			Ok(Instruction::Jnz { value })
		}
		"ADD" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			Ok(Instruction::Add { reg, value })
		}
		"SUB" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			Ok(Instruction::Sub { reg, value })
		}
		"AND" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			Ok(Instruction::And { reg, value })
		}
		"OR" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			Ok(Instruction::Or { reg, value })
		}
		"NOR" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			Ok(Instruction::Nor { reg, value })
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
	for (i, line) in lines.iter().enumerate() {
		let line_number = i + 1;

		if parse_label(line).is_some() {
			continue;
		}

		let (mnemonic, operands) = split_mnemonic_operands(line);
		if mnemonic.is_empty() {
			continue;
		}

		let instruction = parse_instruction(&mnemonic, &operands, &symbol_table)
			.map_err(|error| AsmParseError::new(error, line_number))?;

		instructions.push(instruction);
	}

	Ok(instructions)
}

#[cfg(test)]
mod tests {
	use crate::{self as ardemu_core, parse_asm, AsmParseError, AsmParseErrorType};
	use crate::{Instruction, A, B, C};
	use ardemu_asm_parse_macro::parse_asm;

	#[test]
	fn test_parse_asm() {
		assert_eq!(
			parse_asm!(
				r"
				; leading comment
				mw a, 0 ; comment
				mw b, 1
				mw c, 2
				; another comment
			loop:
				add a, b
				and a, b
				lda loop
				jnz 1
			"
			),
			[
				Instruction::Mw {
					reg: A,
					value: 0.into()
				},
				Instruction::Mw {
					reg: B,
					value: 1.into()
				},
				Instruction::Mw {
					reg: C,
					value: 2.into()
				},
				Instruction::Add {
					reg: A,
					value: B.into(),
				},
				Instruction::And {
					reg: A,
					value: B.into(),
				},
				Instruction::Lda {
					address: crate::register::Imm16(3)
				},
				Instruction::Jnz { value: 1.into() },
			]
		);
	}

	#[test]
	fn test_parse_invalid_argument_count() {
		assert_eq!(
			parse_asm("mw a"),
			Err(AsmParseError::new(
				AsmParseErrorType::InvalidArgumentCount {
					expected_count: 2,
					actual_count: 1,
				},
				1
			))
		);
		assert_eq!(
			parse_asm("jnz "),
			Err(AsmParseError::new(
				AsmParseErrorType::InvalidArgumentCount {
					expected_count: 1,
					actual_count: 0,
				},
				1
			))
		);
		assert_eq!(
			parse_asm("add a 0x0 0x1"),
			Err(AsmParseError::new(
				AsmParseErrorType::InvalidArgumentCount {
					expected_count: 2,
					actual_count: 3,
				},
				1
			))
		);
	}
}
