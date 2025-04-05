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

pub fn get_bit_from_u16(value: u16, bit: u16) -> bool {
	value & (1 << bit) != 0
}

pub fn set_bit_in_u16(value: u16, bit: u16, bit_value: bool) -> u16 {
	if bit_value {
		value | (1 << bit)
	} else {
		value & !(1 << bit)
	}
}

/// [low, high]
pub fn u8s_from_u16(value: u16) -> [u8; 2] {
	let low_value = (value & 0x00FF) as u8;
	let high_value = (value >> 8) as u8;
	[low_value, high_value]
}

pub fn u8s_to_u16(low: u8, high: u8) -> u16 {
	(low as u16) | ((high as u16) << 8)
}
