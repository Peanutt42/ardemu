use crate::{
	get_bit_from_u16, set_bit_in_u16, FlagType, Imm16, Imm3, Imm8, Instruction, LowerEvenRegister,
	PointerRegister,
	Register::{self, R16, R24},
	RegisterAddress, UpperRegister, WordAddress, WordOffset16, WordOffset8, WordRegister,
};

pub trait Opcode: Sized {
	fn is_32bit(&self) -> bool;
	fn get_word_size(&self) -> u8;
	fn get_byte_size(&self) -> u8 {
		self.get_word_size() * 2
	}
	fn load(opcode_32bit: u32) -> Option<Self>;
	fn get_opcode(self) -> u32;
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

	fn get_word_size(&self) -> u8 {
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
			let d4 = ((opcode_16bit >> 4) & 0xf) as u8;
			// upper registers start at R16
			let rd4 = UpperRegister::try_from(d4 + UpperRegister::R16 as u8).ok()?;
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
		fn load_k12(opcode_16bit: u16) -> WordOffset16 {
			let k12 = opcode_16bit & 0x0fff;
			let negative = (k12 & 0x0800) != 0;
			if negative {
				(k12 | 0xF000) as i16
			} else {
				k12 as i16
			}
			.into()
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

		/// ____ ___d dddd _bbb
		fn load_rd5_b3(opcode_16bit: u16) -> Option<(Register, Imm3)> {
			let rd5 = ((opcode_16bit & 0x1f0) >> 4) as u8;
			let b3 = (opcode_16bit & 0x0007) as u8;
			Some((Register::try_from(rd5).ok()?, Imm3::try_from(b3).ok()?))
		}

		/// ____ ____ kkdd kkkk
		fn load_rd2_k6(opcode_16bit: u16) -> Option<(WordRegister, Imm8)> {
			let rd2 = ((opcode_16bit & 0x0030) >> 4) as u8;
			let k6 = (((opcode_16bit & 0x00c0) >> 2) | (opcode_16bit & 0x000f)) as u8;
			// rd2 = 0 means WordRegister::R24
			// rd2 = 1 means WordRegister::R26
			// ..
			Some((
				WordRegister::try_from(rd2 * 2 + WordRegister::R24 as u8).ok()?,
				k6.into(),
			))
		}

		/// ____ ___d dddd ____ kkkk kkkk kkkk kkkk   (32-bit!)
		fn load_rr_k16(opcode_16bit: u16, opcode_32bit: u32) -> Option<(Register, Imm16)> {
			let rr = ((opcode_16bit & 0x01f0) >> 4) as u8;
			let k16 = (opcode_32bit & 0x0000ffff) as u16;
			Some((Register::try_from(rr).ok()?, k16.into()))
		}

		/// ____ ___k kkkk ___k kkkk kkkk kkkk kkkk    (32-bit!)
		fn load_k24(opcode_32bit: u32) -> WordAddress {
			(((opcode_32bit & 0x01f00000) >> 3) | (opcode_32bit & 0x0001ffff)).into()
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
		fn load_k7(opcode_16bit: u16) -> WordOffset8 {
			let k7 = (opcode_16bit & 0x03f8) >> 3;
			let negative = (k7 & 0x40) != 0;
			if negative {
				(k7 | 0x80) as i8
			} else {
				k7 as i8
			}
			.into()
		}

		/// ____ ____ _sss ____
		/// loads a flag type from the opcode
		fn load_s3(opcode_16bit: u16) -> Option<FlagType> {
			(((opcode_16bit & 0b111_0000) >> 4) as u8).try_into().ok()
		}

		// XXXX ____ ____ ____
		match front_4_bits {
			//				first 2 bits after first 4 bits: ____ XX__ ____ ____
			0b0000 => match (opcode_16bit & 0x0c00) >> 10 {
				//				last 2 bits after first 2 bits after first 4 bits: ____ __XX ____ ____
				0b00 => match (opcode_16bit & 0x0300) >> 8 {
					0b00 if (opcode_16bit & 0xff) == 0x00 => Some(Instruction::Nop),
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
				0b11 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Add { reg_dest, reg_read })
				}
				_ => None,
			},
			//				first 2 bits after front_4_bits: ____ XX__ ____ ____
			0b0001 => match (opcode_16bit & 0x0c00) >> 10 {
				0b00 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Cpse { reg_dest, reg_read })
				}
				0b01 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Cp { reg_dest, reg_read })
				}
				0b10 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Sub { reg_dest, reg_read })
				}
				0b11 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Adc { reg_dest, reg_read })
				}
				_ => None,
			},
			//				first 2 bits after front_4_bits: ____ XX__ ____ ____
			0b0010 => match (opcode_16bit & 0x0c00) >> 10 {
				0b00 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::And { reg_dest, reg_read })
				}
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
			// 				bit after the first_4_bits: ____ X___ ____ ____
			0b1011 => match (opcode_16bit & 0x800) >> 11 {
				0b0 => {
					let (register, address) = load_rr5_a6(opcode_16bit)?;
					Some(Instruction::In { address, register })
				}
				0b1 => {
					let (register, address) = load_rr5_a6(opcode_16bit)?;
					Some(Instruction::Out { address, register })
				}
				_ => None,
			},
			0b0100 => {
				let (register, value) = load_rd4_k8(opcode_16bit)?;
				Some(Instruction::Sbci { register, value })
			}
			//             3 bits after front_4_bits: ____ XXX_ ____ ____
			0b1001 => match (opcode_16bit & 0xe00) >> 9 {
				// 			4 bits at the right end of the opcode: ____ ____ ____ XXXX
				0b000 => match opcode_16bit & 0xf {
					0b0000 => {
						let (register, address) = load_rr_k16(opcode_16bit, opcode_32bit)?;
						Some(Instruction::Lds { register, address })
					}
					0b0001 => Some(Instruction::Ld {
						register: load_rr(opcode_16bit)?,
						pointer_register: PointerRegister::Z_POST_INC,
					}),
					0b0010 => Some(Instruction::Ld {
						register: load_rr(opcode_16bit)?,
						pointer_register: PointerRegister::Z_PRE_DEC,
					}),
					0b1001 => Some(Instruction::Ld {
						register: load_rr(opcode_16bit)?,
						pointer_register: PointerRegister::Y_POST_INC,
					}),
					0b1010 => Some(Instruction::Ld {
						register: load_rr(opcode_16bit)?,
						pointer_register: PointerRegister::Y_PRE_DEC,
					}),
					0b1100 => Some(Instruction::Ld {
						register: load_rr(opcode_16bit)?,
						pointer_register: PointerRegister::X,
					}),
					0b1101 => Some(Instruction::Ld {
						register: load_rr(opcode_16bit)?,
						pointer_register: PointerRegister::X_POST_INC,
					}),
					0b1110 => Some(Instruction::Ld {
						register: load_rr(opcode_16bit)?,
						pointer_register: PointerRegister::X_PRE_DEC,
					}),
					0b1111 => Some(Instruction::Pop {
						register: load_rr(opcode_16bit)?,
					}),
					_ => None,
				},
				// 4 bits of the end of opcode: ____ ____ ____ XXXX
				0b001 => match opcode_16bit & 0x000f {
					0b0000 => {
						let (register, address) = load_rr_k16(opcode_16bit, opcode_32bit)?;
						Some(Instruction::Sts { address, register })
					}
					0b0001 => Some(Instruction::St {
						pointer_register: PointerRegister::Z_POST_INC,
						register: load_rr(opcode_16bit)?,
					}),
					0b0010 => Some(Instruction::St {
						pointer_register: PointerRegister::Z_PRE_DEC,
						register: load_rr(opcode_16bit)?,
					}),
					0b1001 => Some(Instruction::St {
						pointer_register: PointerRegister::Y_POST_INC,
						register: load_rr(opcode_16bit)?,
					}),
					0b1010 => Some(Instruction::St {
						pointer_register: PointerRegister::Y_PRE_DEC,
						register: load_rr(opcode_16bit)?,
					}),
					0b1100 => Some(Instruction::St {
						pointer_register: PointerRegister::X,
						register: load_rr(opcode_16bit)?,
					}),
					0b1101 => Some(Instruction::St {
						pointer_register: PointerRegister::X_POST_INC,
						register: load_rr(opcode_16bit)?,
					}),
					0b1110 => Some(Instruction::St {
						pointer_register: PointerRegister::X_PRE_DEC,
						register: load_rr(opcode_16bit)?,
					}),
					0b1111 => Some(Instruction::Push {
						register: load_rr(opcode_16bit)?,
					}),
					_ => None,
				},
				//			first 3 bits of the 4 bits at the right end of the opcode: ____ ____ ____ XXX_
				0b010 => match (opcode_16bit & 0x000e) >> 1 {
					// 				last bit (lsb): ____ ____ ____ ___X
					0b000 => match opcode_16bit & 0x0001 {
						0b0 => Some(Instruction::Com {
							register: load_rr(opcode_16bit)?,
						}),
						0b1 => Some(Instruction::Neg {
							register: load_rr(opcode_16bit)?,
						}),
						_ => None,
					},
					//				right bit of the 4 bits one to the right of the first 4 bits: ____ ___X ____ ____
					0b100 => match (opcode_16bit & 0x100) >> 8 {
						//		left bit of the 4 bits one to the right of the end of the opcode: ____ ____ X___ ____
						0b0 => match (opcode_16bit & 0x80) >> 7 {
							0b0 if (opcode_16bit & 0x1) == 0 => Some(Instruction::Bset {
								flag_type: load_s3(opcode_16bit)?,
							}),
							0b1 if (opcode_16bit & 0x1) == 0 => Some(Instruction::Bclr {
								flag_type: load_s3(opcode_16bit)?,
							}),
							_ => None,
						},
						// 				4 bits with 4 bit offset to right end of opcode: ____ ____ XXXX ____
						0b1 => match (opcode_16bit & 0x00f0) >> 4 {
							0b0000 if (opcode_16bit & 0x1) == 0 => Some(Instruction::Ret),
							0b0001 if (opcode_16bit & 0x1) == 0 => Some(Instruction::Reti),
							0b1001 if (opcode_16bit & 0x1) == 0 => Some(Instruction::Break),
							_ => None,
						},
						_ => None,
					},
					//				lsb: : ____ ____ ____ ___X
					0b011 => match opcode_16bit & 0x0001 {
						0b0 => Some(Instruction::Lsr {
							register: load_rr(opcode_16bit)?,
						}),
						0b1 => Some(Instruction::Ror {
							register: load_rr(opcode_16bit)?,
						}),
						_ => None,
					},
					//				lsb: : ____ ____ ____ ___X
					0b001 => match opcode_16bit & 0x0001 {
						0b0 => Some(Instruction::Swap {
							register: load_rr(opcode_16bit)?,
						}),
						0b1 => Some(Instruction::Inc {
							register: load_rr(opcode_16bit)?,
						}),
						_ => None,
					},
					0b110 => Some(Instruction::Jmp {
						word_address: load_k24(opcode_32bit),
					}),
					0b010 if (opcode_16bit & 0x1) == 1 => Some(Instruction::Asr {
						register: load_rr(opcode_16bit)?,
					}),
					0b111 => Some(Instruction::Call {
						word_address: load_k24(opcode_32bit),
					}),
					0b101 if (opcode_16bit & 0x1) == 0 => Some(Instruction::Dec {
						register: load_rr(opcode_16bit)?,
					}),
					_ => None,
				},
				0b100 if (opcode_16bit & 0x100) == 0x000 => {
					let (register_address, bit) = load_a5_b3(opcode_16bit)?;
					Some(Instruction::Cbi {
						register_address,
						bit,
					})
				}
				0b101 if (opcode_16bit & 0x100) == 0x000 => {
					let (register_address, bit) = load_a5_b3(opcode_16bit)?;
					Some(Instruction::Sbi {
						register_address,
						bit,
					})
				}
				// single missing bit of the prior 3 bits: ____ ___X ____ ____
				0b011 => match (opcode_16bit & 0x100) >> 8 {
					0b0 => {
						let (register, value) = load_rd2_k6(opcode_16bit)?;
						Some(Instruction::Adiw { register, value })
					}
					0b1 => {
						let (register, value) = load_rd2_k6(opcode_16bit)?;
						Some(Instruction::Sbiw { register, value })
					}
					_ => None,
				},
				0b110 | 0b111 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Mul { reg_dest, reg_read })
				}
				_ => None,
			},
			//				last 4 bits at the right end of the opcode: ____ ____ ____ XXXX
			0b1000 => match opcode_16bit & 0xf {
				//				first 3 bits after first_4_bits: ____ XXX_ ____ ____
				0b1000 => match (opcode_16bit & 0xe00) >> 9 {
					0b000 => Some(Instruction::Ld {
						register: load_rr(opcode_16bit)?,
						pointer_register: PointerRegister::Y,
					}),
					0b001 => Some(Instruction::St {
						register: load_rr(opcode_16bit)?,
						pointer_register: PointerRegister::Y,
					}),
					_ => None,
				},
				//				first 3 bits after first_4_bits: ____ XXX_ ____ ____
				0b0000 => match (opcode_16bit & 0xe00) >> 9 {
					0b000 => Some(Instruction::Ld {
						register: load_rr(opcode_16bit)?,
						pointer_register: PointerRegister::Z,
					}),
					0b001 => Some(Instruction::St {
						register: load_rr(opcode_16bit)?,
						pointer_register: PointerRegister::Z,
					}),
					_ => None,
				},
				_ => None,
			},
			0b1100 => Some(Instruction::RJmp {
				word_offset: load_k12(opcode_16bit),
			}),
			0b0101 => {
				let (register, value) = load_rd4_k8(opcode_16bit)?;
				Some(Instruction::Subi { register, value })
			}
			0b1101 => Some(Instruction::RCall {
				word_offset: load_k12(opcode_16bit),
			}),
			0b1110 => {
				let (register, value) = load_rd4_k8(opcode_16bit)?;
				Some(Instruction::Ldi { register, value })
			}
			0b0111 => {
				let (register, value) = load_rd4_k8(opcode_16bit)?;
				Some(Instruction::Andi { register, value })
			}
			//				first 2 bits after front_4_bits: ____ XX__ ____ ____
			0b1111 => match (opcode_16bit & 0x0c00) >> 10 {
				//				3 bits at then end of opcode_16bit: ____ ____ ____ _XXX
				0b00 => match opcode_16bit & 0x0007 {
					0b000 => Some(Instruction::Brcs {
						word_offset: load_k7(opcode_16bit),
					}),
					0b001 => Some(Instruction::Breq {
						word_offset: load_k7(opcode_16bit),
					}),
					0b100 => Some(Instruction::Brlt {
						word_offset: load_k7(opcode_16bit),
					}),
					_ => None,
				},
				//				3 bits at then end of opcode_16bit: ____ ____ ____ _XXX
				0b01 => match opcode_16bit & 0x0007 {
					0b000 => Some(Instruction::Brcc {
						word_offset: load_k7(opcode_16bit),
					}),
					0b001 => Some(Instruction::Brne {
						word_offset: load_k7(opcode_16bit),
					}),
					_ => None,
				},
				// 				second bit of the 4 bits after front_4_bits: ____ __X_ ____ ____
				0b10 => match (opcode_16bit & 0x200) >> 9 {
					0b0 if (opcode_16bit & 0x8) == 0x0 => {
						let (register, bit) = load_rd5_b3(opcode_16bit)?;
						Some(Instruction::Bld { register, bit })
					}
					0b1 if (opcode_16bit & 0x8) == 0x0 => {
						let (register, bit) = load_rd5_b3(opcode_16bit)?;
						Some(Instruction::Bst { register, bit })
					}
					_ => None,
				},
				_ => None,
			},
			_ => None,
		}
	}

	fn get_opcode(self) -> u32 {
		/// binary value in the opcode!
		fn upper_register_value(upper_register: UpperRegister) -> u16 {
			(upper_register as u16) - R16 as u16
		}

		/// binary value in the opcode!
		fn lower_even_register_value(lower_even_register: LowerEvenRegister) -> u16 {
			(lower_even_register as u16) / 2
		}

		/// binary value in the opcode!
		fn word_register_value(word_register: WordRegister) -> u16 {
			(word_register as u16 - R24 as u16) / 2
		}

		/// ____ ___k kkkk ___k kkkk kkkk kkkk kkkk
		fn word_address_opcode(word_address: WordAddress) -> u32 {
			let first_address_part = (word_address.0 >> 16) as u16;
			let mut first_opcode = (first_address_part >> 1) << 4;
			first_opcode = set_bit_in_u16(first_opcode, 0, get_bit_from_u16(first_address_part, 0));
			let second_opcode = word_address.0 & 0xFFFF;

			((first_opcode as u32) << 16) | second_opcode
		}

		/// ____ __rd dddd rrrr
		fn r5_d5_opcode(reg_dest: Register, reg_read: Register) -> u16 {
			let d = reg_dest as u16;
			let r = reg_read as u16;
			let mut opcode = (r & 0xf) | (d << 4);
			opcode = set_bit_in_u16(opcode, 9, get_bit_from_u16(r, 4));
			opcode
		}

		/// ____ ___d dddd ____
		fn d5_opcode(register: Register) -> u16 {
			let d = register as u16;
			d << 4
		}

		/// ____ kkkk dddd kkkk
		fn d4_k8_opcode(register: UpperRegister, value: Imm8) -> u16 {
			let d = upper_register_value(register);
			let k8 = value.0 as u16;
			(d << 4) | (k8 & 0xf) | ((k8 >> 4) << 8)
		}

		/// ____ ____ dddd rrrr
		fn r4_d4_opcode(reg_dest: LowerEvenRegister, reg_read: LowerEvenRegister) -> u16 {
			let d = lower_even_register_value(reg_dest);
			let r = lower_even_register_value(reg_read);
			r | (d << 4)
		}

		/// ____ kkkk kkkk kkkk
		fn k12_opcode(word_offset: WordOffset16) -> u16 {
			if word_offset.0 < 0 {
				(((!word_offset.0.unsigned_abs()) & 0x7ff) + 1) | 0x0800
			} else {
				word_offset.0 as u16
			}
		}

		/// ____ __kk kkkk k___
		fn k7_opcode(word_offset: WordOffset8) -> u16 {
			let k6 = word_offset.0.unsigned_abs() as u16 & 0x3f;
			let opcode = if word_offset.0.is_negative() {
				(((!k6) & 0x3f) + 1) | 0x40
			} else {
				k6
			};
			opcode << 3
		}

		/// ____ ____ kkdd kkkk
		fn d2_k6_opcode(register: WordRegister, value: Imm8) -> u16 {
			let d = word_register_value(register);
			let k6 = value.0 as u16;
			(d << 4) | (k6 & 0xf) | ((k6 & 0x30) << 2)
		}

		/// ____ ____ _sss ____
		fn s3_opcode(bit: Imm3) -> u16 {
			(bit.0 as u16) << 4
		}

		/// ____ ____ aaaa abbb
		fn a5_b3_opcode(register_address: RegisterAddress, bit: Imm3) -> u16 {
			let b = bit.0 as u16;
			let a = register_address.0 as u16;
			b | (a << 3)
		}

		/// ____ ___d dddd _bbb
		fn d5_b3_opcode(register: Register, bit: Imm3) -> u16 {
			let b = bit.0 as u16;
			let d = register as u16;
			b | (d << 4)
		}

		/// ____ ___d dddd ____ kkkk kkkk kkkk kkkk
		fn d5_k16_opcode(register: Register, address: Imm16) -> u32 {
			let k16 = address.0 as u32;
			let d = register as u32;
			k16 | (d << 20)
		}

		/// ____ _aad dddd aaaa
		fn d5_a6_opcode(register: Register, address: Imm8) -> u16 {
			let d = register as u16;
			let a = address.0 as u16;
			(d << 4) | (a & 0xf) | ((a & 0x30) << 5)
		}

		fn single_16bit_opcode(opcode_16bit: u16) -> u32 {
			(opcode_16bit as u32) << 16
		}

		match self {
			Self::Nop => 0x0000,
			Self::Break => 0b1001_0101_1001_1000_0000_0000_0000_0000,
			Self::Jmp { word_address } => {
				0b1001_0100_0000_1100_0000_0000_0000_0000 | word_address_opcode(word_address)
			}
			Self::Or { reg_dest, reg_read } => {
				single_16bit_opcode(0b0010_1000_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Ori { register, value } => {
				single_16bit_opcode(0b0110_0000_0000_0000 | d4_k8_opcode(register, value))
			}
			Self::Eor { reg_dest, reg_read } => {
				single_16bit_opcode(0b0010_0100_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Com { register } => {
				single_16bit_opcode(0b1001_0100_0000_0000 | d5_opcode(register))
			}
			Self::Neg { register } => {
				single_16bit_opcode(0b1001_0100_0000_0001 | d5_opcode(register))
			}
			Self::Swap { register } => {
				single_16bit_opcode(0b1001_0100_0000_0010 | d5_opcode(register))
			}
			Self::Lsr { register } => {
				single_16bit_opcode(0b1001_0100_0000_0110 | d5_opcode(register))
			}
			Self::Ror { register } => {
				single_16bit_opcode(0b1001_0100_0000_0111 | d5_opcode(register))
			}
			Self::Asr { register } => {
				single_16bit_opcode(0b1001_0100_0000_0101 | d5_opcode(register))
			}
			Self::Mul { reg_dest, reg_read } => {
				single_16bit_opcode(0b1001_1100_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Ldi { register, value } => {
				single_16bit_opcode(0b1110_0000_0000_0000 | d4_k8_opcode(register, value))
			}
			Self::Mov { reg_dest, reg_read } => {
				single_16bit_opcode(0b0010_1100_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Movw { reg_dest, reg_read } => {
				single_16bit_opcode(0b0000_0001_0000_0000 | r4_d4_opcode(reg_dest, reg_read))
			}
			Self::RJmp { word_offset } => {
				single_16bit_opcode(0b1100_0000_0000_0000 | k12_opcode(word_offset))
			}
			Self::Push { register } => {
				single_16bit_opcode(0b1001_0010_0000_1111 | d5_opcode(register))
			}
			Self::Pop { register } => {
				single_16bit_opcode(0b1001_0000_0000_1111 | d5_opcode(register))
			}
			Self::Cpi { register, value } => {
				single_16bit_opcode(0b0011_0000_0000_0000 | d4_k8_opcode(register, value))
			}
			Self::Cp { reg_dest, reg_read } => {
				single_16bit_opcode(0b0001_0100_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Cpc { reg_dest, reg_read } => {
				single_16bit_opcode(0b0000_0100_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Cpse { reg_dest, reg_read } => {
				single_16bit_opcode(0b0001_0000_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Breq { word_offset } => {
				single_16bit_opcode(0b1111_0000_0000_0001 | k7_opcode(word_offset))
			}
			Self::Brne { word_offset } => {
				single_16bit_opcode(0b1111_0100_0000_0001 | k7_opcode(word_offset))
			}
			Self::Brlt { word_offset } => {
				single_16bit_opcode(0b1111_0000_0000_0100 | k7_opcode(word_offset))
			}
			Self::Brcc { word_offset } => {
				single_16bit_opcode(0b1111_0100_0000_0000 | k7_opcode(word_offset))
			}
			Self::Brcs { word_offset } => {
				single_16bit_opcode(0b1111_0000_0000_0000 | k7_opcode(word_offset))
			}
			Self::Call { word_address } => {
				0b1001_0100_0000_1110_0000_0000_0000_0000 | word_address_opcode(word_address)
			}
			Self::Ret => 0b1001_0101_0000_1000_0000_0000_0000_0000,
			Self::Reti => 0b1001_0101_0001_1000_0000_0000_0000_0000,
			Self::RCall { word_offset } => {
				single_16bit_opcode(0b1101_0000_0000_0000 | k12_opcode(word_offset))
			}
			Self::Sub { reg_dest, reg_read } => {
				single_16bit_opcode(0b0001_1000_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Sbc { reg_dest, reg_read } => {
				single_16bit_opcode(0b0000_1000_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Subi { register, value } => {
				single_16bit_opcode(0b0101_0000_0000_0000 | d4_k8_opcode(register, value))
			}
			Self::Sbci { register, value } => {
				single_16bit_opcode(0b0100_0000_0000_0000 | d4_k8_opcode(register, value))
			}
			Self::Sbiw { register, value } => {
				single_16bit_opcode(0b1001_0111_0000_0000 | d2_k6_opcode(register, value))
			}
			Self::Dec { register } => {
				single_16bit_opcode(0b1001_0100_0000_1010 | d5_opcode(register))
			}
			Self::Add { reg_dest, reg_read } => {
				single_16bit_opcode(0b0000_1100_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Adc { reg_dest, reg_read } => {
				single_16bit_opcode(0b0001_1100_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Adiw { register, value } => {
				single_16bit_opcode(0b1001_0110_0000_0000 | d2_k6_opcode(register, value))
			}
			Self::Inc { register } => {
				single_16bit_opcode(0b1001_0100_0000_0011 | d5_opcode(register))
			}
			Self::And { reg_dest, reg_read } => {
				single_16bit_opcode(0b0010_0000_0000_0000 | r5_d5_opcode(reg_dest, reg_read))
			}
			Self::Andi { register, value } => {
				single_16bit_opcode(0b0111_0000_0000_0000 | d4_k8_opcode(register, value))
			}
			Self::Bset { flag_type } => {
				single_16bit_opcode(0b1001_0100_0000_1000 | s3_opcode(flag_type.as_imm3()))
			}
			Self::Bclr { flag_type } => {
				single_16bit_opcode(0b1001_0100_1000_1000 | s3_opcode(flag_type.as_imm3()))
			}
			Self::Sbi {
				register_address,
				bit,
			} => single_16bit_opcode(0b1001_1010_0000_0000 | a5_b3_opcode(register_address, bit)),
			Self::Cbi {
				register_address,
				bit,
			} => single_16bit_opcode(0b1001_1000_0000_0000 | a5_b3_opcode(register_address, bit)),
			Self::Bst { register, bit } => {
				single_16bit_opcode(0b1111_1010_0000_0000 | d5_b3_opcode(register, bit))
			}
			Self::Bld { register, bit } => {
				single_16bit_opcode(0b1111_1000_0000_0000 | d5_b3_opcode(register, bit))
			}
			Self::Sts { address, register } => {
				0b1001_0010_0000_0000_0000_0000_0000_0000 | d5_k16_opcode(register, address)
			}
			Self::Lds { address, register } => {
				0b1001_0000_0000_0000_0000_0000_0000_0000 | d5_k16_opcode(register, address)
			}
			Self::St {
				pointer_register,
				register,
			} => single_16bit_opcode(match pointer_register {
				PointerRegister::X => 0b1001_0010_0000_1100 | d5_opcode(register),
				PointerRegister::X_POST_INC => 0b1001_0010_0000_1101 | d5_opcode(register),
				PointerRegister::X_PRE_DEC => 0b1001_0010_0000_1110 | d5_opcode(register),
				PointerRegister::Y => 0b1000_0010_0000_1000 | d5_opcode(register),
				PointerRegister::Y_POST_INC => 0b1001_0010_0000_1001 | d5_opcode(register),
				PointerRegister::Y_PRE_DEC => 0b1001_0010_0000_1010 | d5_opcode(register),
				PointerRegister::Z => 0b1000_0010_0000_0000 | d5_opcode(register),
				PointerRegister::Z_POST_INC => 0b1001_0010_0000_0001 | d5_opcode(register),
				PointerRegister::Z_PRE_DEC => 0b1001_0010_0000_0010 | d5_opcode(register),
			}),
			Self::Ld {
				register,
				pointer_register,
			} => single_16bit_opcode(match pointer_register {
				PointerRegister::X => 0b1001_0000_0000_1100 | d5_opcode(register),
				PointerRegister::X_POST_INC => 0b1001_0000_0000_1101 | d5_opcode(register),
				PointerRegister::X_PRE_DEC => 0b1001_0000_0000_1110 | d5_opcode(register),
				PointerRegister::Y => 0b1000_0000_0000_1000 | d5_opcode(register),
				PointerRegister::Y_POST_INC => 0b1001_0000_0000_1001 | d5_opcode(register),
				PointerRegister::Y_PRE_DEC => 0b1001_0000_0000_1010 | d5_opcode(register),
				PointerRegister::Z => 0b1000_0000_0000_0000 | d5_opcode(register),
				PointerRegister::Z_POST_INC => 0b1001_0000_0000_0001 | d5_opcode(register),
				PointerRegister::Z_PRE_DEC => 0b1001_0000_0000_0010 | d5_opcode(register),
			}),
			Self::In { register, address } => {
				single_16bit_opcode(0b1011_0000_0000_0000 | d5_a6_opcode(register, address))
			}
			Self::Out { register, address } => {
				single_16bit_opcode(0b1011_1000_0000_0000 | d5_a6_opcode(register, address))
			}
		}
	}
}
