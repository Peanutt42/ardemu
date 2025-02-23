use std::{collections::HashSet, ops::RangeInclusive};

use crate::{register::RegisterPair16, CpuError, Flags, Instruction, Register};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CpuStatus {
	Normal,
	/// the current instruction was not run, as it was marked as a breakpoint
	BreakpointHit,
	/// the current instruction was not run, as it is a break instruction
	BreakHit,
	ProgramFinished,
}

#[derive(Debug, Clone)]
pub struct Cpu {
	program: Box<[Instruction]>,
	registers: [u8; Register::COUNT],
	program_counter: u16,
	stack_pointer: u16,
	flags: Flags,
	sram: [u8; Self::SRAM_SIZE], // SRAM (64KB)
	// contains the program address of the breakpoints
	breakpoints: HashSet<u16>,
}

impl Cpu {
	/// 64 KB
	const SRAM_SIZE: usize = 64 * 1024;
	const STACK_START_ADDRESS: u16 = 0xFEFF;
	const STACK_END_ADDRESS: u16 = 0xFC00;

	pub fn new(program: impl Into<Box<[Instruction]>>) -> Self {
		Self {
			program: program.into(),
			registers: [0; Register::COUNT],
			program_counter: 0,
			stack_pointer: Self::STACK_START_ADDRESS,
			flags: Flags::default(),
			sram: [0u8; Self::SRAM_SIZE],
			breakpoints: HashSet::new(),
		}
	}

	/// resets everything except the last loaded program
	pub fn reset(&mut self) {
		self.registers = [0; Register::COUNT];
		self.program_counter = 0;
		self.stack_pointer = Self::STACK_START_ADDRESS;
		self.sram = [0u8; Self::SRAM_SIZE];
		self.breakpoints.clear();
	}

	pub fn add_breakpoint(&mut self, address: u16) {
		self.breakpoints.insert(address);
	}

	pub fn remove_breakpoint(&mut self, address: u16) {
		self.breakpoints.remove(&address);
	}

	pub fn get_breakpoints(&self) -> &HashSet<u16> {
		&self.breakpoints
	}

	pub fn flags(&self) -> Flags {
		self.flags
	}

	pub fn get_program_counter(&self) -> u16 {
		self.program_counter
	}

	pub fn get_current_instruction(&self) -> Option<Instruction> {
		self.program.get(self.program_counter as usize).copied()
	}

	pub fn read_register(&self, reg: impl Into<Register>) -> u8 {
		reg.into().read_from(&self.registers)
	}

	pub fn write_register(&mut self, reg: impl Into<Register>, value: u8) {
		reg.into().write_in(&mut self.registers, value);
	}

	pub fn read_register_pair16(&self, reg_pair: RegisterPair16) -> u16 {
		reg_pair.read_from(&self.registers)
	}

	pub fn write_register_pair16(&mut self, reg_pair: RegisterPair16, value: u16) {
		reg_pair.write_in(&mut self.registers, value);
	}

	pub fn read_ram(&self, address: u16) -> Result<u8, CpuError> {
		let ram = self
			.sram
			.get(address as usize)
			.ok_or(CpuError::InvalidRamAddress { addr: address })?;
		Ok(*ram)
	}

	pub fn read_ram_range(&self, address_range: RangeInclusive<u16>) -> Result<&[u8], CpuError> {
		self.sram
			.get(*address_range.start() as usize..=*address_range.end() as usize)
			.ok_or(CpuError::InvalidRamAddress {
				addr: *address_range.start(),
			})
	}

	pub fn write_ram(&mut self, address: u16, value: u8) -> Result<(), CpuError> {
		let mut_ram = self
			.sram
			.get_mut(address as usize)
			.ok_or(CpuError::InvalidRamAddress { addr: address })?;
		*mut_ram = value;
		Ok(())
	}

