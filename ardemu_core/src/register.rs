use num_enum::{IntoPrimitive, TryFromPrimitive};
use self_rust_tokenize::SelfRustTokenize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, SelfRustTokenize)]
pub struct Imm16(pub u16);

impl From<u16> for Imm16 {
	fn from(value: u16) -> Self {
		Self(value)
	}
}

impl std::fmt::Display for Imm16 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:#04x}", self.0)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, SelfRustTokenize)]
pub enum HlOrImm16 {
	Hl,
	Imm16(Imm16),
}
impl HlOrImm16 {
	pub fn imm16_or_hl(&self, hl: u16) -> u16 {
		match *self {
			Self::Hl => hl,
			Self::Imm16(imm16) => imm16.0,
		}
	}
}
impl From<u16> for HlOrImm16 {
	fn from(value: u16) -> Self {
		Self::Imm16(Imm16(value))
	}
}
impl std::fmt::Display for HlOrImm16 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Hl => write!(f, "HL"),
			Self::Imm16(imm) => write!(f, "{}", imm),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, SelfRustTokenize)]
pub enum RegisterOrImm8 {
	Register(Register),
	Imm8(u8),
}

impl RegisterOrImm8 {
	pub fn imm8_or_else(&self, else_callback: impl FnOnce(Register) -> u8) -> u8 {
		match *self {
			Self::Imm8(immediate) => immediate,
			Self::Register(reg) => else_callback(reg),
		}
	}
}

impl From<Register> for RegisterOrImm8 {
	fn from(register: Register) -> Self {
		Self::Register(register)
	}
}
impl From<u8> for RegisterOrImm8 {
	fn from(immediate: u8) -> Self {
		Self::Imm8(immediate)
	}
}
impl std::fmt::Display for RegisterOrImm8 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			RegisterOrImm8::Register(r) => write!(f, "{}", r),
			RegisterOrImm8::Imm8(i) => write!(f, "{}", i),
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
	/// GP register
	A = 0,
	/// GP register
	B,
	/// GP register
	C,
	/// GP register
	D,
	/// GP register/(L)ow index register
	L,
	/// GP register/(H)igh index register
	H,
	/// GP register
	Z,
	/// flags (LSB to MSB)
	/// LESS
	/// EQUAL
	/// CARRY
	/// BORROW
	F,
}

impl Register {
	pub const COUNT: usize = 8;
	pub const ALL: &[Register; Self::COUNT] = &[
		Self::A,
		Self::B,
		Self::C,
		Self::D,
		Self::L,
		Self::H,
		Self::Z,
		Self::F,
	];

	/// this will not panic, enforced by the type
	pub fn read_from(&self, registers: &[u8; Self::COUNT]) -> u8 {
		registers[*self as usize]
	}

	/// this will not panic, enforced by the type
	pub fn write_in(&self, registers: &mut [u8; Self::COUNT], value: u8) {
		registers[*self as usize] = value;
	}
}

macro_rules! display_register {
	($($variant:ident),*) => {
		impl std::fmt::Display for Register {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match *self {
					$(Register::$variant => write!(f, "{}", stringify!($variant).to_uppercase()),)*
				}
			}
		}
	};
}
display_register!(A, B, C, D, L, H, Z, F);
