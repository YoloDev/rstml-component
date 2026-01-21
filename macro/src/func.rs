use proc_macro2::{Span, TokenStream};
use proc_macro2_diagnostics::{Diagnostic, Level};
use quote::quote;
use syn::{
	parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma, Field, FnArg, GenericParam,
	Generics, Ident, Pat, Path, PathArguments, PathSegment, Token, Type, TypeImplTrait, TypeParam,
	TypePath, Visibility,
};

trait IdentPath {
	fn write_to(&self, target: &mut dyn FnMut(&str));
}

impl IdentPath for &str {
	fn write_to(&self, target: &mut dyn FnMut(&str)) {
		target(self);
	}
}

impl IdentPath for usize {
	fn write_to(&self, target: &mut dyn FnMut(&str)) {
		target(&self.to_string());
	}
}

impl<L, R> IdentPath for (L, R)
where
	L: IdentPath,
	R: IdentPath,
{
	fn write_to(&self, target: &mut dyn FnMut(&str)) {
		self.0.write_to(target);
		self.1.write_to(target);
	}
}

impl IdentPath for &dyn IdentPath {
	fn write_to(&self, target: &mut dyn FnMut(&str)) {
		(*self).write_to(target);
	}
}

#[derive(Clone)]
struct IdentParts<I>(I);

impl<'a, I> IdentPath for IdentParts<I>
where
	I: Iterator<Item = &'a str> + Clone,
{
	fn write_to(&self, target: &mut dyn FnMut(&str)) {
		for part in self.0.clone() {
			target(part);
		}
	}
}

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

	fn resolve_type(&mut self, ty: &Type, ident_hint: &dyn IdentPath) -> Type {
		match ty {
			Type::ImplTrait(inner) => self.add_generic(inner.clone(), ident_hint),

			Type::Array(inner) => {
				let elem = self.resolve_type(&inner.elem, ident_hint);
				Type::Array(syn::TypeArray {
					bracket_token: inner.bracket_token,
					elem: Box::new(elem),
					semi_token: inner.semi_token,
					len: inner.len.clone(),
				})
			}

			Type::Paren(inner) => {
				let elem = self.resolve_type(&inner.elem, ident_hint);
				Type::Paren(syn::TypeParen {
					paren_token: inner.paren_token,
					elem: Box::new(elem),
				})
			}

			Type::Ptr(inner) => {
				let elem = self.resolve_type(&inner.elem, ident_hint);
				Type::Ptr(syn::TypePtr {
					star_token: inner.star_token,
					const_token: inner.const_token,
					mutability: inner.mutability,
					elem: Box::new(elem),
				})
			}

			Type::Reference(inner) => {
				let elem = self.resolve_type(&inner.elem, ident_hint);
				Type::Reference(syn::TypeReference {
					and_token: inner.and_token,
					lifetime: inner.lifetime.clone(),
					mutability: inner.mutability,
					elem: Box::new(elem),
				})
			}

			Type::Slice(inner) => {
				let elem = self.resolve_type(&inner.elem, ident_hint);
				Type::Slice(syn::TypeSlice {
					bracket_token: inner.bracket_token,
					elem: Box::new(elem),
				})
			}

			Type::Tuple(inner) => {
				let elems = inner
					.elems
					.iter()
					.enumerate()
					.map(|(idx, elem)| self.resolve_type(elem, &(ident_hint, idx)))
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
		let ty = self.resolve_type(&ty, &IdentParts(ident.to_string().split('_')));

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

	fn add_generic(&mut self, impl_type: TypeImplTrait, ident_hint: &dyn IdentPath) -> Type {
		let mut type_ident_str = String::new();
		type_ident_str.push('T');
		ident_hint.write_to(&mut |part| {
			if part.is_empty() {
				return;
			}

			let first_char = part.chars().next().unwrap();
			type_ident_str.push(first_char.to_ascii_uppercase());
			type_ident_str.push_str(&part[first_char.len_utf8()..]);
		});

		let existing_set = self
			.generics
			.type_params()
			.map(|par| par.ident.to_string())
			.collect::<std::collections::HashSet<_>>();

		while existing_set.contains(&type_ident_str) {
			type_ident_str.push('_');
		}

		let type_ident = Ident::new(&type_ident_str, Span::call_site());
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
			#[allow(non_snake_case)]
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
		diagnostics.push(Diagnostic::spanned(
			asyncness.span(),
			Level::Error,
			"component function must not be async",
		));
	} else if let Some(unsafety) = input.sig.unsafety {
		diagnostics.push(Diagnostic::spanned(
			unsafety.span(),
			Level::Error,
			"component function must not be unsafe",
		));
	} else if let Some(ref abi) = input.sig.abi {
		diagnostics.push(Diagnostic::spanned(
			abi.span(),
			Level::Error,
			"component function must not have an abi",
		));
	}

	let mut struct_builder = ComponentStructBuilder::new(attr, input.sig.generics.clone());

	for arg in input.sig.inputs.iter() {
		match arg {
			FnArg::Receiver(_) => {
				diagnostics.push(Diagnostic::spanned(
					arg.span(),
					Level::Error,
					"component function must not have self argument",
				));
			}

			FnArg::Typed(pat_type) => {
				let pat = *pat_type.pat.clone();
				let ty = *pat_type.ty.clone();

				match pat {
					Pat::Ident(pat) => struct_builder.push_field(pat.ident, ty),

					Pat::TupleStruct(pat) => {
						diagnostics.push(Diagnostic::spanned(
							pat.span(),
							Level::Error,
							"tuple struct pattern not supported",
						));
					}

					Pat::Struct(pat) => {
						diagnostics.push(Diagnostic::spanned(
							pat.span(),
							Level::Error,
							"struct pattern not supported",
						));
					}

					Pat::Tuple(pat) => {
						diagnostics.push(Diagnostic::spanned(
							pat.span(),
							Level::Error,
							"tuple pattern not supported",
						));
					}

					Pat::Slice(pat) => {
						diagnostics.push(Diagnostic::spanned(
							pat.span(),
							Level::Error,
							"slice pattern not supported",
						));
					}

					_ => {
						diagnostics.push(Diagnostic::spanned(
							pat.span(),
							Level::Error,
							"couldn't parse function argument",
						));
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
