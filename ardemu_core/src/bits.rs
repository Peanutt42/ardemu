pub fn get_bit_from_u8(value: u8, bit: u8) -> bool {
	value & (1 << bit) != 0
}

pub fn set_bit_in_u8(value: u8, bit: u8, bit_value: bool) -> u8 {
	if bit_value {
		value | (1 << bit)
	} else {
		value & !(1 << bit)
	}
}

/// [low, high]
pub fn u8s_from_u16(value: u16) -> [u8; 2] {
	let low_value = value as u8;
	let high_value = (value >> 8) as u8;
	[low_value, high_value]
}

pub fn u8s_to_u16(low: u8, high: u8) -> u16 {
	(low as u16) | ((high as u16) << 8)
}

#[cfg(test)]
mod test {
	use crate::{get_bit_from_u8, set_bit_in_u8, u8s_from_u16, u8s_to_u16};

	#[test]
	fn test_u16_u8s_conversion() {
		let value_16 = 60000;
		let [low, high] = u8s_from_u16(value_16);
		assert_eq!(value_16, u8s_to_u16(low, high));
	}

	#[test]
	fn test_bit_manipulation() {
		let value: u8 = 0b10101010;
		assert!(!get_bit_from_u8(value, 0));
		assert!(get_bit_from_u8(value, 1));
		assert!(!get_bit_from_u8(value, 2));
		assert!(get_bit_from_u8(value, 3));
		assert!(!get_bit_from_u8(value, 4));
		assert!(get_bit_from_u8(value, 5));
		assert!(!get_bit_from_u8(value, 6));
		assert!(get_bit_from_u8(value, 7));

		assert_eq!(set_bit_in_u8(value, 0, true), 0b10101011);
		assert_eq!(set_bit_in_u8(value, 1, false), 0b10101000);
		assert_eq!(set_bit_in_u8(value, 2, true), 0b10101110);
		assert_eq!(set_bit_in_u8(value, 3, false), 0b10100010);
		assert_eq!(set_bit_in_u8(value, 4, true), 0b10111010);
		assert_eq!(set_bit_in_u8(value, 5, false), 0b10001010);
		assert_eq!(set_bit_in_u8(value, 6, true), 0b11101010);
		assert_eq!(set_bit_in_u8(value, 7, false), 0b00101010);
	}
}
