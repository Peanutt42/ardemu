use crate::{
	AsmParseError, AsmParseErrorType, FlagType, Instruction, Opcode, Program, WordAddress,
};
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
#[derive(Debug)]
enum IntermediateInstruction {
	Jmp { symbol: String, line_number: usize },
	Call { symbol: String, line_number: usize },
	Breq { symbol: String, line_number: usize },
	Brne { symbol: String, line_number: usize },
	Brlt { symbol: String, line_number: usize },
	Brcs { symbol: String, line_number: usize },
	Brcc { symbol: String, line_number: usize },
	Instruction(Instruction),
}
impl IntermediateInstruction {
	fn resolve_into_instruction(
		self,
		program_address: WordAddress,
		symbol_table: &HashMap<String, WordAddress>,
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
			} => resolve_symbol(symbol, line_number)
				.map(|word_address| Instruction::Jmp { word_address }),
			Self::Call {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number)
				.map(|word_address| Instruction::Call { word_address }),
			Self::Breq {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Breq {
				word_offset: ((address.0 as i32 - program_address.0 as i32) as i8).into(),
			}),
			Self::Brne {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Brne {
				word_offset: ((address.0 as i32 - program_address.0 as i32) as i8).into(),
			}),
			Self::Brlt {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Brlt {
				word_offset: ((address.0 as i32 - program_address.0 as i32) as i8).into(),
			}),
			Self::Brcs {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Brcs {
				word_offset: ((address.0 as i32 - program_address.0 as i32) as i8).into(),
			}),
			Self::Brcc {
				symbol,
				line_number,
			} => resolve_symbol(symbol, line_number).map(|address| Instruction::Brcc {
				word_offset: ((address.0 as i32 - program_address.0 as i32) as i8).into(),
			}),
			Self::Instruction(instruction) => Ok(instruction),
		}
	}

	fn get_word_size(&self) -> u8 {
		match self {
			Self::Instruction(instruction) => instruction.get_word_size(),
			Self::Breq { .. } => 1, // see of Instruction::Breq::get_word_size()
			Self::Brne { .. } => 1, // see of Instruction::Brne::get_word_size()
			Self::Brlt { .. } => 1, // see of Instruction::Brlt::get_word_size()
			Self::Brcs { .. } => 1, // see of Instruction::Brlt::get_word_size()
			Self::Brcc { .. } => 1, // see of Instruction::Brlt::get_word_size()
			Self::Call { .. } => 2, // see of Instruction::Call::get_word_size()
			Self::Jmp { .. } => 2,  // see of Instruction::Jmp::get_word_size()
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
		"BRCS" => {
			let symbol = parse_single_symbol(operands)?;
			Ok(IntermediateInstruction::Brcs {
				symbol,
				line_number,
			})
		}
		"BRCC" => {
			let symbol = parse_single_symbol(operands)?;
			Ok(IntermediateInstruction::Brcc {
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
		"CLI" => {
			if operands.is_empty() {
				Ok(IntermediateInstruction::Instruction(Instruction::Bclr {
					flag_type: FlagType::Interrupt,
				}))
			} else {
				Err(AsmParseErrorType::InvalidArgumentCount {
					expected_count: 0,
					actual_count: operands.len(),
				})
			}
		}
		_ => Instruction::parse_asm_instruction(mnemonic, operands)
			.map(IntermediateInstruction::Instruction),
	}
}

/// Assembles the given assembly code into a program, including debug symbols (names of labels).
pub fn assemble(asm: &str) -> Result<Program, AsmParseError> {
	let lines: Vec<Line> = asm
		.lines()
		.enumerate()
		.map(|(i, line_str)| Line::new(line_str, i + 1))
		.filter(|l| !l.instruction.is_empty() || l.label.is_some())
		.collect();

	// maps symbols to program addresses
	let mut symbol_table: HashMap<String, WordAddress> = HashMap::new();
	let mut program_address = WordAddress(0);
	let mut intermediate_instructions: Vec<(WordAddress, IntermediateInstruction)> = Vec::new();
	for line in lines {
		if let Some(label) = &line.label {
			symbol_table.insert(label.clone(), program_address);
		}

		let (mnemonic, operands) = split_mnemonic_operands(&line.instruction);
		if mnemonic.is_empty() {
			continue;
		}

		let instruction = parse_instruction(line.line_number, &mnemonic, &operands)
			.map_err(|error| AsmParseError::new(error, line.line_number))?;
		program_address += instruction.get_word_size() as u16;
		intermediate_instructions.push((program_address, instruction));
	}

	let mut instructions: Vec<Instruction> = Vec::with_capacity(intermediate_instructions.len());
	for (program_address, intermediate_instruction) in intermediate_instructions.into_iter() {
		let instruction =
			intermediate_instruction.resolve_into_instruction(program_address, &symbol_table)?;
		instructions.push(instruction);
	}

	let debug_symbol_table = symbol_table
		.into_iter()
		.map(|(name, address)| (address, name))
		.collect::<HashMap<WordAddress, String>>();

	Ok(Program::with_debug_symbols(
		&instructions,
		debug_symbol_table,
	))
}
