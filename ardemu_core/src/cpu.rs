use std::collections::HashSet;

use crate::{CpuError, Instruction, Register};

/// 64 KB
const SRAM_SIZE: usize = 64 * 1024;
const STACK_START_ADDRESS: u16 = 0xFEFF;
const STACK_END_ADDRESS: u16 = 0xFC00;
const STACK_ADDRESS_RANGE: std::ops::Range<u16> = std::ops::Range {
	start: STACK_END_ADDRESS,
	end: STACK_START_ADDRESS + 1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CpuStatus {
	Normal,
	/// the current instruction was not run, as it was marked as a breakpoint
	BreakpointHit,
	ProgramFinished,
}

#[derive(Debug, Clone)]
pub struct Cpu {
	program: Box<[Instruction]>,
	registers: [u8; 8],
	program_counter: u16,
	stack_pointer: u16,
	/// address pointer
	hl: u16,
	sram: [u8; SRAM_SIZE], // SRAM (64KB)
	// contains the program address of the breakpoints
	breakpoints: HashSet<u16>,
}

impl Cpu {
	pub fn new(program: impl Into<Box<[Instruction]>>) -> Self {
		Self {
			program: program.into(),
			registers: [0; Register::COUNT],
			program_counter: 0,
			stack_pointer: STACK_START_ADDRESS,
			hl: 0,
			sram: [0u8; SRAM_SIZE],
			breakpoints: HashSet::new(),
		}
	}

	/// resets everything except the last loaded program
	pub fn reset(&mut self) {
		self.program_counter = 0;
		self.hl = 0;
		self.stack_pointer = 0;
		self.registers = [0; Register::COUNT];
		self.sram = [0u8; SRAM_SIZE];
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

	pub fn get_program_counter(&self) -> u16 {
		self.program_counter
	}

	pub fn get_current_instruction(&self) -> Option<Instruction> {
		self.program.get(self.program_counter as usize).copied()
	}

	pub fn read_register(&self, reg: Register) -> u8 {
		reg.read_from(&self.registers)
	}

	pub fn write_register(&mut self, reg: Register, value: u8) {
		reg.write_in(&mut self.registers, value);
	}

	fn read_ram(&self, address: u16) -> Result<u8, CpuError> {
		if STACK_ADDRESS_RANGE.contains(&address) {
			return Err(CpuError::InvalidRamAddress { addr: address });
		}

		let ram = self
			.sram
			.get(address as usize)
			.ok_or(CpuError::InvalidRamAddress { addr: address })?;
		Ok(*ram)
	}
	fn write_ram(&mut self, address: u16, value: u8) -> Result<(), CpuError> {
		if STACK_ADDRESS_RANGE.contains(&address) {
			return Err(CpuError::InvalidRamAddress { addr: address });
		}

		let mut_ram = self
			.sram
			.get_mut(address as usize)
			.ok_or(CpuError::InvalidRamAddress { addr: address })?;
		*mut_ram = value;
		Ok(())
	}
	fn push(&mut self, value: u8) -> Result<(), CpuError> {
		if self.stack_pointer <= STACK_END_ADDRESS {
			return Err(CpuError::StackOverflow);
		}
		self.sram[self.stack_pointer as usize] = value;
		self.stack_pointer -= 1;
		Ok(())
	}
	fn pop(&mut self) -> Result<u8, CpuError> {
		self.stack_pointer += 1;
		if self.stack_pointer > STACK_START_ADDRESS {
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
			Instruction::Mw { reg, value } => {
				let value = value.imm8_or_else(|reg| self.read_register(reg));
				self.write_register(reg, value);
				self.program_counter += 1;
			}
			Instruction::Lw { register, address } => {
				let value = self.read_ram(address.imm16_or_hl(self.hl))?;
				self.write_register(register, value);
				self.program_counter += 1;
			}
			Instruction::Sw { address, register } => {
				let address = address.imm16_or_hl(self.hl);
				let value = self.read_register(register);
				self.write_ram(address, value)?;
				self.program_counter += 1;
			}
			Instruction::Push { value } => {
				let value = value.imm8_or_else(|reg| self.read_register(reg));
				self.push(value)?;
				self.program_counter += 1;
			}
			Instruction::Pop { register } => {
				let value = self.pop()?;
				self.write_register(register, value);
				self.program_counter += 1;
			}
			Instruction::Lda { address } => {
				self.hl = address.0;
				self.program_counter += 1;
			}
			Instruction::Jnz { value } => {
				let value = value.imm8_or_else(|reg| self.read_register(reg));
				if value != 0 {
					self.program_counter = self.hl;
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Add { reg, value } => {
				let reg_value = self.read_register(reg);
				let other_value = value.imm8_or_else(|reg| self.read_register(reg));
				self.write_register(reg, reg_value.wrapping_add(other_value));
				self.program_counter += 1;
			}
			Instruction::Sub { reg, value } => {
				let reg_value = self.read_register(reg);
				let other_value = value.imm8_or_else(|reg| self.read_register(reg));
				self.write_register(reg, reg_value.wrapping_sub(other_value));
				self.program_counter += 1;
			}
			Instruction::And { reg, value } => {
				let reg_value = self.read_register(reg);
				let other_value = value.imm8_or_else(|reg| self.read_register(reg));
				self.write_register(reg, reg_value & other_value);
				self.program_counter += 1;
			}
			Instruction::Or { reg, value } => {
				let reg_value = self.read_register(reg);
				let other_value = value.imm8_or_else(|reg| self.read_register(reg));
				self.write_register(reg, reg_value | other_value);
				self.program_counter += 1;
			}
			Instruction::Nor { reg, value } => {
				let reg_value = self.read_register(reg);
				let other_value = value.imm8_or_else(|reg| self.read_register(reg));
				self.write_register(reg, !(reg_value | other_value));
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
}

impl Default for Cpu {
	fn default() -> Self {
		Self::new([])
	}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
	use crate::{
		register::HlOrImm16,
		Register::{C, D},
		A, B,
	};

	use super::*;

	#[test]
	fn test_execute_move() {
		let mut cpu = Cpu::default();
		cpu.execute(Instruction::Mw {
			reg: A,
			value: 42.into(),
		})
		.unwrap();
		assert_eq!(cpu.registers[0], 42);
	}

	#[test]
	fn test_execute_add_sub() {
		let mut cpu = Cpu::default();
		cpu.registers[0] = 42;
		cpu.registers[1] = 23;
		cpu.execute(Instruction::Add {
			reg: A,
			value: B.into(),
		})
		.unwrap();
		assert_eq!(cpu.registers[0], 42 + 23);
		cpu.registers[0] = 42;
		cpu.registers[1] = 23;
		cpu.execute(Instruction::Sub {
			reg: A,
			value: B.into(),
		})
		.unwrap();
		assert_eq!(cpu.registers[0], 42 - 23);
	}

	#[test]
	fn test_execute_push_pop() {
		let mut cpu = Cpu::default();
		cpu.write_register(A, 42);
		cpu.write_register(B, 15);
		cpu.execute(Instruction::Push { value: A.into() }).unwrap();
		cpu.execute(Instruction::Push { value: B.into() }).unwrap();
		cpu.write_register(A, 0);
		cpu.write_register(B, 0);
		cpu.execute(Instruction::Pop { register: B }).unwrap();
		cpu.execute(Instruction::Pop { register: A }).unwrap();
		assert_eq!(cpu.read_register(A), 42);
		assert_eq!(cpu.read_register(B), 15);
	}

	#[test]
	fn test_catch_write_directly_into_stack() {
		let mut cpu = Cpu::default();
		cpu.execute(Instruction::Mw {
			reg: A,
			value: 42.into(),
		})
		.unwrap();
		assert_eq!(
			cpu.execute(Instruction::Sw {
				address: (STACK_START_ADDRESS - 1).into(),
				register: A,
			}),
			Err(CpuError::InvalidRamAddress {
				addr: STACK_START_ADDRESS - 1
			})
		);
	}

	#[test]
	fn test_stackoverflow() {
		let mut cpu = Cpu::default();
		loop {
			match cpu.execute(Instruction::Push { value: 0.into() }) {
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
			cpu.execute(Instruction::Pop { register: A }),
			Err(CpuError::StackUnderflow)
		);
	}

	#[test]
	fn test_execute_and() {
		let mut cpu = Cpu::default();
		cpu.registers[0] = 42;
		cpu.registers[1] = 23;
		cpu.execute(Instruction::And {
			reg: A,
			value: B.into(),
		})
		.unwrap();
		assert_eq!(cpu.registers[0], 42 & 23);
	}

	#[test]
	fn test_execute_or() {
		let mut cpu = Cpu::default();
		cpu.registers[0] = 42;
		cpu.registers[1] = 23;
		cpu.execute(Instruction::Or {
			reg: A,
			value: B.into(),
		})
		.unwrap();
		assert_eq!(cpu.registers[0], 42 | 23);
	}

	#[test]
	fn test_execute_nor() {
		let mut cpu = Cpu::default();
		cpu.registers[0] = 42;
		cpu.registers[1] = 23;
		cpu.execute(Instruction::Nor {
			reg: A,
			value: B.into(),
		})
		.unwrap();
		assert_eq!(cpu.registers[0], !(42 | 23));
	}

	#[test]
	fn test_execute_lda_jnz() {
		let mut cpu = Cpu::default();
		cpu.execute(Instruction::Lda { address: 42.into() })
			.unwrap();
		cpu.execute(Instruction::Jnz { value: 1.into() }).unwrap();
		assert_eq!(cpu.program_counter, 42);
	}

	#[test]
	fn test_execute_lw_sw() {
		let magic_num = 42;
		let address = 0x123;

		let mut cpu = Cpu::default();
		cpu.registers[0] = magic_num;
		cpu.execute(Instruction::Sw {
			register: A,
			address: address.into(),
		})
		.unwrap();
		assert_eq!(cpu.read_ram(address).unwrap(), magic_num);
		cpu.registers[0] = 0;
		cpu.execute(Instruction::Lw {
			register: A,
			address: address.into(),
		})
		.unwrap();
		assert_eq!(cpu.registers[0], magic_num);
	}

	#[test]
	fn test_hl() {
		let mut cpu = Cpu::default();
		cpu.execute(Instruction::Lda {
			address: 0x123.into(),
		})
		.unwrap();
		assert_eq!(cpu.hl, 0x123);
		cpu.registers[0] = 42;
		cpu.execute(Instruction::Sw {
			register: A,
			address: HlOrImm16::Hl,
		})
		.unwrap();
		assert_eq!(cpu.read_ram(cpu.hl).unwrap(), 42);
		cpu.registers[0] = 0;
		cpu.execute(Instruction::Lw {
			register: A,
			address: HlOrImm16::Hl,
		})
		.unwrap();
		assert_eq!(cpu.registers[0], 42);
	}

	#[test]
	fn test_breakpoint() {
		let program = vec![
			Instruction::Mw {
				reg: A,
				value: 0.into(),
			},
			Instruction::Mw {
				reg: B,
				value: 1.into(),
			},
			// breakpoint will be set here
			Instruction::Mw {
				reg: C,
				value: 2.into(),
			},
			Instruction::Mw {
				reg: D,
				value: 3.into(),
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
