use crate::{CpuError, Instruction, Register};

const SRAM_SIZE: usize = 2048;

#[derive(Debug, Clone)]
pub struct Cpu {
	pub registers: [u8; 8],
	pub program_counter: u16,
	/// address pointer
	pub hl: u16,
	pub sram: [u8; SRAM_SIZE], // SRAM (2KB)
}

impl Cpu {
	pub fn new() -> Self {
		Self {
			registers: [0; Register::COUNT],
			program_counter: 0,
			hl: 0,
			sram: [0u8; SRAM_SIZE],
		}
	}

	pub fn get_current_instruction(&self, instructions: &[Instruction]) -> Option<Instruction> {
		instructions.get(self.program_counter as usize).copied()
	}

	fn get_register_value(&self, reg: Register) -> u8 {
		reg.get_from(&self.registers)
	}

	fn set_register_value(&mut self, reg: Register, value: u8) {
		reg.set_in(&mut self.registers, value);
	}

	fn set_ram(&mut self, addr: u32, value: u8) -> Result<(), CpuError> {
		let mut_ram = self
			.sram
			.get_mut(addr as usize)
			.ok_or(CpuError::InvalidRamAddress { addr })?;
		*mut_ram = value;
		Ok(())
	}

	pub fn execute(&mut self, instruction: Instruction) -> Result<(), CpuError> {
		match instruction {
			Instruction::Mw { reg, value } => {
				let value = value.imm_or_else(|reg| self.get_register_value(reg));
				self.set_register_value(reg, value);
				self.program_counter += 1;
			}
			Instruction::Lda { address } => {
				self.hl = address.0;
				self.program_counter += 1;
			}
			Instruction::Jnz { value } => {
				let value = value.imm_or_else(|reg| self.get_register_value(reg));
				if value != 0 {
					self.program_counter = self.hl;
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Add { reg, value } => {
				let reg_value = self.get_register_value(reg);
				let other_value = value.imm_or_else(|reg| self.get_register_value(reg));
				self.set_register_value(reg, reg_value.wrapping_add(other_value));
				self.program_counter += 1;
			}
			Instruction::And { reg, value } => {
				let reg_value = self.get_register_value(reg);
				let other_value = value.imm_or_else(|reg| self.get_register_value(reg));
				self.set_register_value(reg, reg_value & other_value);
				self.program_counter += 1;
			}
			Instruction::Or { reg, value } => {
				let reg_value = self.get_register_value(reg);
				let other_value = value.imm_or_else(|reg| self.get_register_value(reg));
				self.set_register_value(reg, reg_value | other_value);
				self.program_counter += 1;
			}
			Instruction::Nor { reg, value } => {
				let reg_value = self.get_register_value(reg);
				let other_value = value.imm_or_else(|reg| self.get_register_value(reg));
				self.set_register_value(reg, !(reg_value | other_value));
				self.program_counter += 1;
			}
		}

		Ok(())
	}
}

impl Default for Cpu {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
	use crate::{A, B};

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
	fn test_execute_add() {
		let mut cpu = Cpu::default();
		cpu.registers[0] = 42;
		cpu.registers[1] = 23;
		cpu.execute(Instruction::Add {
			reg: A,
			value: B.into(),
		})
		.unwrap();
		assert_eq!(cpu.registers[0], 42 + 23);
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
}
