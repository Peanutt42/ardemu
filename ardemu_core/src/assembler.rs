use crate::{AsmParseError, AsmParseErrorType, Instruction};
use std::collections::HashMap;

struct Line {
	/// preprocessed line
	instruction: String,
	/// original from source code
	line_number: usize,
	/// parsed label
	label: Option<String>,
}

impl Line {
	/// preprocesses line: removes any comments (';'), seperate label and instruction
	fn new(str: &str, line_number: usize) -> Self {
		let str_without_comments = match str.split_once(';') {
			Some((line, _comment)) => line.trim().to_string(),
			None => str.trim().to_string(),
		};
		let (label, instruction) = match str_without_comments.split_once(':') {
			Some((label, instruction)) => (Some(label.to_string()), instruction.trim().to_string()),
			None => (None, str_without_comments.to_string()),
		};

		Self {
			instruction,
			line_number,
			label,
		}
	}
}

pub trait ParseAsmInstruction: Sized {
	fn parse_asm_instruction(mnemonic: &str, operands: &[&str]) -> Result<Self, AsmParseErrorType>;
}

pub trait AsmOperand: Sized {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType>;
}

impl AsmOperand for i16 {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		parse_number_operand(operand).map(|n| n as i16)
	}
}

/// parses different number formats like "0x123" or "0b10101" and normal "42"
pub fn parse_number_operand(operand: &str) -> Result<i32, AsmParseErrorType> {
	if let Some(operand) = operand.strip_prefix("0x") {
		i32::from_str_radix(operand, 16).map_err(|source| AsmParseErrorType::InvalidNumber {
			string: operand.to_string(),
			source,
		})
	} else if let Some(operand) = operand.strip_prefix("0b") {
		i32::from_str_radix(operand, 2).map_err(|source| AsmParseErrorType::InvalidNumber {
			string: operand.to_string(),
			source,
		})
	} else {
		operand
			.parse::<i32>()
			.map_err(|source| AsmParseErrorType::InvalidNumber {
				string: operand.to_string(),
				source,
			})
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

/// Substitutes a LDA instruction to a program address with a LDA instruction to a symbol, which is later converted back to a LDA instruction to a program address.
// #[derive(Debug, ParseAsmInstruction)]
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

fn parse_single_symbol(operands: &[&str]) -> Result<String, AsmParseErrorType> {
	match operands.len() {
		1 => Ok(operands[0].to_string()),
		_ => Err(AsmParseErrorType::InvalidArgumentCount {
			expected_count: 1,
			actual_count: operands.len(),
		}),
	}
}

/// appends the parsed instruction onto 'output_instruction'
fn parse_instruction(
	line_number: usize,
	mnemonic: &str,
	operands: &[&str],
) -> Result<IntermediateInstruction, AsmParseErrorType> {
	match mnemonic.to_uppercase().as_str() {
		"JMP" => {
			let symbol = parse_single_symbol(operands)?;
			Ok(IntermediateInstruction::Jmp {
				symbol,
				line_number,
			})
		}
		"BREQ" => {
			let symbol = parse_single_symbol(operands)?;
			Ok(IntermediateInstruction::Breq {
				symbol,
				line_number,
			})
		}
		"BRNE" => {
			let symbol = parse_single_symbol(operands)?;
			Ok(IntermediateInstruction::Brne {
				symbol,
				line_number,
			})
		}
		"BRLT" => {
			let symbol = parse_single_symbol(operands)?;
			Ok(IntermediateInstruction::Brlt {
				symbol,
				line_number,
			})
		}
		"CALL" => {
			let symbol = parse_single_symbol(operands)?;
			Ok(IntermediateInstruction::Call {
				symbol,
				line_number,
			})
		}
		_ => Instruction::parse_asm_instruction(mnemonic, operands)
			.map(IntermediateInstruction::Instruction),
	}
}

pub fn assemble(asm: &str) -> Result<Vec<Instruction>, AsmParseError> {
	let lines: Vec<Line> = asm
		.lines()
		.enumerate()
		.map(|(i, line_str)| Line::new(line_str, i + 1))
		.filter(|l| !l.instruction.is_empty() || l.label.is_some())
		.collect();

	// maps symbols to program addresses
	let mut symbol_table: HashMap<String, u16> = HashMap::new();
	let mut intermediate_instructions: Vec<IntermediateInstruction> = Vec::new();
	for line in lines {
		if let Some(label) = &line.label {
			symbol_table.insert(label.clone(), intermediate_instructions.len() as u16);
		}

		let (mnemonic, operands) = split_mnemonic_operands(&line.instruction);
		if mnemonic.is_empty() {
			continue;
		}

		let instruction = parse_instruction(line.line_number, &mnemonic, &operands)
			.map_err(|error| AsmParseError::new(error, line.line_number))?;
		intermediate_instructions.push(instruction);
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
	use crate::Register::R31;
	use crate::{
		assemble, AsmParseError, AsmParseErrorType, FlagType, LowerEvenRegister, UpperRegister,
		WordRegister,
	};
	use crate::{Instruction, R0, R1, R16, R17};

	#[test]
	fn test_assemble_every_instruction() {
		assert_eq!(
			assemble(
				r"begin:
				nop
				break
				jmp begin
				or r1, r1
				ori r16, 0
				eor r1, r1
				com r0
				neg r0
				swap r0
				lsr r0
				ror r0
				asr r0
				mul r1, r0
				ldi r16, 0
				mov r0, r1
				movw r30, r24 ; r31:r30 = r25:r24
				rjmp -1       ; would be a infinitive loop
				push r0
				pop r0
				cp r16, r17
				cpi r16, 1
				cpc r16, r17
				cpse r16, r17
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
				bset 1
				bclr 1
				sbi 0x1F, 0
				cbi 0x1F, 0
				bst r16, 1
				bld r16, 1
				sts 0x042, r0
				lds r0, 0x042
				out 0x042, r0
				in r0, 0x042
				",
			),
			Ok(vec![
				Instruction::Nop {},
				Instruction::Break {},
				Instruction::Jmp { address: 0 },
				Instruction::Or {
					reg_dest: R1,
					reg_read: R1
				},
				Instruction::Ori {
					register: UpperRegister::R16,
					value: 0.into()
				},
				Instruction::Eor {
					reg_dest: R1,
					reg_read: R1
				},
				Instruction::Com { register: R0 },
				Instruction::Neg { register: R0 },
				Instruction::Swap { register: R0 },
				Instruction::Lsr { register: R0 },
				Instruction::Ror { register: R0 },
				Instruction::Asr { register: R0 },
				Instruction::Mul {
					reg_dest: R1,
					reg_read: R0
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
					reg_dest: LowerEvenRegister::R30,
					reg_read: LowerEvenRegister::R24
				},
				Instruction::RJmp { offset: -1 },
				Instruction::Push { register: R0 },
				Instruction::Pop { register: R0 },
				Instruction::Cp {
					reg_dest: R16,
					reg_read: R17
				},
				Instruction::Cpi {
					register: UpperRegister::R16,
					value: 1.into()
				},
				Instruction::Cpc {
					reg_dest: R16,
					reg_read: R17
				},
				Instruction::Cpse {
					reg_dest: R16,
					reg_read: R17
				},
				// update offset, if relative offset to 'begin' changes in the source code
				Instruction::Breq { offset: -24 },
				// update offset, if relative offset to 'begin' changes in the source code
				Instruction::Brne { offset: -25 },
				// update offset, if relative offset to 'begin' changes in the source code
				Instruction::Brlt { offset: -26 },
				Instruction::Call { address: 0 },
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
				Instruction::Bset {
					flag_type: FlagType::Zero // 1
				},
				Instruction::Bclr {
					flag_type: FlagType::Zero // 1
				},
				Instruction::Sbi {
					register_address: R31.into(),
					bit: 0.try_into().unwrap()
				},
				Instruction::Cbi {
					register_address: R31.into(),
					bit: 0.try_into().unwrap()
				},
				Instruction::Bst {
					register: R16,
					bit: 1.try_into().unwrap()
				},
				Instruction::Bld {
					register: R16,
					bit: 1.try_into().unwrap()
				},
				Instruction::Sts {
					address: 0x042.into(),
					register: R0
				},
				Instruction::Lds {
					register: R0,
					address: 0x042.into()
				},
				Instruction::Out {
					address: 0x042.into(),
					register: R0
				},
				Instruction::In {
					register: R0,
					address: 0x042.into()
				},
			])
		)
	}

	#[test]
	fn test_parse_line_number() {
		assert_eq!(
			assemble(
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
