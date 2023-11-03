use proc_macro2::{Span, TokenStream};
use proc_macro2_diagnostics::{Diagnostic, Level};
use quote::{quote, ToTokens};
use syn::{
	parse::Parse,
	punctuated::Punctuated,
	token::{Brace, Bracket, Comma, Paren},
	Field, FnArg, GenericParam, Generics, Ident, Pat, Token, Type, TypeImplTrait, TypeParam,
	TypeTuple, Visibility, spanned::Spanned,
};

struct ComponentAttrs {
	vis: syn::Visibility,
	name: syn::Ident,
}

impl Parse for ComponentAttrs {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Self {
			vis: input.parse()?,
			name: input.parse()?,
		})
	}
}

fn create_unique_generic_ident(
	used: Punctuated<GenericParam, Comma>,
	ident_suggestion: Option<Ident>,
) -> Ident {
	let mut ident = ident_suggestion.unwrap_or_else(|| Ident::new("V", Span::call_site()));
	while used.iter().any(|used| {
		if let GenericParam::Type(type_param) = used {
			type_param.ident == ident
		} else {
			false
		}
	}) {
		ident = Ident::new(&format!("{}T", ident), Span::call_site());
	}
	ident
}

fn lowercase_ident(ident: Ident) -> Ident {
	Ident::new(&ident.to_string().to_lowercase(), ident.span())
}

fn is_field_ident_used(ident: &Ident, used: Vec<Pat>) -> bool {
	used.iter().any(|used| match used.clone() {
		Pat::Ident(ident_pat) => ident_pat.ident == *ident,
		Pat::Slice(slice_pat) => {
			let mut new_used = Vec::new();
			new_used.extend(slice_pat.elems);
			is_field_ident_used(ident, new_used)
		}
		Pat::TupleStruct(tuple_struct_pat) => {
			let mut new_used = Vec::new();
			new_used.extend(tuple_struct_pat.elems);
			is_field_ident_used(ident, new_used)
		}
		Pat::Struct(struct_pat) => {
			let mut new_used = Vec::new();
			new_used.extend(struct_pat.fields.iter().map(|field| *field.pat.clone()));
			is_field_ident_used(ident, new_used)
		}
		Pat::Tuple(tuple_pat) => {
			let mut new_used = Vec::new();
			new_used.extend(tuple_pat.elems);
			is_field_ident_used(ident, new_used)
		}
		_ => false,
	})
}

