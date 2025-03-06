use std::ops::{Add, AddAssign, Sub};

use crate::{parse_number_operand, AsmOperand, AsmParseErrorType};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use self_rust_tokenize::SelfRustTokenize;

macro_rules! immediate_type {
	($type_name:ident, $primitive_type:ident, $format_str:literal) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
		pub struct $type_name(pub $primitive_type);

		impl From<$primitive_type> for $type_name {
			fn from(value: $primitive_type) -> Self {
				Self(value)
			}
		}
		impl std::fmt::Display for $type_name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, concat!(concat!("{", $format_str), "}"), self.0)
			}
		}
		impl AsmOperand for $type_name {
			fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
				parse_number_operand(operand).map(|n| $type_name(n as $primitive_type))
			}
		}
	};
}

immediate_type!(Imm16, u16, ":#06x");
immediate_type!(Imm8, u8, ":#04x");

/// 3-bit value: 0-7
/// used for setting a specific bit in byte
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub struct Imm3(pub u8);

impl TryFrom<u8> for Imm3 {
	type Error = ();

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		if value < 8 {
			Ok(Self(value))
		} else {
			Err(())
		}
	}
}
impl AsmOperand for Imm3 {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		let bit_num = operand
			.parse::<u8>()
			.map_err(|_| AsmParseErrorType::ExpectedBitLocation(operand.to_string()))?;
		Imm3::try_from(bit_num)
			.map_err(|_| AsmParseErrorType::ExpectedBitLocation(operand.to_string()))
	}
}
impl std::fmt::Display for Imm3 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// stores address in words (1 word = 2 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub struct WordAddress(pub u32);

impl From<u32> for WordAddress {
	fn from(value: u32) -> Self {
		Self(value)
	}
}
impl From<u16> for WordAddress {
	fn from(value: u16) -> Self {
		Self(value as u32)
	}
}
impl Add<u16> for WordAddress {
	type Output = WordAddress;

	fn add(self, rhs: u16) -> Self::Output {
		WordAddress(self.0.add(rhs as u32))
	}
}
impl Sub<WordAddress> for WordAddress {
	type Output = WordAddress;

	fn sub(self, rhs: WordAddress) -> WordAddress {
		WordAddress(self.0 - rhs.0)
	}
}
impl AddAssign<u16> for WordAddress {
	fn add_assign(&mut self, rhs: u16) {
		self.0.add_assign(rhs as u32);
	}
}
impl AddAssign<u8> for WordAddress {
	fn add_assign(&mut self, rhs: u8) {
		self.0.add_assign(rhs as u32);
	}
}
impl AddAssign<i32> for WordAddress {
	fn add_assign(&mut self, rhs: i32) {
		*self = Self(self.0.saturating_add_signed(rhs));
	}
}
impl std::fmt::Display for WordAddress {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		//					address is displayed in bytes, 1 word = 2 bytes
		write!(f, "{:#06X}", self.0 * 2)
	}
}
impl AsmOperand for WordAddress {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		parse_number_operand(operand).map(|n| WordAddress(n as u32))
	}
}

/// stores offset in words (1 word = 2 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub struct WordOffset16(pub i16);

impl From<i16> for WordOffset16 {
	fn from(value: i16) -> Self {
		Self(value)
	}
}
impl std::fmt::Display for WordOffset16 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		//					offset is displayed in bytes, 1 word = 2 bytes
		write!(f, ".{:+}", self.0 * 2)
	}
}
impl AsmOperand for WordOffset16 {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		parse_number_operand(operand).map(|n| WordOffset16(n as i16))
	}
}

/// stores offset in words (1 word = 2 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub struct WordOffset8(pub i8);

impl From<i8> for WordOffset8 {
	fn from(value: i8) -> Self {
		Self(value)
	}
}
impl std::fmt::Display for WordOffset8 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		//					offset is displayed in bytes, 1 word = 2 bytes
		write!(f, ".{:+}", self.0 * 2)
	}
}
impl AsmOperand for WordOffset8 {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		parse_number_operand(operand).map(|n| WordOffset8(n as i8))
	}
}

macro_rules! display_register {
	($type:ident, $($variant:ident),*) => {
		impl std::fmt::Display for $type {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match *self {
					$($type::$variant => write!(f, "{}", stringify!($variant).to_lowercase()),)*
				}
			}
		}
	};
}

