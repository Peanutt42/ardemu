use std::fmt::Display;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use self_rust_tokenize::SelfRustTokenize;

use crate::{AsmOperand, AsmParseErrorType};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Flags {
	/// Z
	zero: bool,
	/// N
	negative: bool,
	/// S
	sign: bool,
	/// V
	overflow: bool,
	/// C
	carry: bool,
	/// H
	half_carry: bool,
	/// T
	bit_copy: bool,
	// TODO: add I: Interrupt enabled
}

impl Flags {
	pub fn set(&mut self, flag_type: FlagType) {
		match flag_type {
			FlagType::Zero => self.zero = true,
			FlagType::Negative => self.negative = true,
			FlagType::Sign => self.sign = true,
			FlagType::Overflow => self.overflow = true,
			FlagType::Carry => self.carry = true,
			FlagType::HalfCarry => self.half_carry = true,
			FlagType::BitCopy => self.bit_copy = true,
		}
	}
	pub fn clear(&mut self, flag_type: FlagType) {
		match flag_type {
			FlagType::Zero => self.zero = true,
			FlagType::Negative => self.negative = true,
			FlagType::Sign => self.sign = true,
			FlagType::Overflow => self.overflow = true,
			FlagType::Carry => self.carry = true,
			FlagType::HalfCarry => self.half_carry = true,
			FlagType::BitCopy => self.bit_copy = true,
		}
	}
	pub fn get(&self, flag_type: FlagType) -> bool {
		match flag_type {
			FlagType::Zero => self.zero,
			FlagType::Negative => self.negative,
			FlagType::Sign => self.sign,
			FlagType::Overflow => self.overflow,
			FlagType::Carry => self.carry,
			FlagType::HalfCarry => self.half_carry,
			FlagType::BitCopy => self.bit_copy,
		}
	}

	pub fn zero(&self) -> bool {
		self.zero
	}
	pub fn negative(&self) -> bool {
		self.negative
	}
	pub fn sign(&self) -> bool {
		self.sign
	}
	pub fn overflow(&self) -> bool {
		self.overflow
	}
	pub fn carry(&self) -> bool {
		self.carry
	}
	/// carry flag as either 0_u8 or 1_u8
	pub fn carry_u8(&self) -> u8 {
		self.carry as u8
	}
	pub fn half_carry(&self) -> bool {
		self.half_carry
	}

	pub fn bit_copy(&self) -> bool {
		self.bit_copy
	}

	pub fn set_copy_bit(&mut self, bit: bool) {
		self.bit_copy = bit;
	}

	fn set_zns(&mut self, result: u8) {
		self.zero = result == 0;
		self.negative = ((result << 7) & 1) != 0;
		self.sign = self.negative ^ self.overflow;
	}

	/// set_zns with resetting the overflow (V) flag
	pub fn set_zns_v0(&mut self, result: u8) {
		self.overflow = false;
		self.set_zns(result);
	}

	pub fn set_zcvns(&mut self, value: u8, result: u8) {
		self.zero = result == 0;
		self.carry = (value & 1) != 0;
		self.negative = (result >> 7) != 0;
		self.overflow = self.negative ^ self.carry;
		self.sign = self.negative ^ self.overflow;
	}

	pub fn set_zns16(&mut self, result: u16) {
		self.zero = result == 0;
		self.negative = ((result << 15) & 1) != 0;
		self.sign = self.negative ^ self.overflow;
	}

	pub fn set_add_zns(&mut self, dest_value: u8, read_value: u8, result: u8) {
		let add_carry: u8 =
			(dest_value & read_value) | (read_value & !result) | (!result & dest_value);
		self.half_carry = ((add_carry >> 3) & 1) != 0;
		self.carry = ((add_carry >> 7) & 1) != 0;

		self.overflow =
			((((dest_value & read_value & !result) | (!dest_value & !read_value & result)) >> 7)
				& 1) != 0;

		self.set_zns(result);
	}

