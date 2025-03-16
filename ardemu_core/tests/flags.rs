use ardemu_core::{FlagType, Flags};

#[test]
fn test_add_carry_half_carry_zero() {
	let mut flags = Flags::default();
	flags.set_add_znsvch(0xFF, 0x01, 0x00);
	assert_eq!(
		flags,
		Flags::new(true, false, false, false, true, true, false, false)
	);
}

#[test]
fn test_add_overflow() {
	let mut flags = Flags::default();
	flags.set_add_znsvch(0x7F, 0x01, 0x80);
	assert_eq!(
		flags,
		Flags::new(false, true, false, true, false, true, false, false)
	);
}

#[test]
fn test_sub_zero_result() {
	let mut flags = Flags::default();
	flags.set_sub_znsvch(0x01, 0x01, 0x00);
	assert_eq!(
		flags,
		Flags::new(true, false, false, false, false, false, false, false)
	);
}

#[test]
fn test_sub_borrow() {
	let mut flags = Flags::default();
	flags.set_sub_znsvch(0x00, 0x01, 0xFF);
	assert_eq!(
		flags,
		Flags::new(false, true, true, false, true, true, false, false)
	);
}

#[test]
fn test_lsr_carry_zero() {
	let mut flags = Flags::default();
	flags.set_lsr_znsvc(0x01, 0x00);
	assert_eq!(
		flags,
		Flags::new(true, false, true, true, true, false, false, false)
	);
}

#[test]
fn test_neg_operation() {
	let mut flags = Flags::default();
	flags.set_neg_znsvch(0x01, 0xFF);
	assert_eq!(
		flags,
		Flags::new(false, true, true, false, true, true, false, false)
	);
}

#[test]
fn test_mul_zero_carry() {
	let mut flags = Flags::default();
	flags.set_mul_zc(0x0000);
	assert_eq!(
		flags,
		Flags::new(true, false, false, false, false, false, false, false)
	);
}

#[test]
fn test_mul_carry() {
	let mut flags = Flags::default();
	flags.set_mul_zc(0xFE01);
	assert_eq!(
		flags,
		Flags::new(false, false, false, false, true, false, false, false)
	);
}

#[test]
fn test_add_16bit_overflow() {
	let mut flags = Flags::default();
	flags.set_add_znsvc16(0x7FFF, 0x8000);
	assert_eq!(
		flags,
		Flags::new(false, true, false, true, false, false, false, false)
	);
}

#[test]
fn test_sub_16bit_carry() {
	let mut flags = Flags::default();
	flags.set_sub_znsvc16(0x0000, 0xFFFF);
	assert_eq!(
		flags,
		Flags::new(false, true, true, false, true, false, false, false)
	);
}

#[test]
fn test_sub_rznsvch_non_zero() {
	let mut flags = Flags::default();
	flags.set(FlagType::Zero);
	flags.set_sub_rznsvch(0x02, 0x01, 0x01);
	assert_eq!(
		flags,
		Flags::new(false, false, false, false, false, false, false, false)
	);
}
