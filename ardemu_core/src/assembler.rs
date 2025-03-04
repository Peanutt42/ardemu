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
				word_address: address.into(),
			}),
			Self::Call {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Call {
				word_address: address.into(),
			}),
			Self::Breq {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Breq {
				word_offset: (address as i32 - program_address as i32) as i8 - 1,
			}),
			Self::Brne {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Brne {
				word_offset: (address as i32 - program_address as i32) as i8 - 1,
			}),
			Self::Brlt {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Brlt {
				word_offset: (address as i32 - program_address as i32) as i8 - 1,
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