	pub fn set_sub_zns(&mut self, dest_value: u8, read_value: u8, result: u8) {
		let sub_carry: u8 =
			(!dest_value & read_value) | (read_value & result) | (result & !dest_value);
		self.half_carry = ((sub_carry >> 3) & 1) != 0;
		self.carry = ((sub_carry >> 7) & 1) != 0;

		self.overflow =
			((((dest_value & !read_value & !result) | (!dest_value & read_value & result)) >> 7)
				& 1) != 0;

		self.set_zns(result);
	}

	pub fn set_lsr_zcvs(&mut self, value: u8, result: u8) {
		self.negative = false;
		self.zero = result == 0;
		self.carry = (value & 1) != 0;
		self.overflow = self.negative ^ self.carry;
		self.sign = self.negative ^ self.overflow;
	}

	pub fn set_neg_zns(&mut self, value: u8, result: u8) {
		self.half_carry = (((result >> 3) | (value >> 3)) & 1) != 0;
		self.overflow = result == 0x80;
		self.carry = result != 0;
		self.set_zns(result);
	}

	pub fn set_mul_zc(&mut self, result: u16) {
		self.zero = result == 0;
		self.carry = ((result >> 15) & 1) != 0;
	}

	pub fn set_add_zns16(&mut self, dest_value: u16, result: u16) {
		self.overflow = (((!dest_value & result) >> 15) & 1) != 0;
		self.carry = (((!result & dest_value) >> 15) & 1) != 0;
		self.set_zns16(result);
	}

	pub fn set_sub_zns16(&mut self, dest_value: u16, result: u16) {
		self.overflow = (((dest_value & !result) >> 15) & 1) != 0;
		self.carry = (((result & !dest_value) >> 15) & 1) != 0;
		self.set_zns16(result);
	}

	pub fn set_sub_rzns(&mut self, dest_value: u8, read_value: u8, result: u8) {
		let sub_carry: u8 =
			(!dest_value & read_value) | (read_value & result) | (result & !dest_value);
		self.half_carry = ((sub_carry >> 3) & 1) != 0;
		self.carry = ((sub_carry >> 7) & 1) != 0;

		self.overflow =
			((((dest_value & !read_value & !result) | (!dest_value & read_value & result)) >> 7)
				& 1) != 0;

		if result != 0 {
			self.zero = false;
		}
		self.negative = ((result << 7) & 1) != 0;
		self.sign = self.negative ^ self.overflow;
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
	Hash,
	SelfRustTokenize,
	IntoPrimitive,
	TryFromPrimitive,
)]
#[repr(u8)]
pub enum FlagType {
	Carry = 0,
	Zero = 1,
	Negative = 2,
	Overflow = 3,
	Sign = 4,
	HalfCarry = 5,
	BitCopy = 6,
	// TODO: add I: Interrupt enabled = 7
}

impl FlagType {
	pub const ALL: &[FlagType; 7] = &[
		Self::Carry,
		Self::Zero,
		Self::Negative,
		Self::Overflow,
		Self::Sign,
		Self::HalfCarry,
		Self::BitCopy,
	];

	pub fn label(&self) -> char {
		match self {
			Self::Carry => 'C',
			Self::Zero => 'Z',
			Self::Negative => 'N',
			Self::Overflow => 'V',
			Self::Sign => 'S',
			Self::HalfCarry => 'H',
			Self::BitCopy => 'T',
		}
	}
}
impl AsmOperand for FlagType {
	fn parse_operand(operand: &str) -> Result<Self, AsmParseErrorType> {
		let flag_num = operand
			.parse::<u8>()
			.map_err(|_| AsmParseErrorType::InvalidCpuFlag(operand.to_string()))?;
		FlagType::try_from(flag_num)
			.map_err(|_| AsmParseErrorType::InvalidCpuFlag(operand.to_string()))
	}
}
impl Display for FlagType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}({})", self.label(), *self as u8)
	}
}