fn create_unique_field_ident(
	used: Punctuated<Field, Comma>,
	all_used: Punctuated<FnArg, Comma>,
	ident_suggestion: Option<Ident>,
) -> Ident {
	let mut ident =
		lowercase_ident(ident_suggestion.unwrap_or_else(|| Ident::new("arg", Span::call_site())));
	while used.iter().any(|used| used.ident == Some(ident.clone())) {
		ident = Ident::new(&format!("{}_", ident), Span::call_site());
	}
	let mut used = Vec::new();
	used.extend(all_used.iter().map(|arg| {
		if let FnArg::Typed(pat_type) = arg {
			*pat_type.pat.clone()
		} else {
			unreachable!()
		}
	}));
	while is_field_ident_used(&ident, used.clone()) {
		ident = Ident::new(&format!("{}_", ident), Span::call_site());
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

fn generate_generic(
	generics: Generics,
	ident: Ident,
	impl_type: TypeImplTrait,
) -> (Field, TypeParam) {
	let type_ident = create_unique_generic_ident(
		generics.params,
		Some(Ident::new(
			ident.to_string().to_uppercase().as_str(),
			Span::call_site(),
		)),
	);
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
		vis: Visibility::Public(Token![pub](Span::call_site())),
		mutability: syn::FieldMutability::None,
		ident: Some(ident),
		ty: Type::Verbatim(type_ident.into_token_stream()),
		colon_token: Some(Token![:](Span::call_site())),
	};
	(field, type_param)
}

fn generate_normal_field(ident: Ident, ty: Type) -> Field {
	Field {
		attrs: vec![],
		vis: Visibility::Public(Token![pub](Span::call_site())),
		mutability: syn::FieldMutability::None,
		ident: Some(ident),
		ty: Type::Verbatim(ty.into_token_stream()),
		colon_token: Some(Token![:](Span::call_site())),
	}
}

fn generate_one(ty: Type, generics: &mut Generics, ident: Ident) -> Field {
	match ty {
		Type::ImplTrait(impl_type) => {
			let (field, type_param) = generate_generic(generics.clone(), ident, impl_type);
			generics.params.push(GenericParam::Type(type_param));
			field
		}
		Type::Array(array_type) => {
			let new_ty = *array_type.elem;
			let len = array_type.len;
			let mut field = generate_one(new_ty, generics, ident);
			suffix_type(&mut field.ty, quote!(; #len));
			surround_type(&mut field.ty, array_type.bracket_token);
			field
		}
		Type::Paren(paren_type) => {
			let new_ty = *paren_type.elem;
			let mut field = generate_one(new_ty, generics, ident);
			surround_type(&mut field.ty, paren_type.paren_token);
			field
		}
		Type::Ptr(ptr_type) => {
			let new_ty = *ptr_type.elem;
			let mut prefix_tokens = ptr_type.star_token.into_token_stream();
			if let Some(mutability) = ptr_type.mutability {
				prefix_tokens.extend(mutability.into_token_stream());
			}
			if let Some(const_token) = ptr_type.const_token {
				prefix_tokens.extend(const_token.into_token_stream());
			}
			let mut field = generate_one(new_ty, generics, ident);
			prefix_type(&mut field.ty, prefix_tokens);
			field
		}
		Type::Reference(ref_type) => {
			let new_ty = *ref_type.elem;
			let mut prefix_tokens = ref_type.and_token.into_token_stream();
			if let Some(lifetime) = ref_type.lifetime {
				prefix_tokens.extend(lifetime.into_token_stream());
			}
			if let Some(mutability) = ref_type.mutability {
				prefix_tokens.extend(mutability.into_token_stream());
			}
			let mut field = generate_one(new_ty, generics, ident);
			prefix_type(&mut field.ty, prefix_tokens);
			field
		}
		Type::Slice(slice_type) => {
			let new_ty = *slice_type.elem;
			let mut field = generate_one(new_ty, generics, ident);
			surround_type(&mut field.ty, slice_type.bracket_token);
			field
		}
		Type::Tuple(typle_type) => {
			let TypeTuple { paren_token, elems } = typle_type;
			let mut types: Punctuated<Type, Comma> = Punctuated::new();
			for new_ty in elems.iter() {
				let field = generate_one(new_ty.clone(), generics, ident.clone());
				types.push(field.ty);
			}

			let new_ty = Type::Tuple(TypeTuple {
				paren_token,
				elems: types,
			});

			let field = generate_normal_field(ident, new_ty);
			field
		}
		_ => {
			let field = generate_normal_field(ident, ty);
			field
		}
	}
}

pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
	let mut diagnostics: Vec<Diagnostic> = vec![];
	// parse input
	let input: syn::ItemFn = match syn::parse2(input) {
		Ok(input) => input,
		Err(err) => return err.to_compile_error(),
	};
	let attr: ComponentAttrs = match syn::parse2(attr) {
		Ok(attr) => attr,
		Err(err) => return err.to_compile_error(),
	};

	// check if input is valid
	if let Some(constness) = input.sig.constness {
		diagnostics
			.push(Diagnostic::spanned(constness.span(), Level::Error, "component function must not be const"))
	} else if let Some(asyncness) = input.sig.asyncness {
		diagnostics
			.push(Diagnostic::spanned(asyncness.span(), Level::Error, "component function must not be async").into());
	} else if let Some(unsafety) = input.sig.unsafety {
		diagnostics
			.push(Diagnostic::spanned(unsafety.span(), Level::Error, "component function must not be unsafe").into());
	} else if let Some(ref abi) = input.sig.abi {
		diagnostics
			.push(Diagnostic::spanned(abi.span(), Level::Error, "component function must not have an abi").into());
	}

	let mut generics = input.sig.generics.clone();

	let mut fields: Punctuated<Field, syn::token::Comma> = Punctuated::new();

	for arg in input.sig.inputs.iter() {
		match arg {
			FnArg::Receiver(_) => {
				diagnostics.push(
					Diagnostic::spanned(arg.span(), Level::Error, "component function must not have self argument").into(),
				);
			}
			FnArg::Typed(pat_type) => {
				let pat = *pat_type.pat.clone();
				let ty = *pat_type.ty.clone();
				match pat {
					Pat::Ident(ident_pat) => {
						let field = generate_one(ty, &mut generics, ident_pat.ident);
						fields.push(field);
					}
					Pat::TupleStruct(tuple_struct_pat) => {
						let ident = create_unique_field_ident(
							fields.clone(),
							input.sig.inputs.clone(),
							// path segments should always have at least one element so unwrap is ok
							Some(tuple_struct_pat.path.segments.last().unwrap().ident.clone()),
						);
						let field = generate_one(ty, &mut generics, ident);
						fields.push(field);
					}
					Pat::Struct(struct_pat) => {
						let ident = create_unique_field_ident(
							fields.clone(),
							input.sig.inputs.clone(),
							// path segments should always have at least one element so unwrap is ok
							Some(struct_pat.path.segments.last().unwrap().ident.clone()),
						);
						let field = generate_one(ty, &mut generics, ident);
						fields.push(field);
					}
					Pat::Tuple(_tuple_pat) => {
						let ident = create_unique_field_ident(fields.clone(), input.sig.inputs.clone(), None);
						let field = generate_one(ty, &mut generics, ident);
						fields.push(field);
					}
					Pat::Slice(_slice_pat) => {
						let ident = create_unique_field_ident(fields.clone(), input.sig.inputs.clone(), None);
						let field = generate_one(ty, &mut generics, ident);
						fields.push(field);
					}
					_ => {
						diagnostics
							.push(Diagnostic::spanned(pat.span(), Level::Error, "couldn't parse function argument").into());
					}
				}
			}
		}
	}

	let ident = attr.name;
	let vis = match attr.vis {
		syn::Visibility::Inherited => input.vis.clone(),
		v => v,
	};

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let generated_struct = quote! {
		#[derive(::rstml_component::HtmlComponent)]
		#vis struct #ident #impl_generics #where_clause {#fields}
	};

	let input_ident = input.sig.ident.clone();
	let mut fn_args = Vec::new();
	for field in fields.iter() {
		// unwrap shouldn't panic since each field is generated with an ident
		let ident = field.ident.clone().unwrap();
		fn_args.push(quote!(self.#ident,));
	}

	let impl_block = quote! {
		impl #impl_generics ::rstml_component::HtmlContent for #ident #ty_generics #where_clause {
			fn fmt(self, formatter: &mut ::rstml_component::HtmlFormatter) -> std::fmt::Result {
				formatter.write_content(#input_ident (#(#fn_args)*))
			}
		}
	};

	let diagnostics = diagnostics.iter().map(|d| d.clone().emit_as_item_tokens());
	quote! {
		#(#diagnostics)*
		#input
		#generated_struct
		#impl_block
	}
}
