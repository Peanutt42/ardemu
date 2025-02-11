use ardemu_core::{Instruction, Register};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

fn quote_register(register: Register) -> proc_macro2::TokenStream {
	match register {
		Register::R0 => quote! { ardemu_core::Register::R0 },
		Register::R1 => quote! { ardemu_core::Register::R1 },
		Register::R2 => quote! { ardemu_core::Register::R2 },
		Register::R3 => quote! { ardemu_core::Register::R3 },
		Register::R4 => quote! { ardemu_core::Register::R4 },
		Register::R5 => quote! { ardemu_core::Register::R5 },
		Register::R6 => quote! { ardemu_core::Register::R6 },
		Register::R7 => quote! { ardemu_core::Register::R7 },
		Register::R8 => quote! { ardemu_core::Register::R8 },
		Register::R9 => quote! { ardemu_core::Register::R9 },
		Register::R10 => quote! { ardemu_core::Register::R10 },
		Register::R11 => quote! { ardemu_core::Register::R11 },
		Register::R12 => quote! { ardemu_core::Register::R12 },
		Register::R13 => quote! { ardemu_core::Register::R13 },
		Register::R14 => quote! { ardemu_core::Register::R14 },
		Register::R15 => quote! { ardemu_core::Register::R15 },
		Register::R16 => quote! { ardemu_core::Register::R16 },
		Register::R17 => quote! { ardemu_core::Register::R17 },
		Register::R18 => quote! { ardemu_core::Register::R18 },
		Register::R19 => quote! { ardemu_core::Register::R19 },
		Register::R20 => quote! { ardemu_core::Register::R20 },
		Register::R21 => quote! { ardemu_core::Register::R21 },
		Register::R22 => quote! { ardemu_core::Register::R22 },
		Register::R23 => quote! { ardemu_core::Register::R23 },
		Register::R24 => quote! { ardemu_core::Register::R24 },
		Register::R25 => quote! { ardemu_core::Register::R25 },
		Register::R26 => quote! { ardemu_core::Register::R26 },
		Register::R27 => quote! { ardemu_core::Register::R27 },
		Register::R28 => quote! { ardemu_core::Register::R28 },
		Register::R29 => quote! { ardemu_core::Register::R29 },
		Register::R30 => quote! { ardemu_core::Register::R30 },
		Register::R31 => quote! { ardemu_core::Register::R31 },
	}
}

fn quote_instruction(instruction: Instruction) -> proc_macro2::TokenStream {
	match instruction {
		Instruction::Nop => quote! {
			ardemu_core::Instruction::Nop
		},
		Instruction::Ldi { reg, value } => {
			let reg = quote_register(reg);
			quote! {
				ardemu_core::Instruction::Ldi { reg: #reg, value: #value }
			}
		}
		Instruction::Add { rd, rs } => {
			let rd = quote_register(rd);
			let rs = quote_register(rs);
			quote! {
				ardemu_core::Instruction::Add { rd: #rd, rs: #rs }
			}
		}
		Instruction::Jmp { offset } => quote! {
			ardemu_core::Instruction::Jmp { offset: #offset }
		},
		Instruction::Store { reg, addr } => {
			let reg = quote_register(reg);
			quote! {
				ardemu_core::Instruction::Store { reg: #reg, addr: #addr }
			}
		}
	}
}

#[proc_macro]
pub fn parse_asm(input: TokenStream) -> TokenStream {
	let asm_input = parse_macro_input!(input as LitStr).value();
	let expanded = match ardemu_core::parse_asm(&asm_input) {
		Ok(instructions) => {
			let instruction_tokens = instructions
				.into_iter()
				.map(quote_instruction)
				.collect::<Vec<_>>();

			quote! {
				[ #(#instruction_tokens),* ]
			}
		}
		Err(e) => {
			let compile_error_msg = format!("ASM parse error: {e:?}");

			quote! {
				compile_error!(#compile_error_msg)
			}
		}
	};

	expanded.into()
}

#[proc_macro]
pub fn include_asm(input: TokenStream) -> TokenStream {
	let asm_filepath = parse_macro_input!(input as LitStr).value();

	let current_dir = std::env::current_dir().unwrap();

	let asm_filepath = current_dir.join(asm_filepath);
	let asm_filepath_str = asm_filepath.to_str().unwrap();

	let expanded = match std::fs::read_to_string(&asm_filepath) {
		Ok(asm_file_contents) => {
			match ardemu_core::parse_asm(&asm_file_contents) {
				Ok(instructions) => {
					let instruction_tokens = instructions
						.into_iter()
						.map(quote_instruction)
						.collect::<Vec<_>>();

					quote! {
						{
							/// this will make the compiler recompile when the file changes
							const _RECOMPILE_IF_CHANGED_HANDLE: &str = include_str!(#asm_filepath_str);

							[ #(#instruction_tokens),* ]
						}
					}
				}
				Err(e) => {
					let compile_error_msg =
						format!("ASM parse error in file '{asm_filepath_str}': {e:?}");

					quote! {
						compile_error!(#compile_error_msg)
					}
				}
			}
		}
		Err(e) => {
			let compile_error_msg =
				format!("Failed to read ASM file in '{asm_filepath_str}': {e:?}");
			quote! {
				compile_error!(#compile_error_msg)
			}
		}
	};

	expanded.into()
}
