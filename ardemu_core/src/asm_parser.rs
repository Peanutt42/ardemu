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

/// Substitutes a LDA instruction to a program address with a LDA instruction to a symbol, which is later converted back to a LDA instruction to a program address.
enum IntermediateInstruction {
	Lda { symbol: String, line_number: usize },
	Instruction(Instruction),
}
impl From<Instruction> for IntermediateInstruction {
	fn from(instruction: Instruction) -> Self {
		IntermediateInstruction::Instruction(instruction)
	}
}

fn inc_macro(reg: Register) -> Instruction {
	Instruction::Add {
		reg,
		value: 1.into(),
	}
}

fn dec_macro(reg: Register) -> Instruction {
	Instruction::Sub {
		reg,
		value: 1.into(),
	}
}

fn not_macro(reg: Register) -> Instruction {
	Instruction::Nor {
		reg,
		value: reg.into(),
	}
}

fn nand_macro(reg: Register, value: RegisterOrImm8) -> [Instruction; 2] {
	[Instruction::And { reg, value }, not_macro(reg)]
}

fn jmp_macro(symbol: String, line_number: usize) -> [IntermediateInstruction; 2] {
	[
		IntermediateInstruction::Lda {
			symbol,
			line_number,
		},
		Instruction::Jnz { value: 1.into() }.into(),
	]
}

