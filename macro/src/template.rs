use self::ide::IdeHelper;
use proc_macro2::Span;
use proc_macro2_diagnostics::Diagnostic;
use quote::quote;
use quote::ToTokens;
use rstml::node::{NodeBlock, NodeComment, NodeName, NodeText, RawText};
use std::collections::HashSet;
use std::sync::OnceLock;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::Block;
use syn::LitByteStr;
use syn::Stmt;
use syn::{Expr, Ident, Path};

mod ide;
mod parsing;

pub use parsing::TemplateParser;

static DEFAULT_EMPTY_ELEMENTS: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn default_empty_elements() -> &'static HashSet<&'static str> {
	DEFAULT_EMPTY_ELEMENTS.get_or_init(|| {
		[
			"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
			"source", "track", "wbr",
		]
		.into_iter()
		.collect()
	})
}

enum AttributeValue {
	Constant(String),
	Expression(Expr),
}

enum Children {
	Expr(NodeBlock),
	Template(Template),
}

struct ComponentProp {
	name: Ident,
	value: Expr,
}

struct Component {
	path: Path,
	props: Vec<ComponentProp>,
	children: Option<Children>,
}

enum TemplateWriteInstruction {
	Doctype(RawText),
	OpenTagStart(NodeName),
	AttributeName(NodeName),
	AttributeValue(AttributeValue),
	OpenTagEnd,
	SelfCloseTag,
	EndTag(NodeName),
	Text(NodeText),
	RawText(RawText),
	Comment(NodeComment),
	DynamicAttributes(NodeBlock),
	DynamicContent(NodeBlock),
	Component(Component),
}

pub struct Template {
	instructions: Vec<TemplateWriteInstruction>,
	diagnostics: Vec<Diagnostic>,
	ide_helper: IdeHelper,
}

impl Template {
	pub fn parser() -> TemplateParser {
		TemplateParser::new(default_empty_elements())
	}

	pub fn is_empty(&self) -> bool {
		self.instructions.is_empty() && self.diagnostics.is_empty()
	}

	pub fn with_formatter<'a>(&'a self, formatter: &'a Ident) -> impl ToTokens + 'a {
		TemplateTokensWriter {
			instructions: &self.instructions,
			diagnostics: &self.diagnostics,
			ide_helper: &self.ide_helper,
			formatter,
		}
	}
}

struct TemplateTokensWriter<'a> {
	instructions: &'a [TemplateWriteInstruction],
	diagnostics: &'a [Diagnostic],
	ide_helper: &'a IdeHelper,
	formatter: &'a Ident,
}

struct TemplateInstructionWriter<'a> {
	instruction: &'a TemplateWriteInstruction,
	formatter: &'a Ident,
}

impl<'a> ToTokens for TemplateTokensWriter<'a> {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let instructions = self.instructions.iter().map(|i| TemplateInstructionWriter {
			instruction: i,
			formatter: self.formatter,
		});
		let diagnostics = self
			.diagnostics
			.iter()
			.map(|d| d.clone().emit_as_item_tokens());
		let ide_helper = self.ide_helper;

		tokens.extend(quote! {
			#ide_helper
			#(#instructions)*
			#(#diagnostics)*
		});
	}
}

impl<'a> ToTokens for TemplateInstructionWriter<'a> {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let formatter = self.formatter;
		match self.instruction {
			TemplateWriteInstruction::Doctype(doctype) => {
				let value = &doctype.to_token_stream_string();
				let value = LitByteStr::new(value.as_bytes(), doctype.span());
				tokens.extend(quote!(#formatter.write_doctype(#value);));
			}

			TemplateWriteInstruction::OpenTagStart(name) => {
				let value = name.to_string();
				let value = LitByteStr::new(value.as_bytes(), Span::call_site());
				tokens.extend(quote!(#formatter.write_open_tag_start(#value);));
			}

			TemplateWriteInstruction::AttributeName(name) => {
				let value = name.to_string();
				let value = LitByteStr::new(value.as_bytes(), Span::call_site());
				tokens.extend(quote!(#formatter.write_attribute_name(#value);));
			}

			TemplateWriteInstruction::AttributeValue(expr) => {
				tokens.extend(quote!(#formatter.write_attribute_value(#expr)?;));
			}

			TemplateWriteInstruction::OpenTagEnd => {
				tokens.extend(quote!(#formatter.write_open_tag_end();));
			}

			TemplateWriteInstruction::SelfCloseTag => {
				tokens.extend(quote!(#formatter.write_self_close_tag();));
			}

			TemplateWriteInstruction::EndTag(name) => {
				let value = name.to_string();
				let value = LitByteStr::new(value.as_bytes(), Span::call_site());
				tokens.extend(quote!(#formatter.write_end_tag(#value);));
			}

			TemplateWriteInstruction::Text(content) => {
				let value = content.value_string();
				let value = LitByteStr::new(value.as_bytes(), content.span());
				tokens.extend(quote!(#formatter.write_bytes(#value);));
			}

			TemplateWriteInstruction::RawText(content) => {
				let value = content.to_string_best();
				let value = LitByteStr::new(value.as_bytes(), content.span());
				tokens.extend(quote!(#formatter.write_bytes(#value);));
			}

			TemplateWriteInstruction::Comment(comment) => {
				let value = &comment.value;
				let value = LitByteStr::new(value.value().as_bytes(), comment.value.span());
				tokens.extend(quote!(#formatter.write_comment(#value);));
			}

			TemplateWriteInstruction::DynamicAttributes(content) => {
				match content {
					NodeBlock::ValidBlock(Block { stmts, .. }) if stmts.len() == 1 => {
						if let Stmt::Expr(expr, None) = &stmts[0] {
							tokens.extend(quote!(#formatter.write_attributes(#expr)?;));
							return;
						}
					}
					_ => (),
				}

				tokens.extend(quote!(#formatter.write_attributes(#content)?;));
			}

			TemplateWriteInstruction::DynamicContent(content) => {
				match content {
					NodeBlock::ValidBlock(Block { stmts, .. }) if stmts.len() == 1 => {
						if let Stmt::Expr(expr, None) = &stmts[0] {
							tokens.extend(quote!(#formatter.write_content(#expr)?;));
							return;
						}
					}
					_ => (),
				}

				tokens.extend(quote!(#formatter.write_content(#content)?;));
			}

			TemplateWriteInstruction::Component(Component {
				path: name,
				props,
				children,
			}) => {
				let mut props = props
					.iter()
					.map(|ComponentProp { name, value }| quote!(#name: #value))
					.collect::<Vec<_>>();

				if let Some(children) = children {
					let children = match children {
						Children::Expr(expr) => quote!(#expr),
						Children::Template(template) => {
							let template = template.with_formatter(formatter);
							quote! {
								|#formatter: &mut ::rstml_component::HtmlFormatter| -> ::std::fmt::Result {
									#template
									Ok(())
								}
							}
						}
					};

					props.push(quote!(children: #children));
				}

				tokens.extend(quote!(#formatter.write_content(#name { #(#props),* })?;));
			}
		}
	}
}

impl ToTokens for AttributeValue {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			AttributeValue::Constant(value) => {
				let value = &**value;
				tokens.extend(quote!(#value));
			}

			AttributeValue::Expression(expr) => {
				tokens.extend(quote!(#expr));
			}
		}
	}
}

impl Parse for Template {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Template::parser().parse_syn_stream(input))
	}
}
