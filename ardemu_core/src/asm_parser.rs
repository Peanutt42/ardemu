use crate::{
	register::{RegisterPair16, UpperRegister},
	AsmParseError, AsmParseErrorType, Instruction, Register, WordRegister,
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
fn parse_number(s: &str) -> Result<i32, AsmParseErrorType> {
	if let Some(s) = s.strip_prefix("0x") {
		i32::from_str_radix(s, 16).map_err(|source| AsmParseErrorType::InvalidNumber {
			string: s.to_string(),
			source,
		})
	} else if let Some(s) = s.strip_prefix("0b") {
		i32::from_str_radix(s, 2).map_err(|source| AsmParseErrorType::InvalidNumber {
			string: s.to_string(),
			source,
		})
	} else {
		s.parse::<i32>()
			.map_err(|source| AsmParseErrorType::InvalidNumber {
				string: s.to_string(),
				source,
			})
	}
}

fn parse_register(s: &str) -> Result<Register, AsmParseErrorType> {
	match s.strip_prefix("r") {
		Some(s) => s
			.parse::<u8>()
			.map_err(|_| ())
			.and_then(|reg_index| Register::try_from(reg_index).map_err(|_| ()))
			.map_err(|_| AsmParseErrorType::InvalidRegister(s.to_string())),
		None => Err(AsmParseErrorType::InvalidRegister(s.to_string())),
	}
}

fn parse_upper_register(s: &str) -> Result<UpperRegister, AsmParseErrorType> {
	let register = parse_register(s)?;
	UpperRegister::try_from(register)
		.map_err(|_| AsmParseErrorType::ExpectedUpperRegister(register))
}

fn parse_word_register(s: &str) -> Result<WordRegister, AsmParseErrorType> {
	let register = parse_register(s)?;
	WordRegister::try_from(register).map_err(|_| AsmParseErrorType::ExpectedWordRegister(register))
}

fn parse_register_pair(s: &str) -> Result<RegisterPair16, AsmParseErrorType> {
	let low_register = parse_register(s)?;
	RegisterPair16::new(low_register).ok_or(AsmParseErrorType::InvalidRegisterPairLowRegister)
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
	Jmp { symbol: String, line_number: usize },
	Call { symbol: String, line_number: usize },
	Breq { symbol: String, line_number: usize },
	Brne { symbol: String, line_number: usize },
	Brlt { symbol: String, line_number: usize },
	Instruction(Instruction),
}
impl IntermediateInstruction {
	fn resolve_into_instruction(
		self,
		program_address: u16,
		symbol_table: &HashMap<String, u16>,
	) -> Result<Instruction, AsmParseError> {
		let resolve_symbol = |symbol: String, line_number: usize| {
			symbol_table.get(&symbol).copied().ok_or(AsmParseError::new(
				AsmParseErrorType::UndefinedLabel(symbol),
				line_number,
			))
		};

		match self {
			Self::Jmp {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Jmp {
				address: address.into(),
			}),
			Self::Call {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Call {
				address: address.into(),
			}),
			Self::Breq {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Breq {
				offset: (address as i32 - program_address as i32) as i8 - 1,
			}),
			Self::Brne {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Brne {
				offset: (address as i32 - program_address as i32) as i8 - 1,
			}),
			Self::Brlt {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Brlt {
				offset: (address as i32 - program_address as i32) as i8 - 1,
			}),
			Self::Instruction(instruction) => Ok(instruction),
		}
	}
}
impl From<Instruction> for IntermediateInstruction {
	fn from(instruction: Instruction) -> Self {
		IntermediateInstruction::Instruction(instruction)
	}
}

/// appends the parsed instruction onto 'output_instruction'
fn parse_instruction(
	line_number: usize,
	mnemonic: &str,
	operands: &[&str],
	output_instruction: &mut Vec<IntermediateInstruction>,
) -> Result<(), AsmParseErrorType> {
	match mnemonic.to_uppercase().as_str() {
		"JMP" => {
			let operands = consume_operands::<1>(operands)?;
			let symbol = operands[0].to_string();
			output_instruction.push(IntermediateInstruction::Jmp {
				symbol,
				line_number,
			});
			Ok(())
		}
		"EOR" => {
			let operands = consume_operands::<2>(operands)?;
			let reg_dest = parse_register(operands[0])?;
			let reg_read = parse_register(operands[1])?;
			output_instruction.push(Instruction::Eor { reg_dest, reg_read }.into());
			Ok(())
		}
		"LDI" => {
			let operands = consume_operands::<2>(operands)?;
			let register = parse_upper_register(operands[0])?;
			let value = parse_number(operands[1])? as u8;
			output_instruction.push(
				Instruction::Ldi {
					register,
					value: value.into(),
				}
				.into(),
			);
			Ok(())
		}
		"MOV" => {
			let operands = consume_operands::<2>(operands)?;
			let reg_dest = parse_register(operands[0])?;
			let reg_read = parse_register(operands[1])?;
			output_instruction.push(Instruction::Mov { reg_dest, reg_read }.into());
			Ok(())
		}
		"MOVW" => {
			let operands = consume_operands::<2>(operands)?;
			let reg_dest = parse_register_pair(operands[0])?;
			let reg_read = parse_register_pair(operands[1])?;
			output_instruction.push(Instruction::Movw { reg_dest, reg_read }.into());
			Ok(())
		}
		"RJMP" => {
			let operands = consume_operands::<1>(operands)?;
			let offset = parse_number(operands[0])? as i16;
			output_instruction.push(Instruction::RJmp { offset }.into());
			Ok(())
		}
		"PUSH" => {
			let operands = consume_operands::<1>(operands)?;
			let register = parse_register(operands[0])?;
			output_instruction.push(Instruction::Push { register }.into());
			Ok(())
		}
		"POP" => {
			let operands = consume_operands::<1>(operands)?;
			let register = parse_register(operands[0])?;
			output_instruction.push(Instruction::Pop { register }.into());
			Ok(())
		}
		"CPI" => {
			let operands = consume_operands::<2>(operands)?;
			let register = parse_upper_register(operands[0])?;
			let value = parse_number(operands[1])? as u8;
			output_instruction.push(
				Instruction::Cpi {
					register,
					value: value.into(),
				}
				.into(),
			);
			Ok(())
		}
		"CPC" => {
			let operands = consume_operands::<2>(operands)?;
			let reg_dest = parse_register(operands[0])?;
			let reg_read = parse_register(operands[1])?;
			output_instruction.push(Instruction::Cpc { reg_dest, reg_read }.into());
			Ok(())
		}
		"BREQ" => {
			let operands = consume_operands::<1>(operands)?;
			let symbol = operands[0].to_string();
			output_instruction.push(IntermediateInstruction::Breq {
				symbol,
				line_number,
			});
			Ok(())
		}
		"BRNE" => {
			let operands = consume_operands::<1>(operands)?;
			let symbol = operands[0].to_string();
			output_instruction.push(IntermediateInstruction::Brne {
				symbol,
				line_number,
			});
			Ok(())
		}
		"BRLT" => {
			let operands = consume_operands::<1>(operands)?;
			let symbol = operands[0].to_string();
			output_instruction.push(IntermediateInstruction::Brlt {
				symbol,
				line_number,
			});
			Ok(())
		}
		"CALL" => {
			let operands = consume_operands::<1>(operands)?;
			let symbol = operands[0].to_string();
			output_instruction.push(IntermediateInstruction::Call {
				symbol,
				line_number,
			});
			Ok(())
		}
		"RET" => {
			output_instruction.push(Instruction::Ret {}.into());
			Ok(())
		}
		"SUB" => {
			let operands = consume_operands::<2>(operands)?;
			let reg_dest = parse_register(operands[0])?;
			let reg_read = parse_register(operands[1])?;
			output_instruction.push(Instruction::Sub { reg_dest, reg_read }.into());
			Ok(())
		}
		"SBC" => {
			let operands = consume_operands::<2>(operands)?;
			let reg_dest = parse_register(operands[0])?;
			let reg_read = parse_register(operands[1])?;
			output_instruction.push(Instruction::Sbc { reg_dest, reg_read }.into());
			Ok(())
		}
		"SUBI" => {
			let operands = consume_operands::<2>(operands)?;
			let register = parse_upper_register(operands[0])?;
			let value = parse_number(operands[1])? as u8;
			output_instruction.push(
				Instruction::Subi {
					register,
					value: value.into(),
				}
				.into(),
			);
			Ok(())
		}
		"SBCI" => {
			let operands = consume_operands::<2>(operands)?;
			let register = parse_upper_register(operands[0])?;
			let value = parse_number(operands[1])? as u8;
			output_instruction.push(
				Instruction::Sbci {
					register,
					value: value.into(),
				}
				.into(),
			);
			Ok(())
		}
		"SBIW" => {
			let operands = consume_operands::<2>(operands)?;
			let register = parse_word_register(operands[0])?;
			let value = parse_number(operands[1])? as u16;
			output_instruction.push(
				Instruction::Sbiw {
					register,
					value: value.into(),
				}
				.into(),
			);
			Ok(())
		}
		"DEC" => {
			let operands = consume_operands::<1>(operands)?;
			let register = parse_register(operands[0])?;
			output_instruction.push(Instruction::Dec { register }.into());
			Ok(())
		}
		"ADD" => {
			let operands = consume_operands::<2>(operands)?;
			let reg_dest = parse_register(operands[0])?;
			let reg_read = parse_register(operands[1])?;
			output_instruction.push(Instruction::Add { reg_dest, reg_read }.into());
			Ok(())
		}
		"ADC" => {
			let operands = consume_operands::<2>(operands)?;
			let reg_dest = parse_register(operands[0])?;
			let reg_read = parse_register(operands[1])?;
			output_instruction.push(Instruction::Adc { reg_dest, reg_read }.into());
			Ok(())
		}
		"ADIW" => {
			let operands = consume_operands::<2>(operands)?;
			let register = parse_word_register(operands[0])?;
			let value = parse_number(operands[1])? as u16;
			output_instruction.push(
				Instruction::Adiw {
					register,
					value: value.into(),
				}
				.into(),
			);
			Ok(())
		}
		"INC" => {
			let operands = consume_operands::<1>(operands)?;
			let register = parse_register(operands[0])?;
			output_instruction.push(Instruction::Inc { register }.into());
			Ok(())
		}
		"AND" => {
			let operands = consume_operands::<2>(operands)?;
			let reg_dest = parse_register(operands[0])?;
			let reg_read = parse_register(operands[1])?;
			output_instruction.push(Instruction::And { reg_dest, reg_read }.into());
			Ok(())
		}
		"ANDI" => {
			let operands = consume_operands::<2>(operands)?;
			let register = parse_upper_register(operands[0])?;
			let value = parse_number(operands[1])? as u8;
			output_instruction.push(
				Instruction::Andi {
					register,
					value: value.into(),
				}
				.into(),
			);
			Ok(())
		}
		"STS" => {
			let operands = consume_operands::<2>(operands)?;
			let address = parse_number(operands[0])? as u16;
			let register = parse_register(operands[1])?;
			output_instruction.push(
				Instruction::Sts {
					address: address.into(),
					register,
				}
				.into(),
			);
			Ok(())
		}
		"LDS" => {
			let operands = consume_operands::<2>(operands)?;
			let register = parse_register(operands[0])?;
			let address = parse_number(operands[1])? as u16;
			output_instruction.push(
				Instruction::Lds {
					address: address.into(),
					register,
				}
				.into(),
			);
			Ok(())
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
	for (program_address, intermediate_instruction) in
		intermediate_instructions.into_iter().enumerate()
	{
		instructions.push(
			intermediate_instruction
				.resolve_into_instruction(program_address as u16, &symbol_table)?,
		);
	}

	Ok(instructions)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
	use crate::{parse_asm, AsmParseError, AsmParseErrorType, UpperRegister, WordRegister};
	use crate::{Instruction, RegisterPair16, R0, R1, R16, R17, R24, R30};

	#[test]
	fn test_parse_asm() {
		assert_eq!(
			parse_asm(
				r"begin:
				jmp begin
				eor r1, r1
				ldi r16, 0
				mov r0, r1
				movw r30, r24 ; r31:r30 = r25:r24
				rjmp -1       ; would be a infinitive loop
				push r0
				pop r0
				cpi r16, 1
				cpc r16, r17
				breq begin
				brne begin
				brlt begin
				call begin    ; would also be a infinitive loop
				ret
				sub r16, r17
				sbc r16, r17
				subi r16, 1
				sbci r16, 1
				sbiw r28, 42
				dec r16
				add r16, r17
				adc r16, r17
				adiw r28, 42
				inc r16
				and r16, r17
				andi r16, 1
				sts 0x042, r0
				lds r0, 0x042
				",
			),
			Ok(vec![
				Instruction::Jmp { address: 0.into() },
				Instruction::Eor {
					reg_dest: R1,
					reg_read: R1
				},
				Instruction::Ldi {
					register: UpperRegister::R16,
					value: 0.into()
				},
				Instruction::Mov {
					reg_dest: R0,
					reg_read: R1
				},
				Instruction::Movw {
					reg_dest: RegisterPair16::new(R30).unwrap(),
					reg_read: RegisterPair16::new(R24).unwrap()
				},
				Instruction::RJmp { offset: -1 },
				Instruction::Push { register: R0 },
				Instruction::Pop { register: R0 },
				Instruction::Cpi {
					register: UpperRegister::R16,
					value: 1.into()
				},
				Instruction::Cpc {
					reg_dest: R16,
					reg_read: R17
				},
				// update offset, if relative offset to 'begin' changes in the source code
				Instruction::Breq { offset: -11 },
				// update offset, if relative offset to 'begin' changes in the source code
				Instruction::Brne { offset: -12 },
				// update offset, if relative offset to 'begin' changes in the source code
				Instruction::Brlt { offset: -13 },
				Instruction::Call { address: 0.into() },
				Instruction::Ret {},
				Instruction::Sub {
					reg_dest: R16,
					reg_read: R17
				},
				Instruction::Sbc {
					reg_dest: R16,
					reg_read: R17
				},
				Instruction::Subi {
					register: UpperRegister::R16,
					value: 1.into()
				},
				Instruction::Sbci {
					register: UpperRegister::R16,
					value: 1.into()
				},
				Instruction::Sbiw {
					register: WordRegister::R28,
					value: 42.into()
				},
				Instruction::Dec { register: R16 },
				Instruction::Add {
					reg_dest: R16,
					reg_read: R17
				},
				Instruction::Adc {
					reg_dest: R16,
					reg_read: R17
				},
				Instruction::Adiw {
					register: WordRegister::R28,
					value: 42.into()
				},
				Instruction::Inc { register: R16 },
				Instruction::And {
					reg_dest: R16,
					reg_read: R17
				},
				Instruction::Andi {
					register: UpperRegister::R16,
					value: 1.into()
				},
				Instruction::Sts {
					address: 0x042.into(),
					register: R0
				},
				Instruction::Lds {
					register: R0,
					address: 0x042.into()
				},
			])
		)
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
