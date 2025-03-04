use ardemu_core::{
	assemble, AsmParseError, AsmParseErrorType, FlagType, Instruction, LowerEvenRegister,
	UpperRegister, WordRegister, R0, R1, R16, R17, R31,
};

#[test]
fn test_assemble_every_instruction() {
	assert_eq!(
		assemble(
			r"begin:
				nop
				break
				jmp begin
				or r1, r1
				ori r16, 0
				eor r1, r1
				com r0
				neg r0
				swap r0
				lsr r0
				ror r0
				asr r0
				mul r1, r0
				ldi r16, 0
				mov r0, r1
				movw r30, r24 ; r31:r30 = r25:r24
				rjmp -1       ; would be a infinitive loop
				push r0
				pop r0
				cp r16, r17
				cpi r16, 1
				cpc r16, r17
				cpse r16, r17
				breq begin
				brne begin
				brlt begin
				call begin    ; would also be a infinitive loop
				ret
				sub r16, r17
				sbc r16, r17
				subi r16, 1
				sbci r16, 1
				sbiw r28, 42
				dec r16
				add r16, r17
				adc r16, r17
				adiw r28, 42
				inc r16
				and r16, r17
				andi r16, 1
				bset 1
				bclr 1
				sbi 0x1F, 0
				cbi 0x1F, 0
				bst r16, 1
				bld r16, 1
				sts 0x042, r0
				lds r0, 0x042
				out 0x042, r0
				in r0, 0x042
				",
		),
		Ok(vec![
			Instruction::Nop {},
			Instruction::Break {},
			Instruction::Jmp { address: 0 },
			Instruction::Or {
				reg_dest: R1,
				reg_read: R1
			},
			Instruction::Ori {
				register: UpperRegister::R16,
				value: 0.into()
			},
			Instruction::Eor {
				reg_dest: R1,
				reg_read: R1
			},
			Instruction::Com { register: R0 },
			Instruction::Neg { register: R0 },
			Instruction::Swap { register: R0 },
			Instruction::Lsr { register: R0 },
			Instruction::Ror { register: R0 },
			Instruction::Asr { register: R0 },
			Instruction::Mul {
				reg_dest: R1,
				reg_read: R0
			},
			Instruction::Ldi {
				register: UpperRegister::R16,
				value: 0.into()
			},
			Instruction::Mov {
				reg_dest: R0,
				reg_read: R1
			},
			Instruction::Movw {
				reg_dest: LowerEvenRegister::R30,
				reg_read: LowerEvenRegister::R24
			},
			Instruction::RJmp { offset: -1 },
			Instruction::Push { register: R0 },
			Instruction::Pop { register: R0 },
			Instruction::Cp {
				reg_dest: R16,
				reg_read: R17
			},
			Instruction::Cpi {
				register: UpperRegister::R16,
				value: 1.into()
			},
			Instruction::Cpc {
				reg_dest: R16,
				reg_read: R17
			},
			Instruction::Cpse {
				reg_dest: R16,
				reg_read: R17
			},
			// update offset, if relative offset to 'begin' changes in the source code
			Instruction::Breq { offset: -24 },
			// update offset, if relative offset to 'begin' changes in the source code
			Instruction::Brne { offset: -25 },
			// update offset, if relative offset to 'begin' changes in the source code
			Instruction::Brlt { offset: -26 },
			Instruction::Call { address: 0 },
			Instruction::Ret {},
			Instruction::Sub {
				reg_dest: R16,
				reg_read: R17
			},
			Instruction::Sbc {
				reg_dest: R16,
				reg_read: R17
			},
			Instruction::Subi {
				register: UpperRegister::R16,
				value: 1.into()
			},
			Instruction::Sbci {
				register: UpperRegister::R16,
				value: 1.into()
			},
			Instruction::Sbiw {
				register: WordRegister::R28,
				value: 42.into()
			},
			Instruction::Dec { register: R16 },
			Instruction::Add {
				reg_dest: R16,
				reg_read: R17
			},
			Instruction::Adc {
				reg_dest: R16,
				reg_read: R17
			},
			Instruction::Adiw {
				register: WordRegister::R28,
				value: 42.into()
			},
			Instruction::Inc { register: R16 },
			Instruction::And {
				reg_dest: R16,
				reg_read: R17
			},
			Instruction::Andi {
				register: UpperRegister::R16,
				value: 1.into()
			},
			Instruction::Bset {
				flag_type: FlagType::Zero // 1
			},
			Instruction::Bclr {
				flag_type: FlagType::Zero // 1
			},
			Instruction::Sbi {
				register_address: R31.into(),
				bit: 0.try_into().unwrap()
			},
			Instruction::Cbi {
				register_address: R31.into(),
				bit: 0.try_into().unwrap()
			},
			Instruction::Bst {
				register: R16,
				bit: 1.try_into().unwrap()
			},
			Instruction::Bld {
				register: R16,
				bit: 1.try_into().unwrap()
			},
			Instruction::Sts {
				address: 0x042.into(),
				register: R0
			},
			Instruction::Lds {
				register: R0,
				address: 0x042.into()
			},
			Instruction::Out {
				address: 0x042.into(),
				register: R0
			},
			Instruction::In {
				register: R0,
				address: 0x042.into()
			},
		])
	)
}

#[test]
fn test_parse_line_number() {
	assert_eq!(
		assemble(
			r"; line 1
			; line 2

			; empty line above...
			invalidinstruction ; on line 3"
		),
		Err(AsmParseError::new(
			AsmParseErrorType::InvalidInstruction("invalidinstruction".to_string()),
			5
		))
	);
}
