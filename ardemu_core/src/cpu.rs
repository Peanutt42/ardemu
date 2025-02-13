use crate::{CpuError, Instruction, Register};

const SRAM_SIZE: usize = 2048;

#[derive(Debug, Clone)]
pub struct Cpu {
	program: Box<[Instruction]>,
	pub registers: [u8; 8],
	pub program_counter: u16,
	/// address pointer
	pub hl: u16,
	pub sram: [u8; SRAM_SIZE], // SRAM (2KB)
}

impl Cpu {
	pub fn new(program: impl Into<Box<[Instruction]>>) -> Self {
		Self {
			program: program.into(),
			registers: [0; Register::COUNT],
			program_counter: 0,
			hl: 0,
			sram: [0u8; SRAM_SIZE],
		}
	}

	/// resets everything except the last loaded program
	pub fn reset(&mut self) {
		self.program_counter = 0;
		self.hl = 0;
		self.registers = [0; Register::COUNT];
		self.sram = [0u8; SRAM_SIZE];
	}

	pub fn get_current_instruction(&self) -> Option<Instruction> {
		self.program.get(self.program_counter as usize).copied()
	}

	fn get_register_value(&self, reg: Register) -> u8 {
		reg.get_from(&self.registers)
	}

	fn set_register_value(&mut self, reg: Register, value: u8) {
		reg.set_in(&mut self.registers, value);
	}

	fn read_ram(&self, address: u16) -> Result<u8, CpuError> {
		let ram = self
			.sram
			.get(address as usize)
			.ok_or(CpuError::InvalidRamAddress { addr: address })?;
		Ok(*ram)
	}
	fn write_ram(&mut self, address: u16, value: u8) -> Result<(), CpuError> {
		let mut_ram = self
			.sram
			.get_mut(address as usize)
			.ok_or(CpuError::InvalidRamAddress { addr: address })?;
		*mut_ram = value;
		Ok(())
	}

	pub fn execute(&mut self, instruction: Instruction) -> Result<(), CpuError> {
		match instruction {
			Instruction::Mw { reg, value } => {
				let value = value.imm8_or_else(|reg| self.get_register_value(reg));
				self.set_register_value(reg, value);
				self.program_counter += 1;
			}
			Instruction::Lw { register, address } => {
				let value = self.read_ram(address.imm16_or_hl(self.hl))?;
				self.set_register_value(register, value);
				self.program_counter += 1;
			}
			Instruction::Sw { address, register } => {
				let address = address.imm16_or_hl(self.hl);
				let value = self.get_register_value(register);
				self.write_ram(address, value)?;
				self.program_counter += 1;
			}
			Instruction::Lda { address } => {
				self.hl = address.0;
				self.program_counter += 1;
			}
			Instruction::Jnz { value } => {
				let value = value.imm8_or_else(|reg| self.get_register_value(reg));
				if value != 0 {
					self.program_counter = self.hl;
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Add { reg, value } => {
				let reg_value = self.get_register_value(reg);
				let other_value = value.imm8_or_else(|reg| self.get_register_value(reg));
				self.set_register_value(reg, reg_value.wrapping_add(other_value));
				self.program_counter += 1;
			}
			Instruction::Sub { reg, value } => {
				let reg_value = self.get_register_value(reg);
				let other_value = value.imm8_or_else(|reg| self.get_register_value(reg));
				self.set_register_value(reg, reg_value.wrapping_sub(other_value));
				self.program_counter += 1;
			}
			Instruction::And { reg, value } => {
				let reg_value = self.get_register_value(reg);
				let other_value = value.imm8_or_else(|reg| self.get_register_value(reg));
				self.set_register_value(reg, reg_value & other_value);
				self.program_counter += 1;
			}
			Instruction::Or { reg, value } => {
				let reg_value = self.get_register_value(reg);
				let other_value = value.imm8_or_else(|reg| self.get_register_value(reg));
				self.set_register_value(reg, reg_value | other_value);
				self.program_counter += 1;
			}
			Instruction::Nor { reg, value } => {
				let reg_value = self.get_register_value(reg);
				let other_value = value.imm8_or_else(|reg| self.get_register_value(reg));
				self.set_register_value(reg, !(reg_value | other_value));
				self.program_counter += 1;
			}
		}

		Ok(())
	}

	/// Result::Ok(bool) returns false if the program has finished
	pub fn step(&mut self) -> Result<bool, CpuError> {
		match self.get_current_instruction() {
			Some(instruction) => {
				self.execute(instruction)?;
				Ok(true)
			}
			None => Ok(false),
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
mod tests {
	use crate::{register::HlOrImm16, A, B};

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
}
