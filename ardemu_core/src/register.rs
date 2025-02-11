#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
	R0,
	R1,
	R2,
	R3,
	R4,
	R5,
	R6,
	R7,
	R8,
	R9,
	R10,
	R11,
	R12,
	R13,
	R14,
	R15,
	R16,
	R17,
	R18,
	R19,
	R20,
	R21,
	R22,
	R23,
	R24,
	R25,
	R26,
	R27,
	R28,
	R29,
	R30,
	R31,
}

impl Register {
	/// this will not fail, as there are only 32 registers possible, enforced by the type
	pub fn get_from(&self, registers: &[u8; 32]) -> u8 {
		registers[*self as usize]
	}

	/// this will not fail, as there are only 32 registers possible, enforced by the type
	pub fn set_in(&self, registers: &mut [u8; 32], value: u8) {
		registers[*self as usize] = value;
	}
}

impl TryFrom<u8> for Register {
	type Error = ();

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0 => Ok(Register::R0),
			1 => Ok(Register::R1),
			2 => Ok(Register::R2),
			3 => Ok(Register::R3),
			4 => Ok(Register::R4),
			5 => Ok(Register::R5),
			6 => Ok(Register::R6),
			7 => Ok(Register::R7),
			8 => Ok(Register::R8),
			9 => Ok(Register::R9),
			10 => Ok(Register::R10),
			11 => Ok(Register::R11),
			12 => Ok(Register::R12),
			13 => Ok(Register::R13),
			14 => Ok(Register::R14),
			15 => Ok(Register::R15),
			16 => Ok(Register::R16),
			17 => Ok(Register::R17),
			18 => Ok(Register::R18),
			19 => Ok(Register::R19),
			20 => Ok(Register::R20),
			21 => Ok(Register::R21),
			22 => Ok(Register::R22),
			23 => Ok(Register::R23),
			24 => Ok(Register::R24),
			25 => Ok(Register::R25),
			26 => Ok(Register::R26),
			27 => Ok(Register::R27),
			28 => Ok(Register::R28),
			29 => Ok(Register::R29),
			30 => Ok(Register::R30),
			31 => Ok(Register::R31),
			_ => Err(()),
		}
	}
}

impl std::fmt::Display for Register {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Register::R0 => write!(f, "r0"),
			Register::R1 => write!(f, "r1"),
			Register::R2 => write!(f, "r2"),
			Register::R3 => write!(f, "r3"),
			Register::R4 => write!(f, "r4"),
			Register::R5 => write!(f, "r5"),
			Register::R6 => write!(f, "r6"),
			Register::R7 => write!(f, "r7"),
			Register::R8 => write!(f, "r8"),
			Register::R9 => write!(f, "r9"),
			Register::R10 => write!(f, "r10"),
			Register::R11 => write!(f, "r11"),
			Register::R12 => write!(f, "r12"),
			Register::R13 => write!(f, "r13"),
			Register::R14 => write!(f, "r14"),
			Register::R15 => write!(f, "r15"),
			Register::R16 => write!(f, "r16"),
			Register::R17 => write!(f, "r17"),
			Register::R18 => write!(f, "r18"),
			Register::R19 => write!(f, "r19"),
			Register::R20 => write!(f, "r20"),
			Register::R21 => write!(f, "r21"),
			Register::R22 => write!(f, "r22"),
			Register::R23 => write!(f, "r23"),
			Register::R24 => write!(f, "r24"),
			Register::R25 => write!(f, "r25"),
			Register::R26 => write!(f, "r26"),
			Register::R27 => write!(f, "r27"),
			Register::R28 => write!(f, "r28"),
			Register::R29 => write!(f, "r29"),
			Register::R30 => write!(f, "r30"),
			Register::R31 => write!(f, "r31"),
		}
	}
}
