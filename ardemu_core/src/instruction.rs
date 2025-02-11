use crate::Register;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
	// Move immediate value to register
	Ldi { reg: Register, value: u8 },
	// Add two registers (store result in first)
	Add { rd: Register, rs: Register },
	// Jump to relative address
	Jmp { offset: i32 },
	// Stores value from register to address
	Store { reg: Register, addr: usize },
	// No operation
	Nop,
}

impl std::fmt::Debug for Instruction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Instruction::Ldi { reg, value } => write!(f, "LDI {reg}, {value:#04x}"),
			Instruction::Add { rd, rs } => write!(f, "ADD {rd}, {rs}"),
			Instruction::Jmp { offset } => write!(f, "JMP {offset}"),
			Instruction::Store { reg, addr } => write!(f, "STORE {reg}, {addr:#04x}"),
			Instruction::Nop => write!(f, "NOP"),
		}
	}
}
