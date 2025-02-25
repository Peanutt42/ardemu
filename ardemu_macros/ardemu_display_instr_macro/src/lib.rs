use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Data, DataEnum, DeriveInput, Fields};

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
