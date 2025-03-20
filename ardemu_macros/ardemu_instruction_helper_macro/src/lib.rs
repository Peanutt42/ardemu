use proc_macro::TokenStream;
use quote::quote;
use syn::{
	parse_macro_input, punctuated::Punctuated, token::Comma, Data, DataEnum, DeriveInput, Field,
	Fields,
};

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

/// Implements the 'std::fmt::Display' trait for the Instruction enum.
/// Variant names are converted to uppercase and variant fields are displayed as parameters.
#[proc_macro_derive(DisplayInstruction)]
pub fn derive_display_instruction(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let enum_name = &input.ident;

	let variants = match &input.data {
		Data::Enum(DataEnum { variants, .. }) => variants,
		_ => panic!("DisplayInstruction can only be derived for enums"),
	};

	let arms = variants.iter().map(|variant| {
		let variant_ident = &variant.ident;
		let fields = match &variant.fields {
			Fields::Named(fields) => &fields.named,
			Fields::Unit => &Punctuated::new(),
			_ => panic!("Unnamed variant fields are not supported"),
		};

		let variant_name_upper = variant_ident.to_string().to_uppercase();

		let placeholders: Vec<String> = fields
			.iter()
			.map(|field| {
				let field_ident = field.ident.as_ref().unwrap().to_string();
				format!("{{{0}}}", field_ident)
			})
			.collect();

		let format_str = if placeholders.is_empty() {
			variant_name_upper
		} else {
			format!("{variant_name_upper} {}", placeholders.join(", "))
		};

		let field_patterns = fields.iter().map(|field| {
			let ident = &field.ident;
			quote! { ref #ident }
		});

		quote! {
			#enum_name::#variant_ident { #(#field_patterns),* } => write!(f, #format_str),
		}
	});

	let expanded = quote! {
		impl std::fmt::Display for #enum_name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match self {
					#(#arms)*
				}
			}
		}
	};

	TokenStream::from(expanded)
}

fn is_any_regsiter_type(ty: &syn::Type) -> bool {
	is_register_type(ty)
		|| is_upper_register_type(ty)
		|| is_register_address_type(ty)
		|| is_word_register_type(ty)
		|| is_lower_even_register_type(ty)
		|| is_pointer_register_type(ty)
}
fn is_register_type(ty: &syn::Type) -> bool {
	ty == &syn::parse_str::<syn::Type>("Register").unwrap()
}
fn is_upper_register_type(ty: &syn::Type) -> bool {
	ty == &syn::parse_str::<syn::Type>("UpperRegister").unwrap()
}
fn is_register_address_type(ty: &syn::Type) -> bool {
	ty == &syn::parse_str::<syn::Type>("RegisterAddress").unwrap()
}
fn is_word_register_type(ty: &syn::Type) -> bool {
	ty == &syn::parse_str::<syn::Type>("WordRegister").unwrap()
}
fn is_lower_even_register_type(ty: &syn::Type) -> bool {
	ty == &syn::parse_str::<syn::Type>("LowerEvenRegister").unwrap()
}
fn is_pointer_register_type(ty: &syn::Type) -> bool {
	ty == &syn::parse_str::<syn::Type>("PointerRegister").unwrap()
}

fn parse_registers(
	fields: &Punctuated<Field, Comma>,
	filter_callback: impl Fn(&syn::Type) -> bool,
	quote_callback: impl Fn(&syn::Ident) -> proc_macro2::TokenStream,
	output_registers: &mut Vec<proc_macro2::TokenStream>,
) {
	output_registers.extend(fields.iter().filter_map(|field| {
		field.ident.as_ref().and_then(|ident| {
			if filter_callback(&field.ty) {
				Some(quote_callback(ident))
			} else {
				None
			}
		})
	}));
}

#[proc_macro_derive(ReferencedRegisters)]
pub fn referenced_registers_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = &input.ident;

	let variants = match &input.data {
		Data::Enum(data) => &data.variants,
		_ => panic!("ReferencedRegisters can only be derived for enums"),
	};

	let match_arms = variants.iter().map(|variant| {
		let variant_name = &variant.ident;
		let fields = match &variant.fields {
			Fields::Named(fields) => &fields.named,
			Fields::Unnamed(fields) => &fields.unnamed,
			Fields::Unit => &Punctuated::new(),
		};

		let mut registers = Vec::new();
		parse_registers(
			fields,
			is_register_type,
			|ident| quote! { *#ident },
			&mut registers,
		);
		parse_registers(
			fields,
			is_upper_register_type,
			|ident| {
				quote! {
					(*#ident).into()
				}
			},
			&mut registers,
		);
		parse_registers(
			fields,
			is_register_address_type,
			|ident| {
				quote! {
					(*#ident).into()
				}
			},
			&mut registers,
		);
		parse_registers(
			fields,
			is_word_register_type,
			|ident| {
				quote! {
					(*#ident).into(), #ident.get_higher_uneven_register()
				}
			},
			&mut registers,
		);
		parse_registers(
			fields,
			is_lower_even_register_type,
			|ident| {
				quote! {
					(*#ident).into(), #ident.get_higher_uneven_register()
				}
			},
			&mut registers,
		);
		parse_registers(
			fields,
			is_pointer_register_type,
			|ident| {
				quote! {
					(*#ident).into(), #ident.get_higher_uneven_register()
				}
			},
			&mut registers,
		);
		// Mul instruction sets result into r1:r0
		if variant_name.to_string().to_uppercase() == "MUL" {
			registers.push(quote! {
				crate::Register::R0, crate::Register::R1
			})
		}

		let mut field_idents = Vec::with_capacity(fields.len());
		let mut fields_ignored = false;
		for (index, field) in fields.iter().enumerate() {
			if is_any_regsiter_type(&field.ty) {
				let ident = field.ident.as_ref().unwrap();
				field_idents.push(quote! {
					#ident
				});
			} else {
				fields_ignored = true;
			}
			if fields_ignored && index == fields.len() - 1 {
				field_idents.push(quote! {
					..
				});
			}
		}

		quote! {
			#name::#variant_name { #(#field_idents),* } => vec![#(#registers),*],
		}
	});

	let expanded = quote! {
		impl #name {
			pub fn get_referenced_registers(&self) -> Vec<Register> {
				match self {
					#(#match_arms)*
					_ => vec![],
				}
			}
		}
	};

	TokenStream::from(expanded)
}
