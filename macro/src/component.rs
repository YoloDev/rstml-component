use proc_macro2::TokenStream;
use proc_macro2_diagnostics::SpanDiagnosticExt;
use quote::quote;
use syn::{spanned::Spanned, Fields, Item};

pub fn derive_html_component(input: TokenStream) -> TokenStream {
	let full_span = input.span();
	let input = match syn::parse2(input) {
		Ok(Item::Struct(item_struct)) => item_struct,
		Ok(_) => {
			return full_span
				.error("Derived `HtmlComponent`s must be structs")
				.emit_as_item_tokens()
		}
		Err(err) => return err.into_compile_error(),
	};

	if matches!(&input.fields, Fields::Unnamed(_)) {
		return input
			.fields
			.span()
			.error("Derived `HtmlComponent`s cannot have tuple fields")
			.emit_as_item_tokens();
	}

	let ident = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	quote! {
		#[automatically_derived]
		impl #impl_generics ::rstml_component::HtmlComponent for #ident #ty_generics #where_clause {
			type Content = Self;

			fn into_content(self) -> Self::Content {
				self
			}
		}
	}
}
