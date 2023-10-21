use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{parse::Parse, Ident, Token, punctuated::Punctuated, FnArg, Field, GenericParam, token::{Comma, Brace, Bracket, Paren}, TypeParam, Type, Generics, PatIdent, TypeImplTrait, Visibility, TypeTuple, ReturnType};
use proc_macro2_diagnostics::Diagnostic;

struct AttrParser {
	vis: syn::Visibility,
	name: syn::Ident,
}

impl Parse for AttrParser {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Self {
			vis: input.parse()?,
			name: input.parse()?,
		})
	}
}

fn create_not_used_ident(used: Punctuated<GenericParam, Comma>, ident_suggestion: Option<Ident>) -> Ident {
	let mut ident = ident_suggestion.unwrap_or_else(|| Ident::new("V", Span::call_site()));
	while used.iter().any(|used| {if let GenericParam::Type(type_param) = used {type_param.ident == ident} else {false}}/* used.ident == Some(ident.clone()) */) {
		ident = Ident::new(&format!("{}T", ident), Span::call_site());
	}
	ident
}

trait Surround: Clone {
	fn trait_surround(self, tokens: TokenStream) -> TokenStream;
}

impl Surround for Brace {
	fn trait_surround(self, tokens: TokenStream) -> TokenStream {
		quote!({#tokens})
	}
}

impl Surround for Bracket {
	fn trait_surround(self, tokens: TokenStream) -> TokenStream {
		quote!([#tokens])
	}
}

impl Surround for Paren {
	fn trait_surround(self, tokens: TokenStream) -> TokenStream {
		quote!((#tokens))
	}
}

fn prefix_type(ty: &mut Type, prefix: TokenStream) {
	let type_tokens = ty.into_token_stream();
	*ty = Type::Verbatim(quote!(#prefix #type_tokens))
}

fn suffix_type(ty: &mut Type, suffix: TokenStream) {
	let type_tokens = ty.into_token_stream();
	*ty = Type::Verbatim(quote!(#type_tokens #suffix))
}

fn surround_type(ty: &mut Type, surround: impl Surround) {
	let type_tokens = ty.into_token_stream();
	*ty = Type::Verbatim(surround.trait_surround(type_tokens))
}

fn generate_generic(generics: Generics, pat_ident: PatIdent, impl_type: TypeImplTrait, vis: Visibility) -> (Field, TypeParam) {
	let type_ident = create_not_used_ident(generics.params.clone(), Some(Ident::new(pat_ident.ident.to_string().to_uppercase().as_str(), Span::call_site())));
	let type_param = TypeParam {
		attrs: vec![],
		ident: type_ident.clone(),
		colon_token: Some(Token![:](Span::call_site())),
		bounds: impl_type.bounds,
		eq_token: None,
		default: None,
	};
	let field = Field {
		attrs: vec![],
		vis,
		mutability: syn::FieldMutability::None,
		ident: Some(pat_ident.ident.clone()),
		ty: Type::Verbatim(type_ident.into_token_stream()),
		colon_token: Some(Token![:](Span::call_site())),
	};
	(field, type_param)
}

fn generate_normal_field(pat_ident: PatIdent, vis: Visibility, ty: Type) -> Field {
	Field {
		attrs: vec![],
		vis,
		mutability: syn::FieldMutability::None,
		ident: Some(pat_ident.ident.clone()),
		ty: Type::Verbatim(ty.into_token_stream()),
		colon_token: Some(Token![:](Span::call_site())),
	}
}

fn generate_one(ty: Type, generics: &mut Generics, pat_ident: PatIdent, vis: Visibility) -> Field {
	match ty {
		Type::ImplTrait(impl_type) => {
			let (field, type_param) = generate_generic(generics.clone(), pat_ident.clone(), impl_type, vis);
			generics.params.push(GenericParam::Type(type_param));
			field
		},
		Type::Array(array_type) => {
			let new_ty = *array_type.elem;
			let len = array_type.len;
			let mut field = generate_one(new_ty, generics, pat_ident, vis);
			suffix_type(&mut field.ty, quote!(; #len));
			surround_type(&mut field.ty, array_type.bracket_token);
			field
		},
		Type::Paren(paren_type) => {
			let new_ty = *paren_type.elem;
			let mut field = generate_one(new_ty, generics, pat_ident, vis);
			surround_type(&mut field.ty, paren_type.paren_token);
			field
		},
		Type::Ptr(ptr_type) => {
			let new_ty = *ptr_type.elem;
			let mut prefix_tokens = ptr_type.star_token.into_token_stream();
			if let Some(mutability) = ptr_type.mutability {
				prefix_tokens.extend(mutability.into_token_stream());
			}
			if let Some(const_token) = ptr_type.const_token {
				prefix_tokens.extend(const_token.into_token_stream());
			}
			let mut field = generate_one(new_ty, generics, pat_ident, vis);
			prefix_type(&mut field.ty, prefix_tokens);
			field
		},
		Type::Reference(ref_type) => {
			let new_ty = *ref_type.elem;
			let mut prefix_tokens = ref_type.and_token.into_token_stream();
			if let Some(lifetime) = ref_type.lifetime {
				prefix_tokens.extend(lifetime.into_token_stream());
			}
			if let Some(mutability) = ref_type.mutability {
				prefix_tokens.extend(mutability.into_token_stream());
			}
			let mut field = generate_one(new_ty, generics, pat_ident, vis);
			prefix_type(&mut field.ty, prefix_tokens);
			field
		},
		Type::Slice(slice_type) => {
			let new_ty = *slice_type.elem;
			let mut field = generate_one(new_ty, generics, pat_ident, vis);
			surround_type(&mut field.ty, slice_type.bracket_token);
			field
		},
		Type::Tuple(typle_type) => {
			let TypeTuple {paren_token, elems} = typle_type;
			let mut types: Punctuated<Type, Comma> = Punctuated::new();
			for new_ty in elems.iter() {
				let field = generate_one(new_ty.clone(), generics, pat_ident.clone(), vis.clone());
				types.push(field.ty);
			}

			let new_ty = Type::Tuple(TypeTuple {
				paren_token,
				elems: types,
			});

			let field = generate_normal_field(pat_ident, vis, new_ty);
			field
		},
		_ => {
			let field = generate_normal_field(pat_ident, vis, ty);
			field
		},
	}
}

pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
	let mut diagnostics: Vec<Diagnostic> = vec![];
	// parse input
	let input: syn::ItemFn = match syn::parse2(input) {
		Ok(input) => input,
		Err(err) => return err.to_compile_error(),
	};
	let attr: AttrParser = match syn::parse2(attr) {
		Ok(attr) => attr,
		Err(err) => return err.to_compile_error(),
	};

	// check if input is valid
	if let Some(constness) = input.sig.constness {
		diagnostics.push(syn::Error::new_spanned(
			constness,
			"component function must not be const"
		).into())
	} else if let Some(asyncness) = input.sig.asyncness {
		diagnostics.push(syn::Error::new_spanned(
			asyncness,
			"component function must not be async",
		).into());
	} else if let Some(unsafety) = input.sig.unsafety {
		diagnostics.push(syn::Error::new_spanned(
			unsafety,
			"component function must not be unsafe",
		).into());
	} else if let Some(ref abi) = input.sig.abi {
		diagnostics.push(syn::Error::new_spanned(
			abi,
			"component function must not have an abi",
		).into());
	}

	let mut generics = input.sig.generics.clone();

	let mut fields: Punctuated<Field, syn::token::Comma> = Punctuated::new();

	for arg in input.sig.inputs.iter() {
		if let FnArg::Receiver(receiver) = arg {
			diagnostics.push(syn::Error::new_spanned(receiver.self_token, "component function must not have self argument").into());
		} else if let FnArg::Typed(pat_type) = arg {
			let pat = *pat_type.pat.clone();
			let ty = *pat_type.ty.clone();
			match pat {
				syn::Pat::Ident(pat_ident) => {
					let field = generate_one(ty, &mut generics, pat_ident, attr.vis.clone());
					fields.push(field);
				},
				_ => {
					diagnostics.push(syn::Error::new_spanned(pat, "couldn't parse function argument").into());
				},
			}
		}
	}

	let ident = attr.name.clone();
	let vis = attr.vis.clone();

	let generated_struct = quote!{
		#[derive(::rstml_component::HtmlComponent)]
		#vis struct #ident #generics {#fields}
	};

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let input_ident = input.sig.ident.clone();
	let mut fn_args = Vec::new();
	for field in fields.clone().iter() {
		let ident = field.ident.clone().unwrap();
		fn_args.push(quote!(self.#ident,));
	}

	let impl_block = quote!{
		impl #impl_generics ::rstml_component::HtmlContent for #ident #ty_generics #where_clause {
			fn fmt(self, formatter: &mut ::rstml_component::HtmlFormatter) -> std::fmt::Result {
				formatter.write_content(#input_ident (#(#fn_args)*))
			}
		}
	};

	let diagnostics = diagnostics
		.iter()
		.map(|d| d.clone().emit_as_item_tokens());
	quote! {
		#(#diagnostics)*
		#generated_struct
		#impl_block
		#input
	}
}
