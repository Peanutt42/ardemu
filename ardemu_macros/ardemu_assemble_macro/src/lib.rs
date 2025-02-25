use proc_macro::TokenStream;
use quote::quote;
use self_rust_tokenize::SelfRustTokenize;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn assemble(input: TokenStream) -> TokenStream {
	let asm_input = parse_macro_input!(input as LitStr).value();
	let expanded = match ardemu_core::assemble(&asm_input) {
		Ok(instructions) => {
			let instruction_tokens = instructions
				.into_iter()
				.map(|instr| instr.to_tokens())
				.collect::<Vec<_>>();
			quote! {
				{
					use ardemu_core::{Instruction, Register, UpperRegister, WordRegister, RegisterPair16, Imm3, Imm8, Imm16};

					[ #(#instruction_tokens),* ]
				}
			}
		}
		Err(e) => {
			let compile_error_msg = format!("failed to assemble: {e:?}");

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
			match ardemu_core::assemble(&asm_file_contents) {
				Ok(instructions) => {
					let instruction_tokens = instructions
						.into_iter()
						.map(|instr| instr.to_tokens())
						.collect::<Vec<_>>();

					quote! {
						{
							use ardemu_core::{Instruction, Register, UpperRegister, WordRegister, RegisterPair16, Imm3, Imm8, Imm16};

							/// this will make the compiler recompile when the file changes
							const _RECOMPILE_IF_CHANGED_HANDLE: &str = include_str!(#asm_filepath_str);

							[ #(#instruction_tokens),* ]
						}
					}
				}
				Err(e) => {
					let compile_error_msg =
						format!("failed to assemble '{asm_filepath_str}': {e:?}");

					quote! {
						compile_error!(#compile_error_msg)
					}
				}
			}
		}
		Err(e) => {
			let compile_error_msg =
				format!("failed to read assembly file in '{asm_filepath_str}': {e:?}");
			quote! {
				compile_error!(#compile_error_msg)
			}
		}
	};

	expanded.into()
}
