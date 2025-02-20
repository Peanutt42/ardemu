use num_enum::{IntoPrimitive, TryFromPrimitive};
use self_rust_tokenize::SelfRustTokenize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub struct Imm8(pub u8);

impl From<u8> for Imm8 {
	fn from(value: u8) -> Self {
		Self(value)
	}
}

impl std::fmt::Display for Imm8 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:#04x}", self.0)
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

/// combines value of two 8 bit registers into a 16 bit value
/// high_register must always be low_register + 1, as the values must be stored continuously
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub struct RegisterPair16 {
	pub high: Register,
	pub low: Register,
}

impl RegisterPair16 {
	/// only fails if register is the low register is the last register,
	/// as there is no register after it to store the high value
	pub fn new(low: Register) -> Option<Self> {
		let high = Register::try_from(low as u8 + 1).ok()?;
		Some(Self { high, low })
	}

	/// [low, high]
	pub(crate) fn u8s_from_u16(value: u16) -> [u8; 2] {
		let low_value = value as u8;
		let high_value = (value >> 8) as u8;
		[low_value, high_value]
	}

	pub(crate) fn u8s_to_u16(low: u8, high: u8) -> u16 {
		(low as u16) | (high as u16) << 8
	}

	/// this will not panic, enforced by the type
	pub fn read_from(&self, registers: &[u8; Register::COUNT]) -> u16 {
		Self::u8s_to_u16(registers[self.low as usize], registers[self.high as usize])
	}

	/// this will not panic, enforced by the type
	pub fn write_in(&self, registers: &mut [u8; Register::COUNT], value: u16) {
		let [low_value, high_value] = Self::u8s_from_u16(value);
		registers[self.high as usize] = high_value;
		registers[self.low as usize] = low_value;
	}
}

impl std::fmt::Display for RegisterPair16 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}", self.high, self.low)
	}
}

#[cfg(test)]
mod test {
	use crate::RegisterPair16;

	#[test]
	fn test_u16_u8s_conversion() {
		let value_16 = 60000;
		let [low, high] = RegisterPair16::u8s_from_u16(value_16);
		assert_eq!(value_16, RegisterPair16::u8s_to_u16(low, high));
	}
}