/// General purpose registers
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Hash,
	Ord,
	IntoPrimitive,
	TryFromPrimitive,
	SelfRustTokenize,
)]
#[repr(u8)]
pub enum Register {
	R0,
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
	pub const COUNT: usize = 32;
	pub const ALL: &[Register; Self::COUNT] = &[
		Self::R0,
		Self::R1,
		Self::R2,
		Self::R3,
		Self::R4,
		Self::R5,
		Self::R6,
		Self::R7,
		Self::R8,
		Self::R9,
		Self::R10,
		Self::R11,
		Self::R12,
		Self::R13,
		Self::R14,
		Self::R15,
		Self::R16,
		Self::R17,
		Self::R18,
		Self::R19,
		Self::R20,
		Self::R21,
		Self::R22,
		Self::R23,
		Self::R24,
		Self::R25,
		Self::R26,
		Self::R27,
		Self::R28,
		Self::R29,
		Self::R30,
		Self::R31,
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
impl AsmOperand for Register {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		match operand.strip_prefix("r") {
			Some(s) => s
				.parse::<u8>()
				.map_err(|_| ())
				.and_then(|reg_index| Register::try_from(reg_index).map_err(|_| ()))
				.map_err(|_| AsmParseErrorType::InvalidRegister(s.to_string())),
			None => Err(AsmParseErrorType::InvalidRegister(operand.to_string())),
		}
	}
}
display_register!(
	Register, R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18,
	R19, R20, R21, R22, R23, R24, R25, R26, R27, R28, R29, R30, R31
);

macro_rules! map_register {
	($src_type:ident, $dst_type:ident, $($variant:ident),*) => {
		impl From<$src_type> for $dst_type {
			fn from(value: $src_type) -> Self {
				match value {
					$($src_type::$variant => $dst_type::$variant,)*
				}
			}
		}
		impl TryFrom<$dst_type> for $src_type {
			type Error = ();

			fn try_from(value: $dst_type) -> Result<Self, Self::Error> {
				match value {
					$($dst_type::$variant => Ok($src_type::$variant),)*
					_ => Err(()),
				}
			}
		}
	};
}

/// Upper register (R16-R31)
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Hash,
	Ord,
	IntoPrimitive,
	TryFromPrimitive,
	SelfRustTokenize,
)]
#[repr(u8)]
pub enum UpperRegister {
	R16 = 16,
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
display_register!(
	UpperRegister,
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
	R31
);
map_register!(
	UpperRegister,
	Register,
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
	R31
);
impl AsmOperand for UpperRegister {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		let register = Register::parse_operand(operand)?;
		UpperRegister::try_from(register)
			.map_err(|_| AsmParseErrorType::InvalidRegister(operand.to_string()))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub struct RegisterAddress(pub Register);

impl From<RegisterAddress> for Register {
	fn from(value: RegisterAddress) -> Self {
		value.0
	}
}
impl From<Register> for RegisterAddress {
	fn from(value: Register) -> Self {
		Self(value)
	}
}
impl TryFrom<u8> for RegisterAddress {
	type Error = ();

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match Register::try_from(value) {
			Ok(reg) => Ok(Self(reg)),
			Err(_) => Err(()),
		}
	}
}
impl AsmOperand for RegisterAddress {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		let address = parse_number_operand(operand)? as u8;
		RegisterAddress::try_from(address)
			.map_err(|_| AsmParseErrorType::InvalidRegisterIoAddress(operand.to_string()))
	}
}
impl std::fmt::Display for RegisterAddress {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:#04X}", self.0 as u8)
	}
}

macro_rules! display_register_pair {
	($type:ident, $($variant:ident),*) => {
		impl std::fmt::Display for $type {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match *self {
					$($type::$variant => write!(f, "{}:{}", $type::$variant.get_higher_uneven_register(), stringify!($variant).to_lowercase()),)*
				}
			}
		}
	};
}

/// ensures that the register is any of:
/// R24, R26, R28, R30
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Hash,
	Ord,
	IntoPrimitive,
	TryFromPrimitive,
	SelfRustTokenize,
)]
#[repr(u8)]
pub enum WordRegister {
	R24,
	R26,
	R28,
	R30,
}
impl WordRegister {
	pub fn get_higher_uneven_register(self) -> Register {
		let lower_even_register: LowerEvenRegister = self.into();
		lower_even_register.get_higher_uneven_register()
	}
}
display_register_pair!(WordRegister, R24, R26, R28, R30);
map_register!(WordRegister, Register, R24, R26, R28, R30);
map_register!(WordRegister, LowerEvenRegister, R24, R26, R28, R30);
impl AsmOperand for WordRegister {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		let register = Register::parse_operand(operand)?;
		WordRegister::try_from(register)
			.map_err(|_| AsmParseErrorType::InvalidRegister(operand.to_string()))
	}
}

/// Even register (R0, R2, R4, .., R30)
/// used for register pairs where even register is lower register, uneven is higher register
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Hash,
	Ord,
	IntoPrimitive,
	TryFromPrimitive,
	SelfRustTokenize,
)]
#[repr(u8)]
pub enum LowerEvenRegister {
	R0 = 0,
	R2 = 2,
	R4 = 4,
	R6 = 6,
	R8 = 8,
	R10 = 10,
	R12 = 12,
	R14 = 14,
	R16 = 16,
	R18 = 18,
	R20 = 20,
	R22 = 22,
	R24 = 24,
	R26 = 26,
	R28 = 28,
	R30 = 30,
}
impl LowerEvenRegister {
	/// returns the register+1
	pub fn get_higher_uneven_register(&self) -> Register {
		match self {
			Self::R0 => Register::R1,
			Self::R2 => Register::R3,
			Self::R4 => Register::R5,
			Self::R6 => Register::R7,
			Self::R8 => Register::R9,
			Self::R10 => Register::R11,
			Self::R12 => Register::R13,
			Self::R14 => Register::R15,
			Self::R16 => Register::R17,
			Self::R18 => Register::R19,
			Self::R20 => Register::R21,
			Self::R22 => Register::R23,
			Self::R24 => Register::R25,
			Self::R26 => Register::R27,
			Self::R28 => Register::R29,
			Self::R30 => Register::R31,
		}
	}
}
display_register_pair!(
	LowerEvenRegister,
	R0,
	R2,
	R4,
	R6,
	R8,
	R10,
	R12,
	R14,
	R16,
	R18,
	R20,
	R22,
	R24,
	R26,
	R28,
	R30
);
map_register!(
	LowerEvenRegister,
	Register,
	R0,
	R2,
	R4,
	R6,
	R8,
	R10,
	R12,
	R14,
	R16,
	R18,
	R20,
	R22,
	R24,
	R26,
	R28,
	R30
);
impl AsmOperand for LowerEvenRegister {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		let register = Register::parse_operand(operand)?;
		LowerEvenRegister::try_from(register)
			.map_err(|_| AsmParseErrorType::InvalidRegister(operand.to_string()))
	}
}
