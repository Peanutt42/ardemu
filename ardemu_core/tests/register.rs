use ardemu_core::{AsmOperand, AsmParseErrorType, PointerRegister};

#[test]
fn test_pointer_register_asm_parsing() {
	fn test(pointer_register: PointerRegister, operand: &str) {
		assert_eq!(
			Ok(pointer_register),
			PointerRegister::parse_operand(operand)
		);
	}

	test(PointerRegister::X, "X");
	test(PointerRegister::X_PRE_DEC, "-X");
	test(PointerRegister::X_POST_INC, "X+");

	test(PointerRegister::Y, "Y");
	test(PointerRegister::Y_PRE_DEC, "-Y");
	test(PointerRegister::Y_POST_INC, "Y+");

	test(PointerRegister::Z, "Z");
	test(PointerRegister::Z_PRE_DEC, "-Z");
	test(PointerRegister::Z_POST_INC, "Z+");

	assert_eq!(
		Err(AsmParseErrorType::InvalidPointerRegister("-X+".to_string())),
		PointerRegister::parse_operand("-X+")
	);
}
