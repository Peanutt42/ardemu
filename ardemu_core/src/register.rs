use num_enum::{IntoPrimitive, TryFromPrimitive};
use self_rust_tokenize::SelfRustTokenize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, SelfRustTokenize)]
pub enum RegisterOrImmediate {
	Register(Register),
	Immediate(u8),
}

impl RegisterOrImmediate {
	pub fn immediate_or_else(&self, else_callback: impl FnOnce(Register) -> u8) -> u8 {
		match *self {
			Self::Immediate(immediate) => immediate,
			Self::Register(reg) => else_callback(reg),
		}
	}
}

impl From<Register> for RegisterOrImmediate {
	fn from(register: Register) -> Self {
		Self::Register(register)
	}
}
impl From<u8> for RegisterOrImmediate {
	fn from(immediate: u8) -> Self {
		Self::Immediate(immediate)
	}
}
impl std::fmt::Display for RegisterOrImmediate {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			RegisterOrImmediate::Register(r) => write!(f, "{}", r),
			RegisterOrImmediate::Immediate(i) => write!(f, "{}", i),
		}
	}
}

#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	IntoPrimitive,
	TryFromPrimitive,
	SelfRustTokenize,
)]
#[repr(u8)]
pub enum Register {
	R0 = 0,
	R1,
	R2,
	R3,
	R4,
	R5,
	R6,
	R7,
	R8,
	R9,
	R10,
	R11,
	R12,
	R13,
	R14,
	R15,
	R16,
	R17,
	R18,
	R19,
	R20,
	R21,
	R22,
	R23,
	R24,
	R25,
	R26,
	R27,
	R28,
	R29,
	R30,
	R31,
}

impl Register {
	/// this will not fail, as there are only 32 registers possible, enforced by the type
	pub fn get_from(&self, registers: &[u8; 32]) -> u8 {
		registers[*self as usize]
	}

	/// this will not fail, as there are only 32 registers possible, enforced by the type
	pub fn set_in(&self, registers: &mut [u8; 32], value: u8) {
		registers[*self as usize] = value;
	}
}

/// displays registers as 'r0', 'r1', etc.
macro_rules! display_register {
	($($variant:ident),*) => {
		impl std::fmt::Display for Register {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match *self {
					$(Register::$variant => write!(f, "r{}", Register::$variant as u8),)*
				}
			}
		}
	};
}
display_register!(
	R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20,
	R21, R22, R23, R24, R25, R26, R27, R28, R29, R30, R31
);