/// appends the parsed instruction onto 'output_instruction'
fn parse_instruction(
	line_number: usize,
	mnemonic: &str,
	operands: &[&str],
	output_instruction: &mut Vec<IntermediateInstruction>,
) -> Result<(), AsmParseErrorType> {
	match mnemonic.to_uppercase().as_str() {
		"MW" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			output_instruction.push(Instruction::Mw { reg, value }.into());
			Ok(())
		}
		"LW" => match operands.len() {
			1 | 2 => {
				let register = parse_register(operands[0])?;
				let address = match operands.get(1) {
					Some(address) => (parse_number(address)? as u16).into(),
					None => HlOrImm16::Hl,
				};
				output_instruction.push(Instruction::Lw { register, address }.into());
				Ok(())
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
				output_instruction.push(Instruction::Sw { address, register }.into());
				Ok(())
			}
			2 => {
				let address = (parse_number(operands[0])? as u16).into();
				let register = parse_register(operands[1])?;
				output_instruction.push(Instruction::Sw { address, register }.into());
				Ok(())
			}
			_ => Err(AsmParseErrorType::InvalidDynamicArgumentCount {
				allowed_counts: vec![1, 2],
				actual_count: operands.len(),
			}),
		},
		"PUSH" => {
			let operands = consume_operands::<1>(operands)?;
			let value = parse_imm8_or_register(operands[0])?;
			output_instruction.push(Instruction::Push { value }.into());
			Ok(())
		}
		"POP" => {
			let operands = consume_operands::<1>(operands)?;
			let register = parse_register(operands[0])?;
			output_instruction.push(Instruction::Pop { register }.into());
			Ok(())
		}
		"LDA" => {
			let operands = consume_operands::<1>(operands)?;
			output_instruction.push(IntermediateInstruction::Lda {
				symbol: operands[0].to_string(),
				line_number,
			});
			Ok(())
		}
		"JNZ" => {
			let operands = consume_operands::<1>(operands)?;
			let value = parse_imm8_or_register(operands[0])?;
			output_instruction.push(Instruction::Jnz { value }.into());
			Ok(())
		}
		"ADD" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			output_instruction.push(Instruction::Add { reg, value }.into());
			Ok(())
		}
		"SUB" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			output_instruction.push(Instruction::Sub { reg, value }.into());
			Ok(())
		}
		"AND" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			output_instruction.push(Instruction::And { reg, value }.into());
			Ok(())
		}
		"OR" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			output_instruction.push(Instruction::Or { reg, value }.into());
			Ok(())
		}
		"NOR" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			output_instruction.push(Instruction::Nor { reg, value }.into());
			Ok(())
		}
		/*
		<=========================>
		from here on only "macros", so only alias instructions that are composed of the native instructions from above ^
		<=========================>
		*/
		"INC" => {
			let operands = consume_operands::<1>(operands)?;
			let reg = parse_register(operands[0])?;
			output_instruction.push(inc_macro(reg).into());
			Ok(())
		}
		"DEC" => {
			let operands = consume_operands::<1>(operands)?;
			let reg = parse_register(operands[0])?;
			output_instruction.push(dec_macro(reg).into());
			Ok(())
		}
		"NOT" => {
			let operands = consume_operands::<1>(operands)?;
			let reg = parse_register(operands[0])?;
			output_instruction.push(not_macro(reg).into());
			Ok(())
		}
		"NAND" => {
			let operands = consume_operands::<2>(operands)?;
			let reg = parse_register(operands[0])?;
			let value = parse_imm8_or_register(operands[1])?;
			output_instruction.append(
				&mut nand_macro(reg, value)
					.into_iter()
					.map(|i| i.into())
					.collect::<Vec<IntermediateInstruction>>(),
			);
			Ok(())
		}
		"JMP" => match operands.len() {
			0 => {
				output_instruction.push(Instruction::Jnz { value: 1.into() }.into());
				Ok(())
			}
			1 => {
				output_instruction.extend(jmp_macro(operands[0].to_string(), line_number));
				Ok(())
			}
			_ => Err(AsmParseErrorType::InvalidDynamicArgumentCount {
				allowed_counts: vec![0, 1],
				actual_count: operands.len(),
			}),
		},
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

	// maps symbols to program addresses
	let mut symbol_table: HashMap<String, u16> = HashMap::new();
	let mut intermediate_instructions: Vec<IntermediateInstruction> = Vec::new();
	for line in lines {
		if let Some(label) = &line.label {
			symbol_table.insert(label.clone(), intermediate_instructions.len() as u16);
			continue;
		}

		let (mnemonic, operands) = split_mnemonic_operands(&line.str);
		if mnemonic.is_empty() {
			continue;
		}

		parse_instruction(
			line.line_number,
			&mnemonic,
			&operands,
			&mut intermediate_instructions,
		)
		.map_err(|error| AsmParseError::new(error, line.line_number))?;
	}

	let mut instructions: Vec<Instruction> = Vec::with_capacity(intermediate_instructions.len());
	for intermediate_instruction in intermediate_instructions {
		let instruction = match intermediate_instruction {
			IntermediateInstruction::Instruction(instruction) => instruction,
			IntermediateInstruction::Lda {
				symbol,
				line_number,
			} => match symbol_table.get(&symbol.clone()) {
				Some(&program_address) => Instruction::Lda {
					address: program_address.into(),
				},
				None => {
					return Err(AsmParseError::new(
						AsmParseErrorType::UndefinedLabel(symbol),
						line_number,
					));
				}
			},
		};
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
				push a
				pop a
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
				Instruction::Push { value: A.into() },
				Instruction::Pop { register: A },
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

	#[test]
	fn test_asm_macro_expansion() {
		assert_eq!(
			parse_asm(
				r"start_label_at_0:
				inc a
				dec a
				not a
				nand a, b
				lda start_label_at_0
				jmp
				jmp start_label_at_0
				"
			),
			Ok(vec![
				Instruction::Add {
					reg: A,
					value: 1.into()
				},
				Instruction::Sub {
					reg: A,
					value: 1.into()
				},
				Instruction::Nor {
					reg: A,
					value: A.into()
				},
				Instruction::And {
					reg: A,
					value: B.into()
				},
				Instruction::Nor {
					reg: A,
					value: A.into()
				},
				Instruction::Lda { address: 0.into() },
				Instruction::Jnz { value: 1.into() },
				Instruction::Lda { address: 0.into() },
				Instruction::Jnz { value: 1.into() },
			])
		)
	}
}
