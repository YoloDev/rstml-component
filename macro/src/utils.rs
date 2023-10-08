use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, ItemImpl, ItemStruct};

pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
	let input: syn::ItemFn = match syn::parse2(input) {
		Ok(input) => input,
		Err(err) => return err.to_compile_error(),
	};
	let struct_name: syn::Ident = match syn::parse2(attr) {
		Ok(struct_name) => struct_name,
		Err(err) => return err.to_compile_error(),
	};
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
	impl_generics.where_clause = None; // this gets copied to impl so remove it for struct<>

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
