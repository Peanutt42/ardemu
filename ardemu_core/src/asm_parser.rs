use crate::{
	register::{HlOrImm16, RegisterOrImm8},
	AsmParseError, AsmParseErrorType, Instruction, Register,
};
use std::collections::HashMap;

struct Line {
	/// preprocessed line
	str: String,
	/// original from source code
	line_number: usize,
	/// parsed label
	label: Option<String>,
}

impl Line {
	/// preprocesses `str` and removes any comments (';')
	fn new(str: &str, line_number: usize) -> Self {
		let str_without_comments = str.split(';').next().unwrap_or("").trim().to_string();
		let label = if str_without_comments.ends_with(':') {
			str_without_comments
				.split(':')
				.next()
				.map(|s| s.trim().to_string())
		} else {
			None
		};

		Self {
			str: str_without_comments,
			line_number,
			label,
		}
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
	let lines: Vec<Line> = asm
		.lines()
		.enumerate()
		.map(|(i, line_str)| Line::new(line_str, i + 1))
		.filter(|l| !l.str.is_empty())
		.collect();

	// maps symbol label to program address
	let mut symbol_table: HashMap<String, usize> = HashMap::new();
	let mut program_counter = 0;
	for line in &lines {
		match &line.label {
			Some(label) => {
				symbol_table.insert(label.clone(), program_counter);
			}
			None if !line.str.starts_with('.') => program_counter += 1,
			_ => {}
		}
	}

	let mut instructions = Vec::new();
	for line in lines {
		if line.label.is_some() {
			continue;
		}

		let (mnemonic, operands) = split_mnemonic_operands(&line.str);
		if mnemonic.is_empty() {
			continue;
		}

		let instruction = parse_instruction(&mnemonic, &operands, &symbol_table)
			.map_err(|error| AsmParseError::new(error, line.line_number))?;

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

	#[test]
	fn test_parse_line_number() {
		assert_eq!(
			parse_asm(
				r"; line 1
			; line 2

			; empty line above...
			invalidinstruction ; on line 3"
			),
			Err(AsmParseError::new(
				AsmParseErrorType::InvalidInstruction("invalidinstruction".to_string()),
				5
			))
		);
	}
}
