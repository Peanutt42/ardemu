use crate::{register::RegisterOrImmediate, Register};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
	/// moves value into register
	Move {
		reg: Register,
		value: RegisterOrImmediate,
	},
	/// jumps relative to current instruction
	Jmp { offset: i32 },
	/// add value to register
	Add {
		reg: Register,
		value: RegisterOrImmediate,
	},
	/// store value in memory at address
	Store {
		value: RegisterOrImmediate,
		addr: usize,
	},
}

impl std::fmt::Debug for Instruction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Instruction::Move { reg, value } => write!(f, "MOVE {reg}, {value}"),
			Instruction::Add { reg, value } => write!(f, "ADD {reg}, {value}"),
			Instruction::Jmp { offset } => write!(f, "JMP {offset}"),
			Instruction::Store { value, addr } => write!(f, "STORE {value}, {addr:#04x}"),
		}
	}
}
