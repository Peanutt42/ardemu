use crate::{
	Imm16, Imm3, Imm8, Instruction, LowerEvenRegister, Register, RegisterAddress, UpperRegister,
	WordRegister,
};

pub trait Opcode: Sized {
	fn is_32bit(&self) -> bool;
	fn get_byte_size(&self) -> u8;
	fn get_cycles(&self) -> u8;
	fn load(opcode_32bit: u32) -> Option<Self>;
}

impl Opcode for Instruction {
	fn is_32bit(&self) -> bool {
		matches!(
			self,
			Instruction::Sts { .. }
				| Instruction::Lds { .. }
				| Instruction::Jmp { .. }
				| Instruction::Call { .. }
		)
	}

	fn get_byte_size(&self) -> u8 {
		if self.is_32bit() {
			4
		} else {
			2
		}
	}

	/// TODO: implement edge cases where instruction doesnt follow this pattern
	fn get_cycles(&self) -> u8 {
		if self.is_32bit() {
			2
		} else {
			1
		}
	}

	fn load(opcode_32bit: u32) -> Option<Self> {
		let opcode_16bit = ((opcode_32bit & 0xffff0000) >> 16) as u16;
		let front_4_bits = opcode_16bit >> 12;

		/// ____ __rd dddd rrrr
		fn load_rd5_rr5(opcode_16bit: u16) -> Option<(Register, Register)> {
			let d5 = ((opcode_16bit >> 4) & 0x1f) as u8;
			let rd5 = Register::try_from(d5).ok()?;
			let r5 = (((opcode_16bit >> 5) & 0x10) | (opcode_16bit & 0xf)) as u8;
			let rr5 = Register::try_from(r5).ok()?;
			Some((rd5, rr5))
		}

		/// ____ kkkk dddd kkkk
		fn load_rd4_k8(opcode_16bit: u16) -> Option<(UpperRegister, Imm8)> {
			let d4 = 16 + ((opcode_16bit >> 4) & 0xf) as u8;
			let rd4 = UpperRegister::try_from(d4).ok()?;
			let k8 = (((opcode_16bit & 0x0f00) >> 4) | (opcode_16bit & 0xf)) as u8;
			Some((rd4, k8.into()))
		}

		/// ____ _aar rrrr aaaa
		fn load_rr5_a6(opcode_16bit: u16) -> Option<(Register, Imm8)> {
			let r5 = ((opcode_16bit & 0x1f0) >> 4) as u8;
			let rr5 = Register::try_from(r5).ok()?;
			let a6 = (((opcode_16bit & 0x0600) >> 5) | (opcode_16bit & 0xf)) as u8;
			Some((rr5, a6.into()))
		}

		/// ____ kkkk kkkk kkkk
		fn load_k12(opcode_16bit: u16) -> i16 {
			let k12 = opcode_16bit & 0x0fff;
			let negative = (k12 & 0x0800) != 0;
			if negative {
				(k12 | 0xF000) as i16
			} else {
				k12 as i16
			}
		}

		/// ____ ___r rrrr ____
		fn load_rr(opcode_16bit: u16) -> Option<Register> {
			let r5 = ((opcode_16bit & 0x1f0) >> 4) as u8;
			Register::try_from(r5).ok()
		}

		/// ____ ____ aaaa abbb
		fn load_a5_b3(opcode_16bit: u16) -> Option<(RegisterAddress, Imm3)> {
			let a5 = ((opcode_16bit & 0x00f8) >> 3) as u8;
			let b3 = (opcode_16bit & 0x0007) as u8;
			Some((
				RegisterAddress::try_from(a5).ok()?,
				Imm3::try_from(b3).ok()?,
			))
		}

		/// ____ ____ kkdd kkkk
		fn load_rd2_k6(opcode_16bit: u16) -> Option<(WordRegister, Imm8)> {
			let rd2 = ((opcode_16bit & 0x0030) >> 4) as u8;
			let k6 = (((opcode_16bit & 0x00c0) >> 2) | (opcode_16bit & 0x000f)) as u8;
			Some((WordRegister::try_from(rd2).ok()?, k6.into()))
		}

		/// ____ ___d dddd ____ kkkk kkkk kkkk kkkk   (32-bit!)
		fn load_rr_k16(opcode_16bit: u16, opcode_32bit: u32) -> Option<(Register, Imm16)> {
			let rr = ((opcode_16bit & 0x01f0) >> 4) as u8;
			let k16 = (opcode_32bit & 0x0000ffff) as u16;
			Some((Register::try_from(rr).ok()?, k16.into()))
		}

		/// ____ ___k kkkk ___k kkkk kkkk kkkk kkkk    (32-bit!)
		fn load_k24(opcode_32bit: u32) -> u32 {
			((opcode_32bit & 0x01f00000) >> 3) | (opcode_32bit & 0x0001ffff)
		}

		/// ____ ____ dddd rrrr
		fn load_rd4_rr4(opcode_16bit: u16) -> Option<(LowerEvenRegister, LowerEvenRegister)> {
			let d4 = ((opcode_16bit & 0x00f0) >> 4) as u8;
			let r4 = (opcode_16bit & 0x000f) as u8;
			Some((
				LowerEvenRegister::try_from(d4 * 2).ok()?,
				LowerEvenRegister::try_from(r4 * 2).ok()?,
			))
		}

		/// ____ __kk kkkk k___
		fn load_k7(opcode_16bit: u16) -> i8 {
			let k7 = (opcode_16bit & 0x03f8) >> 3;
			let negative = (k7 & 0x40) != 0;
			if negative {
				(k7 | 0x80) as i8
			} else {
				k7 as i8
			}
		}

		match front_4_bits {
			//				first 2 bits after first 4 bits
			0b0000 => match (opcode_16bit & 0x0c00) >> 10 {
				//				last 2 bits after first 2 bits after first 4 bits
				0b00 => match (opcode_16bit & 0x0300) >> 8 {
					0b00 => Some(Instruction::Nop),
					0b01 => {
						let (reg_dest, reg_read) = load_rd4_rr4(opcode_16bit)?;
						Some(Instruction::Movw { reg_dest, reg_read })
					}
					_ => None,
				},
				0b10 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Sbc { reg_dest, reg_read })
				}
				0b01 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Cpc { reg_dest, reg_read })
				}
				_ => None,
			},
			//				first 2 bits after front_4_bits
			0b0010 => match (opcode_16bit & 0x0c00) >> 10 {
				0b10 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Or { reg_dest, reg_read })
				}
				0b01 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Eor { reg_dest, reg_read })
				}
				0b11 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Mov { reg_dest, reg_read })
				}
				_ => None,
			},
			0b0011 => {
				let (register, value) = load_rd4_k8(opcode_16bit)?;
				Some(Instruction::Cpi { register, value })
			}
			0b0110 => {
				let (register, value) = load_rd4_k8(opcode_16bit)?;
				Some(Instruction::Ori { register, value })
			}
			0b1011 => {
				let (register, address) = load_rr5_a6(opcode_16bit)?;
				Some(Instruction::Out { address, register })
			}
			0b0100 => {
				let (register, value) = load_rd4_k8(opcode_16bit)?;
				Some(Instruction::Sbci { register, value })
			}
			// 3 bits after front_4_bits
			0b1001 => match (opcode_16bit & 0xe00) >> 9 {
				0b000 => Some(Instruction::Pop {
					register: load_rr(opcode_16bit)?,
				}),
				// "first" 4 bits of opcode
				0b001 => match opcode_16bit & 0x000f {
					0b0000 => {
						let (register, address) = load_rr_k16(opcode_16bit, opcode_32bit)?;
						Some(Instruction::Sts { address, register })
					}
					0b1111 => Some(Instruction::Push {
						register: load_rr(opcode_16bit)?,
					}),
					_ => None,
				},
				//			first 3 bits of the 4 bits at the right end of the opcode
				0b010 => match (opcode_16bit & 0x000e) >> 1 {
					// 				last bit (lsb)
					0b000 => match opcode_16bit & 0x0001 {
						0b0 => Some(Instruction::Com {
							register: load_rr(opcode_16bit)?,
						}),
						0b1 => Some(Instruction::Neg {
							register: load_rr(opcode_16bit)?,
						}),
						_ => None,
					},
					// 				4 bits with 4 bit offset to right end of opcode
					0b100 => match (opcode_16bit & 0x00f0) >> 4 {
						0b0000 => Some(Instruction::Ret),
						0b1001 => Some(Instruction::Break),
						_ => None,
					},
					0b011 => match opcode_16bit & 0x0001 {
						0b0 => Some(Instruction::Lsr {
							register: load_rr(opcode_16bit)?,
						}),
						0b1 => Some(Instruction::Ror {
							register: load_rr(opcode_16bit)?,
						}),
						_ => None,
					},
					0b001 => Some(Instruction::Swap {
						register: load_rr(opcode_16bit)?,
					}),
					0b110 => Some(Instruction::Jmp {
						address: load_k24(opcode_32bit),
					}),
					0b010 => Some(Instruction::Asr {
						register: load_rr(opcode_16bit)?,
					}),
					0b111 => Some(Instruction::Call {
						address: load_k24(opcode_32bit),
					}),
					_ => None,
				},
				0b101 => {
					let (register_address, bit) = load_a5_b3(opcode_16bit)?;
					Some(Instruction::Sbi {
						register_address,
						bit,
					})
				}
				0b011 => {
					let (register, value) = load_rd2_k6(opcode_16bit)?;
					Some(Instruction::Sbiw { register, value })
				}
				0b110 | 0b111 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Mul { reg_dest, reg_read })
				}
				_ => None,
			},
			//				first 2 bits after front_4_bits
			0b0001 => match (opcode_16bit & 0x0c00) >> 10 {
				0b10 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Sub { reg_dest, reg_read })
				}
				0b01 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Cp { reg_dest, reg_read })
				}
				0b00 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Cpse { reg_dest, reg_read })
				}
				_ => None,
			},
			0b1100 => Some(Instruction::RJmp {
				offset: load_k12(opcode_16bit),
			}),
			0b0101 => {
				let (register, value) = load_rd4_k8(opcode_16bit)?;
				Some(Instruction::Subi { register, value })
			}
			0b1110 => {
				let (register, value) = load_rd4_k8(opcode_16bit)?;
				Some(Instruction::Ldi { register, value })
			}
			//				first 2 bits after front_4_bits
			0b1111 => match (opcode_16bit & 0x0c00) >> 10 {
				//				3 bits at then end of opcode_16bit
				0b00 => match opcode_16bit & 0x0007 {
					0b001 => {
						let offset = load_k7(opcode_16bit);
						Some(Instruction::Breq { offset })
					}
					0b100 => {
						let offset = load_k7(opcode_16bit);
						Some(Instruction::Brlt { offset })
					}
					_ => None,
				},
				0b01 => {
					let offset = load_k7(opcode_16bit);
					Some(Instruction::Brne { offset })
				}
				_ => None,
			},
			_ => None,
		}
	}
}

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
mod tests {
	use crate::{
		Instruction, LowerEvenRegister, Opcode,
		Register::{R0, R15, R16, R31},
		UpperRegister, WordRegister,
	};