	fn push(&mut self, value: u8) -> Result<(), CpuError> {
		if self.stack_pointer <= Self::STACK_END_ADDRESS {
			return Err(CpuError::StackOverflow);
		}
		self.sram[self.stack_pointer as usize] = value;
		self.stack_pointer -= 1;
		Ok(())
	}
	fn pop(&mut self) -> Result<u8, CpuError> {
		self.stack_pointer += 1;
		if self.stack_pointer > Self::STACK_START_ADDRESS {
			return Err(CpuError::StackUnderflow);
		}
		let value = self.sram[self.stack_pointer as usize];
		Ok(value)
	}

	pub fn execute(&mut self, instruction: Instruction) -> Result<CpuStatus, CpuError> {
		if self.breakpoints.contains(&self.program_counter) {
			return Ok(CpuStatus::BreakpointHit);
		}

		match instruction {
			Instruction::Nop {} => {
				self.program_counter += 1;
			}
			Instruction::Break {} => {
				return Ok(CpuStatus::BreakHit);
			}
			Instruction::Jmp { address } => {
				self.program_counter = address.0;
			}
			Instruction::Eor { reg_dest, reg_read } => {
				let result = self.read_register(reg_dest) ^ self.read_register(reg_read);
				self.write_register(reg_dest, result);
				self.flags.set_zns_v0(result);
				self.program_counter += 1;
			}
			Instruction::Or { reg_dest, reg_read } => {
				let result = self.read_register(reg_dest) | self.read_register(reg_read);
				self.write_register(reg_dest, result);
				self.flags.set_zns_v0(result);
				self.program_counter += 1;
			}
			Instruction::Ori { register, value } => {
				let result = self.read_register(register) | value.0;
				self.write_register(register, result);
				self.flags.set_zns_v0(result);
				self.program_counter += 1;
			}
			Instruction::Ldi { register, value } => {
				self.write_register(register, value.0);
				self.program_counter += 1;
			}
			Instruction::Mov { reg_dest, reg_read } => {
				self.write_register(reg_dest, self.read_register(reg_read));
				self.program_counter += 1;
			}
			Instruction::Movw { reg_dest, reg_read } => {
				let value = self.read_register_pair16(reg_read);
				self.write_register_pair16(reg_dest, value);
				self.program_counter += 1;
			}
			Instruction::RJmp { offset } => {
				let new_program_counter = (self.program_counter as i32)
					.wrapping_add(offset as i32)
					.wrapping_add(1);
				self.program_counter = new_program_counter as u16;
			}
			Instruction::Push { register } => {
				self.push(self.read_register(register))?;
				self.program_counter += 1;
			}
			Instruction::Pop { register } => {
				let value = self.pop()?;
				self.write_register(register, value);
				self.program_counter += 1;
			}
			Instruction::Cpi { register, value } => {
				let register_value = self.read_register(register);
				let result = register_value.wrapping_sub(value.0);
				self.flags.set_sub_zns(register_value, value.0, result);
				self.program_counter += 1;
			}
			Instruction::Cp { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value.wrapping_sub(read_value);
				self.flags.set_sub_zns(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Cpc { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value
					.wrapping_sub(read_value)
					.wrapping_sub(self.flags.carry_u8());
				self.flags.set_sub_zns(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Cpse { reg_dest, reg_read } => {
				if self.read_register(reg_dest) == self.read_register(reg_read) {
					self.program_counter += 2;
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Breq { offset } => {
				if self.flags.zero() {
					let new_program_counter = (self.program_counter as i32)
						.wrapping_add(offset as i32)
						.wrapping_add(1);
					self.program_counter = new_program_counter as u16;
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Brne { offset } => {
				if !self.flags.zero() {
					let new_program_counter = (self.program_counter as i32)
						.wrapping_add(offset as i32)
						.wrapping_add(1);
					self.program_counter = new_program_counter as u16;
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Brlt { offset } => {
				if self.flags.sign() {
					let new_program_counter = (self.program_counter as i32)
						.wrapping_add(offset as i32)
						.wrapping_add(1);
					self.program_counter = new_program_counter as u16;
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Call { address } => {
				// TODO: handle 16-bit PC, for now not needed
				self.push(self.program_counter as u8 + 1)?;
				self.program_counter = address.0;
			}
			Instruction::Ret {} => {
				self.program_counter = self.pop()? as u16;
			}
			Instruction::Sub { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value.wrapping_sub(read_value);
				self.write_register(reg_dest, result);
				self.flags.set_sub_zns(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Sbc { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value
					.wrapping_sub(read_value)
					.wrapping_sub(self.flags.carry_u8());
				self.write_register(reg_dest, result);
				self.flags.set_sub_zns(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Subi { register, value } => {
				let register_value = self.read_register(register);
				let result = register_value.wrapping_sub(value.0);
				self.write_register(register, result);
				self.flags.set_sub_zns(register_value, value.0, result);
				self.program_counter += 1;
			}
			Instruction::Sbci { register, value } => {
				let register_value = self.read_register(register);
				let result = register_value
					.wrapping_sub(value.0)
					.wrapping_sub(self.flags.carry_u8());
				self.write_register(register, result);
				self.flags.set_sub_rzns(register_value, value.0, result);
				self.program_counter += 1;
			}
			Instruction::Sbiw { register, value } => {
				let register_value = self.read_register_pair16(register.to_pair());
				let result = register_value.wrapping_sub(value.0);
				self.write_register_pair16(register.to_pair(), result);
				self.flags.set_zns16(result);
				self.program_counter += 1;
			}
			Instruction::Dec { register } => {
				let register_value = self.read_register(register);
				let result = register_value.wrapping_sub(1);
				self.write_register(register, result);
				self.flags.set_sub_zns(register_value, 1, result);
				self.program_counter += 1;
			}
			Instruction::Add { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value.wrapping_add(read_value);
				self.write_register(reg_dest, result);
				self.flags.set_add_zns(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Adc { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value
					.wrapping_add(read_value)
					.wrapping_add(self.flags.carry_u8());
				self.write_register(reg_dest, result);
				self.flags.set_add_zns(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Adiw { register, value } => {
				let register_value = self.read_register_pair16(register.to_pair());
				let result = register_value.wrapping_add(value.0);
				self.write_register_pair16(register.to_pair(), result);
				self.flags.set_zns16(result);
				self.program_counter += 1;
			}
			Instruction::Inc { register } => {
				let register_value = self.read_register(register);
				let result = register_value.wrapping_add(1);
				self.write_register(register, result);
				self.flags.set_add_zns(register_value, 1, result);
				self.program_counter += 1;
			}
			Instruction::And { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value & read_value;
				self.write_register(reg_dest, result);
				self.flags.set_zns_v0(result);
				self.program_counter += 1;
			}
			Instruction::Andi { register, value } => {
				let register_value = self.read_register(register);
				let result = register_value & value.0;
				self.write_register(register, result);
				self.flags.set_zns_v0(result);
				self.program_counter += 1;
			}
			Instruction::Bset { flag_type } => {
				self.flags.set(flag_type);
				self.program_counter += 1;
			}
			Instruction::Bclr { flag_type } => {
				self.flags.clear(flag_type);
				self.program_counter += 1;
			}
			Instruction::Bst { register, bit } => {
				let bit = get_bit_from_u8(self.read_register(register), bit.0);
				self.flags.set_copy_bit(bit);
				self.program_counter += 1;
			}
			Instruction::Bld { register, bit } => {
				let bit_value = self.flags.bit_copy();
				self.write_register(
					register,
					set_bit_in_u8(self.read_register(register), bit.0, bit_value),
				);
				self.program_counter += 1;
			}
			Instruction::Sts { address, register } => {
				self.write_ram(address.0, self.read_register(register))?;
				self.program_counter += 1;
			}
			Instruction::Lds { register, address } => {
				self.write_register(register, self.read_ram(address.0)?);
				self.program_counter += 1;
			}
			Instruction::Out { address, register } => {
				self.write_ram(address.0 as u16, self.read_register(register))?;
				self.program_counter += 1;
			}
			Instruction::In { register, address } => {
				self.write_register(register, self.read_ram(address.0 as u16)?);
				self.program_counter += 1;
			}
		}

		Ok(CpuStatus::Normal)
	}

	pub fn step(&mut self) -> Result<CpuStatus, CpuError> {
		match self.get_current_instruction() {
			Some(instruction) => self.execute(instruction),
			None => Ok(CpuStatus::ProgramFinished),
		}
	}

	/// Skips the current instruction, even if it is a break instruction.
	pub fn skip(&mut self) {
		self.program_counter += 1;
	}
}

impl Default for Cpu {
	fn default() -> Self {
		Self::new([])
	}
}

pub(crate) fn get_bit_from_u8(value: u8, bit: u8) -> bool {
	value & (1 << bit) != 0
}

pub(crate) fn set_bit_in_u8(value: u8, bit: u8, bit_value: bool) -> u8 {
	if bit_value {
		value | (1 << bit)
	} else {
		value & !(1 << bit)
	}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
	use crate::{
		Register::{R0, R1, R16, R17},
		UpperRegister,
	};

	use super::*;

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

	#[test]
	fn test_execute_push_pop() {
		let mut cpu = Cpu::default();
		cpu.write_register(R0, 42);
		cpu.write_register(R1, 15);
		cpu.execute(Instruction::Push { register: R0 }).unwrap();
		cpu.execute(Instruction::Push { register: R1 }).unwrap();
		cpu.write_register(R0, 0);
		cpu.write_register(R1, 0);
		cpu.execute(Instruction::Pop { register: R1 }).unwrap();
		cpu.execute(Instruction::Pop { register: R0 }).unwrap();
		assert_eq!(cpu.read_register(R0), 42);
		assert_eq!(cpu.read_register(R1), 15);
	}

	#[test]
	fn test_stackoverflow() {
		let mut cpu = Cpu::default();
		cpu.write_register(R0, 0);
		loop {
			match cpu.execute(Instruction::Push { register: R0 }) {
				Ok(_) => (),
				Err(CpuError::StackOverflow) => break,
				Err(err) => panic!("Unexpected error before stack overflow: {err}"),
			}
		}
	}

	#[test]
	fn test_stackunderflow() {
		let mut cpu = Cpu::default();
		assert_eq!(
			cpu.execute(Instruction::Pop { register: R0 }),
			Err(CpuError::StackUnderflow)
		);
	}

	#[test]
	fn test_execute_jmp() {
		let mut cpu = Cpu::default();
		cpu.execute(Instruction::Jmp { address: 42.into() })
			.unwrap();
		assert_eq!(cpu.program_counter, 42);
	}

	#[test]
	fn test_register_pair() {
		let n: u16 = 0xFEFF;
		let mut cpu = Cpu::default();
		// R16:R17, little endian
		let register_pair = RegisterPair16::new(R16).unwrap();
		cpu.write_register_pair16(register_pair, n);
		assert_eq!(cpu.read_register(R16), 0xFF);
		assert_eq!(cpu.read_register(R17), 0xFE);
		assert_eq!(cpu.read_register_pair16(register_pair), n);
	}

	#[test]
	fn test_breakpoint() {
		let program = vec![
			Instruction::Ldi {
				register: UpperRegister::R16,
				value: 42.into(),
			},
			Instruction::Ldi {
				register: UpperRegister::R16,
				value: 42.into(),
			},
			// breakpoint will be set here
			Instruction::Ldi {
				register: UpperRegister::R16,
				value: 42.into(),
			},
			Instruction::Ldi {
				register: UpperRegister::R16,
				value: 42.into(),
			},
		];
		let mut cpu = Cpu::new(program);
		cpu.add_breakpoint(2);
		assert_eq!(cpu.step(), Ok(CpuStatus::Normal));
		assert_eq!(cpu.step(), Ok(CpuStatus::Normal));
		assert_eq!(cpu.step(), Ok(CpuStatus::BreakpointHit));
		// should not continue execution after breakpoint!
		assert_eq!(cpu.step(), Ok(CpuStatus::BreakpointHit));
	}
}
