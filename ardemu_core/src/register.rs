use crate::{parse_number_operand, u8s_from_u16, u8s_to_u16, AsmOperand, AsmParseErrorType};
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
	pub fn to_pair(self) -> RegisterPair16 {
		match self {
			Self::R24 => RegisterPair16 {
				high: Register::R25,
				low: Register::R24,
			},
			Self::R26 => RegisterPair16 {
				high: Register::R27,
				low: Register::R26,
			},
			Self::R28 => RegisterPair16 {
				high: Register::R29,
				low: Register::R28,
			},
			Self::R30 => RegisterPair16 {
				high: Register::R31,
				low: Register::R30,
			},
		}
	}
}
display_register!(WordRegister, R24, R26, R28, R30);
map_register!(WordRegister, Register, R24, R26, R28, R30);
impl AsmOperand for WordRegister {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		let register = Register::parse_operand(operand)?;
		WordRegister::try_from(register)
			.map_err(|_| AsmParseErrorType::InvalidRegister(operand.to_string()))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub struct RegisterAddress(pub Register);

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

/// combines value of two 8 bit registers into a 16 bit value
/// high_register must always be low_register + 1, as the values must be stored continuously
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub struct RegisterPair16 {
	pub high: Register,
	pub low: Register,
}
impl RegisterPair16 {
	pub const R1R0: Self = Self {
		high: Register::R1,
		low: Register::R0,
	};

	/// only fails if register is the low register is the last register,
	/// as there is no register after it to store the high value
	pub fn new(low: Register) -> Option<Self> {
		let high = Register::try_from(low as u8 + 1).ok()?;
		Some(Self { high, low })
	}

	/// this will not panic, enforced by the type
	pub fn read_from(&self, registers: &[u8; Register::COUNT]) -> u16 {
		u8s_to_u16(registers[self.low as usize], registers[self.high as usize])
	}

	/// this will not panic, enforced by the type
	pub fn write_in(&self, registers: &mut [u8; Register::COUNT], value: u16) {
		let [low_value, high_value] = u8s_from_u16(value);
		registers[self.high as usize] = high_value;
		registers[self.low as usize] = low_value;
	}
}
impl AsmOperand for RegisterPair16 {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		let low_register = Register::parse_operand(operand)?;
		RegisterPair16::new(low_register).ok_or(AsmParseErrorType::InvalidRegisterPairLowRegister)
	}
}
impl std::fmt::Display for RegisterPair16 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}", self.high, self.low)
	}
}