	#[test]
	fn test_load_instructions_from_opcodes() {
		fn test_16bit(opcode_16bit: u16, expected_instruction: Instruction) {
			let instruction =
				Instruction::load((opcode_16bit as u32) << 16).expect("failed to load opcode");

			assert_eq!(
				instruction, expected_instruction,
				"16-bit Opcode {opcode_16bit:016b} should be {expected_instruction}, but got {instruction}"
			);
		}

		fn test_32bit(opcode_32bit: u32, expected_instruction: Instruction) {
			let instruction = Instruction::load(opcode_32bit).expect("failed to load opcode");

			assert_eq!(
				instruction, expected_instruction,
				"32-bit Opcode {opcode_32bit:032b} should be {expected_instruction}, but got {instruction}"
			);
		}

		test_16bit(0b0000_0000_0000_0000, Instruction::Nop);
		test_16bit(
			0b0010_1001_1111_1111,
			Instruction::Or {
				reg_dest: R31,
				reg_read: R15,
			},
		);
		test_16bit(
			0b0110_0010_0000_1010,
			Instruction::Ori {
				register: UpperRegister::R16,
				// 42
				value: 0b10_1010.into(),
			},
		);
		test_16bit(
			0b1011_1110_1111_1111,
			Instruction::Out {
				address: 0b11_1111.into(),
				register: R15,
			},
		);
		test_16bit(0b1001_0001_0000_1111, Instruction::Pop { register: R16 });
		test_16bit(0b1001_0011_0000_1111, Instruction::Push { register: R16 });
		test_16bit(0b1001_0101_0000_1000, Instruction::Ret);
		test_16bit(
			0b1100_0000_0000_0000 | (u16::from_le_bytes((-2047_i16).to_le_bytes()) & 0x0fff),
			Instruction::RJmp { offset: -2047 },
		);
		test_16bit(0b1001_0101_1111_0111, Instruction::Ror { register: R31 });
		test_16bit(
			0b0000_1001_1111_1111,
			Instruction::Sbc {
				reg_dest: R31,
				reg_read: R15,
			},
		);
		test_16bit(
			0b0100_0010_0000_1010,
			Instruction::Sbci {
				register: UpperRegister::R16,
				value: 42.into(),
			},
		);
		test_16bit(
			0b1001_1010_0111_1100,
			Instruction::Sbi {
				register_address: R15.into(),
				bit: 4.try_into().unwrap(),
			},
		);
		test_16bit(
			0b1001_0111_1100_1111,
			Instruction::Sbiw {
				register: WordRegister::R24,
				value: 63.into(),
			},
		);
		test_32bit(
			0b1001_0011_1111_0000_1000_0000_0000_0000,
			Instruction::Sts {
				address: 32768.into(),
				register: R31,
			},
		);
		test_16bit(
			0b0001_1011_1111_0000,
			Instruction::Sub {
				reg_dest: R31,
				reg_read: R16,
			},
		);
		test_16bit(
			0b0101_1000_0000_0000,
			Instruction::Subi {
				register: UpperRegister::R16,
				value: 0b1000_0000.into(),
			},
		);
		test_16bit(0b1001_0101_1111_0010, Instruction::Swap { register: R31 });
		test_16bit(0b1001_0101_1001_1000, Instruction::Break);
		test_32bit(
			0b1001_0100_0000_1100_1111_1111_1111_1111,
			Instruction::Jmp { address: 0xFFFF },
		);
		test_16bit(
			0b0010_0111_1111_0000,
			Instruction::Eor {
				reg_dest: R31,
				reg_read: R16,
			},
		);
		test_16bit(0b1001_0101_1111_0000, Instruction::Com { register: R31 });
		test_16bit(0b1001_0101_1111_0001, Instruction::Neg { register: R31 });
		test_16bit(0b1001_0101_1111_0110, Instruction::Lsr { register: R31 });
		test_16bit(0b1001_0101_1111_0101, Instruction::Asr { register: R31 });
		test_16bit(
			0b1001_1111_0000_0000,
			Instruction::Mul {
				reg_dest: R16,
				reg_read: R16,
			},
		);
		test_16bit(
			0b1110_1111_0000_1111,
			Instruction::Ldi {
				register: UpperRegister::R16,
				value: 255.into(),
			},
		);
		test_16bit(
			0b0010_1111_0000_1111,
			Instruction::Mov {
				reg_dest: R16,
				reg_read: R31,
			},
		);
		test_16bit(
			0b0000_0001_1111_0000,
			Instruction::Movw {
				reg_dest: LowerEvenRegister::R30,
				reg_read: LowerEvenRegister::R0,
			},
		);
		test_16bit(
			0b0011_1000_1111_0000,
			Instruction::Cpi {
				register: UpperRegister::R31,
				value: 128.into(),
			},
		);
		test_16bit(
			0b0001_0110_0000_1111,
			Instruction::Cp {
				reg_dest: R0,
				reg_read: R31,
			},
		);
		test_16bit(
			0b0000_0111_1111_1111,
			Instruction::Cpc {
				reg_dest: R31,
				reg_read: R31,
			},
		);
		test_16bit(
			0b0001_0011_1111_1111,
			Instruction::Cpse {
				reg_dest: R31,
				reg_read: R31,
			},
		);
		test_16bit(0b1111_0010_0000_1001, Instruction::Breq { offset: -63 });
		test_16bit(0b1111_0110_0000_1001, Instruction::Brne { offset: -63 });
		test_16bit(0b1111_0110_0000_1100, Instruction::Brne { offset: -63 });
		test_32bit(
			0b1001_0100_0000_1110_0011_1111_1111_1111,
			Instruction::Call { address: 0x3FFF },
		);
	}
}
