use crate::{
	FlagType, Imm16, Imm3, Imm8, Instruction, LowerEvenRegister, Register, RegisterAddress,
	UpperRegister, WordAddress, WordOffset16, WordOffset8, WordRegister,
};

pub trait Opcode: Sized {
	fn is_32bit(&self) -> bool;
	fn get_word_size(&self) -> u8;
	fn get_byte_size(&self) -> u8 {
		self.get_word_size() * 2
	}
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
				0b11 => {
					let (reg_dest, reg_read) = load_rd5_rr5(opcode_16bit)?;
					Some(Instruction::Add { reg_dest, reg_read })
				}
				_ => None,
			},
			//				first 2 bits after front_4_bits
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
			//				first 2 bits after front_4_bits
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
			// 				bit after the first_4_bits
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
			//             3 bits after front_4_bits
			0b1001 => match (opcode_16bit & 0xe00) >> 9 {
				0b000 => match opcode_16bit & 0xf {
					0b0000 => {
						let (register, address) = load_rr_k16(opcode_16bit, opcode_32bit)?;
						Some(Instruction::Lds { register, address })
					}
					0b1111 => Some(Instruction::Pop {
						register: load_rr(opcode_16bit)?,
					}),
					_ => None,
				},
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
					//				right bit of the 4 bits one to the right of the first 4 bits
					0b100 => match (opcode_16bit & 0x100) >> 8 {
						//		left bit of the 4 bits one to the right of the end of the opcode
						0b0 => match (opcode_16bit & 0x80) >> 7 {
							0b0 => Some(Instruction::Bset {
								flag_type: load_s3(opcode_16bit)?,
							}),
							0b1 => Some(Instruction::Bclr {
								flag_type: load_s3(opcode_16bit)?,
							}),
							_ => None,
						},
						// 				4 bits with 4 bit offset to right end of opcode
						0b1 => match (opcode_16bit & 0x00f0) >> 4 {
							0b0000 => Some(Instruction::Ret),
							0b1001 => Some(Instruction::Break),
							_ => None,
						},
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
					0b010 => Some(Instruction::Asr {
						register: load_rr(opcode_16bit)?,
					}),
					0b111 => Some(Instruction::Call {
						word_address: load_k24(opcode_32bit),
					}),
					0b101 => Some(Instruction::Dec {
						register: load_rr(opcode_16bit)?,
					}),
					_ => None,
				},
				0b100 => {
					let (register_address, bit) = load_a5_b3(opcode_16bit)?;
					Some(Instruction::Cbi {
						register_address,
						bit,
					})
				}
				0b101 => {
					let (register_address, bit) = load_a5_b3(opcode_16bit)?;
					Some(Instruction::Sbi {
						register_address,
						bit,
					})
				}
				// single missing bit of the prior 3 bits
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
			//				first 2 bits after front_4_bits
			0b1111 => match (opcode_16bit & 0x0c00) >> 10 {
				//				3 bits at then end of opcode_16bit
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
				//				3 bits at then end of opcode_16bit
				0b01 => match opcode_16bit & 0x0007 {
					0b000 => Some(Instruction::Brcc {
						word_offset: load_k7(opcode_16bit),
					}),
					0b001 => Some(Instruction::Brne {
						word_offset: load_k7(opcode_16bit),
					}),
					_ => None,
				},
				// 				second bit of the 4 bits after front_4_bits
				0b10 => match (opcode_16bit & 0x300) >> 9 {
					0b0 => {
						let (register, bit) = load_rd5_b3(opcode_16bit)?;
						Some(Instruction::Bld { register, bit })
					}
					0b1 => {
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
}
