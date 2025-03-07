use std::collections::HashMap;

use ardemu_core::{Program, WordAddress};
use proc_macro::TokenStream;
use quote::quote;
use self_rust_tokenize::SelfRustTokenize;
use syn::{parse_macro_input, LitStr};

fn tokenize_debug_symbol_table(
	debug_symbol_table: HashMap<WordAddress, String>,
) -> proc_macro2::TokenStream {
	let address_symbol_list = debug_symbol_table.into_iter().map(|(address, symbol)| {
		let address_tokens = address.to_tokens();
		quote! {
			(#address_tokens, String::from(#symbol))
		}
	});

	quote! {
		HashMap::from([
			#(#address_symbol_list),*
		])
	}
}

fn tokenize_program(program: Program) -> proc_macro2::TokenStream {
	let program_address_instruction_map_tokens =
		program.program_address_instruction_map.to_tokens();
	let debug_symbol_table_tokens = tokenize_debug_symbol_table(program.debug_symbol_table);

	quote! {
		{
			use std::collections::HashMap;

			Program {
				program_address_instruction_map: #program_address_instruction_map_tokens,
				debug_symbol_table: #debug_symbol_table_tokens,
			}
		}
	}
}

#[proc_macro]
pub fn assemble(input: TokenStream) -> TokenStream {
	let asm_input = parse_macro_input!(input as LitStr).value();
	let expanded = match ardemu_core::assemble(&asm_input) {
		Ok(program) => {
			let program_tokens = tokenize_program(program);
			quote! {
				{
					use ardemu_core::{Program, Instruction, Register, UpperRegister, WordRegister, WordAddress, WordOffset8, WordOffset16, LowerEvenRegister, Imm3, Imm8, Imm16};

					#program_tokens
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
				Ok(program) => {
					let program_tokens = tokenize_program(program);

					quote! {
						{
							use ardemu_core::{Program, Instruction, Register, UpperRegister, WordRegister, WordAddress, WordOffset8, WordOffset16, LowerEvenRegister, Imm3, Imm8, Imm16};

							/// this will make the compiler recompile when the file changes
							const _RECOMPILE_IF_CHANGED_HANDLE: &str = include_str!(#asm_filepath_str);

							#program_tokens
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
