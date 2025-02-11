use ardemu_core::Instruction;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

fn quote_instruction(instruction: Instruction) -> proc_macro2::TokenStream {
	match instruction {
		Instruction::Nop => quote! {
			ardemu_core::Instruction::Nop
		},
		Instruction::Ldi { reg, value } => quote! {
			ardemu_core::Instruction::Ldi { reg: #reg, value: #value }
		},
		Instruction::Add { rd, rs } => quote! {
			ardemu_core::Instruction::Add { rd: #rd, rs: #rs }
		},
		Instruction::Jmp { offset } => quote! {
			ardemu_core::Instruction::Jmp { offset: #offset }
		},
		Instruction::Store { reg, addr } => quote! {
			ardemu_core::Instruction::Store { reg: #reg, addr: #addr }
		},
	}
}

#[proc_macro]
pub fn parse_asm(input: TokenStream) -> TokenStream {
	let asm_input = parse_macro_input!(input as LitStr).value();
	let instrucitons =
		ardemu_core::parse_asm(&asm_input).unwrap_or_else(|e| panic!("ASM parse error: {e:?}"));

	let instruction_tokens = instrucitons
		.into_iter()
		.map(quote_instruction)
		.collect::<Vec<_>>();

	let expanded = quote! {
		[ #(#instruction_tokens),* ]
	};

	expanded.into()
}

#[proc_macro]
pub fn include_asm(input: TokenStream) -> TokenStream {
	let asm_filepath = parse_macro_input!(input as LitStr).value();

	let current_dir = std::env::current_dir().unwrap();

	let asm_filepath = current_dir.join(asm_filepath);
	let asm_filepath_str = asm_filepath.to_str().unwrap();

	let asm_file_contents = std::fs::read_to_string(&asm_filepath)
		.unwrap_or_else(|e| panic!("Failed to read ASM file in '{asm_filepath_str}': {e:?}"));

	let instrucitons = ardemu_core::parse_asm(&asm_file_contents)
		.unwrap_or_else(|e| panic!("ASM parse error in file '{asm_filepath_str}': {e:?}"));

	let instruction_tokens = instrucitons
		.into_iter()
		.map(quote_instruction)
		.collect::<Vec<_>>();

	let expanded = quote! {
		{
			/// this will make the compiler recompile when the file changes
			const _RECOMPILE_IF_CHANGED_HANDLE: &str = include_str!(#asm_filepath_str);

			[ #(#instruction_tokens),* ]
		}
	};

	expanded.into()
}
