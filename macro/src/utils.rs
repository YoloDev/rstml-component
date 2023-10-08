use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{parse_quote, ItemImpl, ItemStruct, parse::Parse, Ident};

use crate::template::Template;

pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
	// parse input
	let input: syn::ItemFn = match syn::parse2(input) {
		Ok(input) => input,
		Err(err) => return err.to_compile_error(),
	};
	let struct_name: syn::Ident = match syn::parse2(attr) {
		Ok(struct_name) => struct_name,
		Err(err) => return err.to_compile_error(),
	};
	// check if input is valid
	assert!(
		input.sig.constness.is_none(),
		"component function must not be const"
	);
	assert!(
		input.sig.asyncness.is_none(),
		"component function must not be async"
	);
	assert!(
		input.sig.unsafety.is_none(),
		"component function must not be unsafe"
	);
	assert!(
		input.sig.abi.is_none(),
		"component function must not have an abi"
	);
	match input.sig.output {
		syn::ReturnType::Type(_, t) => {
			assert_eq!(
				t.into_token_stream().to_string().replace(" ", ""),
				"std::fmt::Result",
				"component function must return std::fmt::Result"
			)
		}
		_ => panic!("component function must return std::fmt::Result"),
	}

	let visibility = input.vis;

	let mut struct_args = Vec::new();
	let mut arg_vars = Vec::new();
	for arg in input.sig.inputs.iter() {
		match arg {
			syn::FnArg::Typed(pat) => {
				let ty = *pat.ty.clone();
				let ident = match *pat.pat.clone() {
					syn::Pat::Ident(pat_ident) => pat_ident,
					_ => panic!("component function must have typed arguments"),
				};
				arg_vars.push(quote!(let #ident = self.#ident;));
				struct_args.push(quote!(#visibility #ident: #ty,));
			}
			_ => panic!("component function must have typed arguments"),
		}
	}

	let mut gen_struct: ItemStruct = parse_quote! {
		#[derive(::rstml_component::HtmlComponent)]
		#visibility struct #struct_name {
			#(#struct_args)*
		}
	};
	gen_struct.generics = input.sig.generics.clone();

	// generics are copied to impl but not to impl ... for SomeStruct<T here there are no generics>
	// so make a Option<Generics> and use it in impl
	let mut impl_generics = input.sig.generics.clone();
	impl_generics.where_clause = None; // this gets copied to impl so remove it for Struct<>

	let fn_body = input.block;
	let mut impl_component: ItemImpl = parse_quote! {
		impl ::rstml_component::HtmlContent for #struct_name #impl_generics {
			fn fmt(self, formatter: &mut ::rstml_component::HtmlFormatter) -> std::fmt::Result {
				// convert self.* to let * = self.*;
				#(#arg_vars)*
				#fn_body
			}
		}
	};
	impl_component.generics = input.sig.generics.clone();

	quote! {
		#gen_struct
		#impl_component
	}
}

struct ComponentHtmlExpr {
	template: Template,
}

impl Parse for ComponentHtmlExpr {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Self {
			template: input.parse()?,
		})
	}
}

impl ToTokens for ComponentHtmlExpr {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self {
			template,
		} = self;

		let formatter = Ident::new("__html", Span::call_site());
		let template = template.with_formatter(&formatter);

		tokens.extend(quote!({
			let #formatter: &mut ::rstml_component::HtmlFormatter = formatter.as_mut();
			#formatter.write_content(
				|#formatter: &mut ::rstml_component::HtmlFormatter| -> ::std::fmt::Result {
					#template
					Ok(())
				},
			)
		}));
	}
}

pub fn component_html(input: TokenStream) -> TokenStream {
	let expr: ComponentHtmlExpr = match syn::parse2(input) {
		Ok(expr) => expr,
		Err(err) => return err.to_compile_error(),
	};

	expr.into_token_stream()
}
