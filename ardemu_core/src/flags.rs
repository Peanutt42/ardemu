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
	// TODO: add T: Bit Copy, I: Interrupt enabled
}

impl Flags {
	pub fn zero(&self) -> bool {
		self.zero
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
}
