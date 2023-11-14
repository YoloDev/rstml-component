use proc_macro2::{Span, TokenStream};
use proc_macro2_diagnostics::{Diagnostic, Level};
use quote::{format_ident, quote};
use syn::{
	parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma, Field, FnArg, GenericParam,
	Generics, Ident, Pat, Path, PathArguments, PathSegment, Token, Type, TypeImplTrait, TypeParam,
	TypePath, Visibility,
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

fn lowercase_ident(ident: &Ident) -> Ident {
	Ident::new(&ident.to_string().to_lowercase(), ident.span())
}

struct ComponentStructBuilder {
	vis: syn::Visibility,
	ident: syn::Ident,
	generics: Generics,
	fields: Punctuated<Field, Comma>,
}

impl ComponentStructBuilder {
	fn new(attr: ComponentAttrs, generics: Generics) -> Self {
		Self {
			vis: attr.vis,
			ident: attr.name,
			generics,
			fields: Punctuated::new(),
		}
	}

	fn create_unique_field_ident(&self, ident: &Ident) -> Ident {
		format_ident!("{}__{}", lowercase_ident(ident), self.fields.len())
	}

	fn create_unique_generic_ident(&self) -> Ident {
		format_ident!("__T{}", self.generics.params.len())
	}

	fn resolve_type(&mut self, ty: &Type) -> Type {
		match ty {
			Type::ImplTrait(inner) => self.add_generic(inner.clone()),
			Type::Array(inner) => {
				let elem = self.resolve_type(&inner.elem);
				Type::Array(syn::TypeArray {
					bracket_token: inner.bracket_token,
					elem: Box::new(elem),
					semi_token: inner.semi_token,
					len: inner.len.clone(),
				})
			}
			Type::Paren(inner) => {
				let elem = self.resolve_type(&inner.elem);
				Type::Paren(syn::TypeParen {
					paren_token: inner.paren_token,
					elem: Box::new(elem),
				})
			}
			Type::Ptr(inner) => {
				let elem = self.resolve_type(&inner.elem);
				Type::Ptr(syn::TypePtr {
					star_token: inner.star_token,
					const_token: inner.const_token,
					mutability: inner.mutability,
					elem: Box::new(elem),
				})
			}
			Type::Reference(inner) => {
				let elem = self.resolve_type(&inner.elem);
				Type::Reference(syn::TypeReference {
					and_token: inner.and_token,
					lifetime: inner.lifetime.clone(),
					mutability: inner.mutability,
					elem: Box::new(elem),
				})
			}
			Type::Slice(inner) => {
				let elem = self.resolve_type(&inner.elem);
				Type::Slice(syn::TypeSlice {
					bracket_token: inner.bracket_token,
					elem: Box::new(elem),
				})
			}
			Type::Tuple(inner) => {
				let elems = inner
					.elems
					.iter()
					.map(|elem| self.resolve_type(elem))
					.collect();
				Type::Tuple(syn::TypeTuple {
					paren_token: inner.paren_token,
					elems,
				})
			}
			_ => ty.clone(),
		}
	}

	fn push_field(&mut self, ident: Ident, ty: Type) {
		let ty = self.resolve_type(&ty);

		let field = Field {
			attrs: vec![],
			vis: Visibility::Public(Token![pub](Span::call_site())),
			mutability: syn::FieldMutability::None,
			ident: Some(ident),
			ty,
			colon_token: Some(Token![:](Span::call_site())),
		};

		self.fields.push(field);
	}

	fn push_non_unique_field(&mut self, ident: &Ident, ty: Type) {
		let ident = self.create_unique_field_ident(ident);
		self.push_field(ident, ty);
	}

	fn add_generic(&mut self, impl_type: TypeImplTrait) -> Type {
		let type_ident = self.create_unique_generic_ident();
		let type_param = TypeParam {
			attrs: vec![],
			ident: type_ident.clone(),
			colon_token: Some(Token![:](Span::call_site())),
			bounds: impl_type.bounds,
			eq_token: None,
			default: None,
		};

		self.generics.params.push(GenericParam::Type(type_param));

		Type::Path(TypePath {
			qself: None,
			path: Path {
				leading_colon: None,
				segments: Punctuated::from_iter(vec![PathSegment {
					ident: type_ident,
					arguments: PathArguments::None,
				}]),
			},
		})
	}

	fn build(self) -> (TokenStream, Ident, Generics, Punctuated<Field, Comma>) {
		let Self {
			vis,
			ident,
			generics,
			fields,
		} = self;
		let (impl_generics, _, where_clause) = generics.split_for_impl();

		let generated_struct = quote! {
			#[derive(::rstml_component::HtmlComponent)]
			#vis struct #ident #impl_generics #where_clause {#fields}
		};

		(generated_struct, ident, generics, fields)
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
		diagnostics.push(Diagnostic::spanned(
			constness.span(),
			Level::Error,
			"component function must not be const",
		))
	} else if let Some(asyncness) = input.sig.asyncness {
		diagnostics.push(
			Diagnostic::spanned(
				asyncness.span(),
				Level::Error,
				"component function must not be async",
			)
			.into(),
		);
	} else if let Some(unsafety) = input.sig.unsafety {
		diagnostics.push(
			Diagnostic::spanned(
				unsafety.span(),
				Level::Error,
				"component function must not be unsafe",
			)
			.into(),
		);
	} else if let Some(ref abi) = input.sig.abi {
		diagnostics.push(
			Diagnostic::spanned(
				abi.span(),
				Level::Error,
				"component function must not have an abi",
			)
			.into(),
		);
	}

	let mut struct_builder = ComponentStructBuilder::new(attr, input.sig.generics.clone());

	for arg in input.sig.inputs.iter() {
		match arg {
			FnArg::Receiver(_) => {
				diagnostics.push(
					Diagnostic::spanned(
						arg.span(),
						Level::Error,
						"component function must not have self argument",
					)
					.into(),
				);
			}
			FnArg::Typed(pat_type) => {
				let pat = *pat_type.pat.clone();
				let ty = *pat_type.ty.clone();
				match pat {
					Pat::Ident(ident_pat) => struct_builder.push_field(ident_pat.ident, ty),
					Pat::TupleStruct(tuple_struct_pat) => struct_builder
						.push_non_unique_field(&tuple_struct_pat.path.segments.last().unwrap().ident, ty),
					Pat::Struct(struct_pat) => struct_builder
						.push_non_unique_field(&struct_pat.path.segments.last().unwrap().ident, ty),
					Pat::Tuple(_tuple_pat) => {
						struct_builder.push_non_unique_field(&format_ident!("tuple"), ty)
					}
					Pat::Slice(_slice_pat) => {
						struct_builder.push_non_unique_field(&format_ident!("slice"), ty)
					}
					_ => {
						diagnostics.push(
							Diagnostic::spanned(pat.span(), Level::Error, "couldn't parse function argument")
								.into(),
						);
					}
				}
			}
		}
	}

	let (generated_struct, ident, generics, fields) = struct_builder.build();

	let input_ident = input.sig.ident.clone();
	let mut fn_args = Vec::new();
	for field in fields.iter() {
		// unwrap shouldn't panic since each field is generated with an ident
		let ident = field.ident.clone().unwrap();
		fn_args.push(quote!(self.#ident,));
	}

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

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
