use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields};

#[proc_macro_derive(ParseAsmInstruction, attributes(skip_parse_asm_instruction))]
pub fn parse_asm_instruction(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let enum_name = &input.ident;

	let variants = match &input.data {
		Data::Enum(DataEnum { variants, .. }) => variants,
		_ => panic!("DisplayInstruction can only be derived for enums"),
	};

	let arms = variants.iter().map(|variant| {
		let should_skip = variant
			.attrs
			.iter()
			.any(|attr| attr.path().is_ident("skip_parse_asm_instruction"));
		if should_skip {
			return quote! {};
		}

		let variant_ident = &variant.ident;
		let variant_name_upper = variant_ident.to_string().to_uppercase();

		match &variant.fields {
			Fields::Named(ref fields) if !fields.named.is_empty() => {
				let fields = &fields.named;
				let fields_len = fields.len();

				let parsed_fields = fields.iter().enumerate().map(|(index, field)| {
					let field_ident = field.ident.as_ref().unwrap();
					let field_type = &field.ty;
					quote! {
						#field_ident: #field_type::parse_operand(operands[#index])?
					}
				});

				quote! {
					#variant_name_upper => {
						let operands: &[&str; #fields_len] = operands.try_into()
							.map_err(|_| crate::AsmParseErrorType::InvalidArgumentCount {
								expected_count: #fields_len,
								actual_count: operands.len(),
							})?;

						Ok(#enum_name::#variant_ident { #(#parsed_fields),* })
					}
				}
			}
			Fields::Unit | Fields::Named(_) => {
				quote! {
					#variant_name_upper => {
						match operands.len() {
							0 => Ok(#enum_name::#variant_ident),
							_ => Err(crate::AsmParseErrorType::InvalidArgumentCount {
								expected_count: 0,
								actual_count: operands.len(),
							})
						}
					}
				}
			}
			_ => panic!("Unnamed variant fields are not supported"),
		}
	});

	let expanded = quote! {
		impl crate::ParseAsmInstruction for #enum_name {
			fn parse_asm_instruction(
				mnemonic: &str,
				operands: &[&str],
			) -> Result<Self, crate::AsmParseErrorType> {
				match mnemonic.to_uppercase().as_str() {
					#(#arms)*,
					_ => Err(crate::AsmParseErrorType::InvalidInstruction(mnemonic.to_string())),
				}
			}
		}
	};

	TokenStream::from(expanded)
}
