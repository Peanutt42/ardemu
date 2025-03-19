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
				parse_number_operand::<$primitive_type>(operand).map(|n| Self(n))
			}
		}
	};
}

macro_rules! word_type {
	($type_name:ident, $inner_type:ident, $format:expr) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
		pub struct $type_name(pub $inner_type);

		impl From<$inner_type> for $type_name {
			fn from(value: $inner_type) -> Self {
				Self(value)
			}
		}

		impl std::fmt::Display for $type_name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, "{}", $format(self.0))
			}
		}

		impl AsmOperand for $type_name {
			fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
				parse_number_operand::<$inner_type>(operand).map(|n| Self(n))
			}
		}
	};
}
macro_rules! impl_asm_operand_for_subregister {
	($type:ty) => {
		impl AsmOperand for $type {
			fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
				let reg = Register::parse_operand(operand)?;
				Self::try_from(reg)
					.map_err(|_| AsmParseErrorType::InvalidRegister(operand.to_string()))
			}
		}
	};
}

immediate_type!(Imm16, u16, ":#06x");
immediate_type!(Imm8, u8, ":#04x");

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
impl std::fmt::Display for Imm3 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}
impl AsmOperand for Imm3 {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		let num = parse_number_operand::<u8>(operand)
			.map_err(|_| AsmParseErrorType::ExpectedBitLocation(operand.to_string()))?;
		Self::try_from(num).map_err(|_| AsmParseErrorType::ExpectedBitLocation(operand.to_string()))
	}
}

word_type!(WordAddress, u32, |this: u32| format!(
	"{:#06X}",
	this as u64 * 2
));
word_type!(WordOffset16, i16, |this: i16| format!(
	".{:+}",
	(this as i32) * 2
));
word_type!(WordOffset8, i8, |this: i8| format!(
	".{:+}",
	(this as i16) * 2
));

impl AddAssign<u32> for WordAddress {
	fn add_assign(&mut self, rhs: u32) {
		self.0 += rhs;
	}
}
impl AddAssign<u16> for WordAddress {
	fn add_assign(&mut self, rhs: u16) {
		self.0 += rhs as u32;
	}
}
impl AddAssign<u8> for WordAddress {
	fn add_assign(&mut self, rhs: u8) {
		self.0 += rhs as u32;
	}
}
impl AddAssign<i32> for WordAddress {
	fn add_assign(&mut self, rhs: i32) {
		*self = Self(self.0.saturating_add_signed(rhs));
	}
}
impl<T: Into<u32>> Add<T> for WordAddress {
	type Output = Self;
	fn add(self, rhs: T) -> Self {
		Self(self.0 + rhs.into())
	}
}
impl Sub<WordAddress> for WordAddress {
	type Output = Self;
	fn sub(self, rhs: Self) -> Self {
		Self(self.0 - rhs.0)
	}
}
impl From<u16> for WordAddress {
	fn from(value: u16) -> Self {
		Self(value as u32)
	}
}
impl WordAddress {
	pub fn wrapping_add_signed(&self, rhs: impl Into<WordOffset16>) -> Self {
		Self(self.0.wrapping_add_signed(rhs.into().0 as i32))
	}
}
impl From<i32> for WordOffset16 {
	fn from(value: i32) -> Self {
		Self(value as i16)
	}
}
impl From<WordOffset16> for i32 {
	fn from(value: WordOffset16) -> Self {
		value.0 as i32
	}
}
impl From<WordOffset8> for WordOffset16 {
	fn from(value: WordOffset8) -> Self {
		Self(value.0 as i16)
	}
}

macro_rules! define_register {
	($($variant:ident),+) => {
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
		pub enum Register { $($variant),+ }

		impl Register {
			pub const COUNT: usize = 32;
			pub const ALL: &[Self; Self::COUNT] = &[$(Self::$variant),+];
		}
	};
}

macro_rules! display_register {
	($type:ident) => {
		impl std::fmt::Display for $type {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, "{}", format!("{:?}", self).to_lowercase())
			}
		}
	};
}

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
macro_rules! register_subset {
	($name:ident: $base:ident => $($variant:ident),+) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Ord, IntoPrimitive, TryFromPrimitive, SelfRustTokenize)]
		#[repr(u8)]
		pub enum $name { $($variant = $base::$variant as u8),+ }

		map_register!($name, $base, $($variant),*);
	};
}

define_register!(
	R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20,
	R21, R22, R23, R24, R25, R26, R27, R28, R29, R30, R31
);

impl Register {
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
				.map_err(|_| AsmParseErrorType::InvalidRegister(operand.to_string())),
			None => Err(AsmParseErrorType::InvalidRegister(operand.to_string())),
		}
	}
}

display_register!(Register);

register_subset!(UpperRegister: Register => R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26, R27, R28, R29, R30, R31);
display_register!(UpperRegister);
impl_asm_operand_for_subregister!(UpperRegister);

register_subset!(WordRegister: Register => R24, R26, R28, R30);
map_register!(WordRegister, LowerEvenRegister, R24, R26, R28, R30);
impl_asm_operand_for_subregister!(WordRegister);
impl WordRegister {
	pub fn get_higher_uneven_register(self) -> Register {
		let lower_even_register: LowerEvenRegister = self.into();
		lower_even_register.get_higher_uneven_register()
	}
}
impl std::fmt::Display for WordRegister {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let lower: u8 = (*self).into();
		write!(f, "r{}:r{}", lower + 1, lower)
	}
}

register_subset!(LowerEvenRegister: Register => R0, R2, R4, R6, R8, R10, R12, R14, R16, R18, R20, R22, R24, R26, R28, R30);
impl_asm_operand_for_subregister!(LowerEvenRegister);

impl std::fmt::Display for LowerEvenRegister {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let lower: u8 = (*self).into();
		write!(f, "r{}:r{}", lower + 1, lower)
	}
}
impl LowerEvenRegister {
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

// RegisterAddress and other remaining types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub struct RegisterAddress(pub Register);

impl From<Register> for RegisterAddress {
	fn from(value: Register) -> Self {
		Self(value)
	}
}
impl From<RegisterAddress> for Register {
	fn from(value: RegisterAddress) -> Self {
		value.0
	}
}
impl TryFrom<u8> for RegisterAddress {
	type Error = ();
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match Register::try_from(value) {
			Ok(register) => Ok(Self(register)),
			Err(_) => Err(()),
		}
	}
}
impl std::fmt::Display for RegisterAddress {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:#04X}", self.0 as u8)
	}
}
impl AsmOperand for RegisterAddress {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		let address = parse_number_operand::<u8>(operand)?;
		Register::try_from(address)
			.map(Self)
			.map_err(|_| AsmParseErrorType::InvalidRegisterIoAddress(operand.to_string()))
	}
}
