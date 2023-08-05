use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::Ident;
use syn::{parse::Parse, Expr, Token};

use crate::template::Template;

struct WriteHtmlExpr {
	writer: Expr,
	#[allow(dead_code)] // used for parsing
	comma: Token![,],
	template: Template,
}

impl Parse for WriteHtmlExpr {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Self {
			writer: input.parse()?,
			comma: input.parse()?,
			template: input.parse()?,
		})
	}
}

impl ToTokens for WriteHtmlExpr {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self {
			writer,
			comma: _,
			template,
		} = self;

		let formatter = Ident::new("__html", Span::call_site());
		let template = template.with_formatter(&formatter);

		tokens.extend(quote!({
			use ::rstml_component::HtmlFormatter;
			#writer.write_content(
				|#formatter: &mut ::rstml_component::HtmlFormatter| -> ::std::fmt::Result {
					#template
					Ok(())
				},
			)
		}));
	}
}

pub fn write_html(input: TokenStream) -> TokenStream {
	let expr: WriteHtmlExpr = match syn::parse2(input) {
		Ok(expr) => expr,
		Err(err) => return err.to_compile_error(),
	};

	expr.into_token_stream()
}

struct HtmlExpr {
	template: Template,
	should_move: bool,
}

impl Parse for HtmlExpr {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let template = input.parse()?;

		Ok(Self {
			template,
			should_move: false,
		})
	}
}

impl ToTokens for HtmlExpr {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let formatter = Ident::new("__html", Span::call_site());
		let template = self.template.with_formatter(&formatter);
		let move_token = self.should_move.then(|| quote!(move));

		tokens.extend(quote! {
			#move_token |#formatter: &mut ::rstml_component::HtmlFormatter| -> ::std::fmt::Result {
				#template
				Ok(())
			}
		})
	}
}

pub fn html(input: TokenStream, should_move: bool) -> TokenStream {
	let mut template: HtmlExpr = match syn::parse2(input) {
		Ok(expr) => expr,
		Err(err) => return err.to_compile_error(),
	};

	template.should_move = should_move;
	template.into_token_stream()
}
