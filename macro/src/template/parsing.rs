use crate::template::{Component, ComponentProp};

use super::{ide::IdeHelper, AttributeValue, Children, Template, TemplateWriteInstruction};
use proc_macro2::{Ident, Span};
use proc_macro2_diagnostics::{Diagnostic, SpanDiagnosticExt};
use rstml::{
	node::{
		AttributeValueExpr, FnBinding, KeyedAttribute, KeyedAttributeValue, Node, NodeAttribute,
		NodeBlock, NodeComment, NodeDoctype, NodeElement, NodeFragment, NodeName, NodeText, RawText,
	},
	ParsingResult,
};
use std::collections::HashSet;
use syn::{spanned::Spanned, Expr, ExprLit, Lit, LitBool, Path};

enum TagType {
	Component(Path),
	Element,
	Block,
}

pub struct TemplateParser {
	empty_elements: &'static HashSet<&'static str>,
	instructions: Vec<TemplateWriteInstruction>,
	diagnostics: Vec<Diagnostic>,
	ide_helper: IdeHelper,
}

impl TemplateParser {
	pub(super) fn new(empty_elements: &'static HashSet<&'static str>) -> Self {
		Self {
			empty_elements,
			instructions: Vec::new(),
			diagnostics: Vec::new(),
			ide_helper: IdeHelper::new(),
		}
	}

	pub fn parse_syn_stream(self, stream: syn::parse::ParseStream) -> Template {
		let config = rstml::ParserConfig::new()
			.recover_block(true)
			.always_self_closed_elements(self.empty_elements.clone())
			.raw_text_elements(["script", "style"].into_iter().collect());

		let parser = rstml::Parser::new(config);
		let parsing_result = parser.parse_syn_stream(stream);

		self.parse(parsing_result)
	}

	pub fn parse(mut self, parsing_result: ParsingResult<Vec<Node>>) -> Template {
		let (nodes, diagnostics) = parsing_result.split();
		self.diagnostics = diagnostics;

		self.parse_nodes(nodes)
	}

	pub fn parse_nodes(mut self, nodes: Option<Vec<Node>>) -> Template {
		if let Some(nodes) = nodes {
			if !nodes.is_empty() {
				self.visit_nodes(nodes);
			}
		}

		Template {
			instructions: self.instructions,
			diagnostics: self.diagnostics,
			ide_helper: self.ide_helper,
		}
	}

	fn visit_nodes(&mut self, nodes: impl IntoIterator<Item = Node>) {
		for node in nodes {
			self.visit_node(node);
		}
	}

	fn visit_node(&mut self, node: Node) {
		match node {
			Node::Doctype(doctype) => self.visit_doctype(doctype),
			Node::Element(element) => self.visit_element(element),
			Node::Text(text) => self.visit_text(text),
			Node::RawText(raw_text) => self.visit_raw_text(raw_text),
			Node::Fragment(fragment) => self.visit_fragment(fragment),
			Node::Comment(comment) => self.visit_comment(comment),
			Node::Block(block) => self.visit_block(block),
		}
	}

	fn visit_doctype(&mut self, doctype: NodeDoctype) {
		self
			.instructions
			.push(TemplateWriteInstruction::Doctype(doctype.value));
	}

	fn visit_element(&mut self, element: NodeElement) {
		fn tag_type(name: &NodeName) -> TagType {
			match name {
				NodeName::Block(_) => TagType::Block,
				NodeName::Punctuated(_) => TagType::Element,
				NodeName::Path(path) => {
					if path.qself.is_some()
						|| path.path.leading_colon.is_some()
						|| path.path.segments.len() > 1
						|| path.path.segments[0]
							.ident
							.to_string()
							.starts_with(char::is_lowercase)
					{
						TagType::Element
					} else {
						TagType::Component(path.path.clone())
					}
				}
			}
		}

		match tag_type(element.name()) {
			TagType::Component(path) => self.visit_component(element, path),
			TagType::Element => self.visit_html_element(element),
			TagType::Block => self.visit_block_element(element),
		}
	}

	fn visit_component_children(&mut self, children: Vec<Node>) -> Option<Children> {
		if children.len() == 1 && matches!(children[0], Node::Block(_)) {
			let block = match children.into_iter().next().unwrap() {
				Node::Block(block) => block,
				_ => unreachable!(),
			};

			Some(Children::Expr(block))
		} else {
			let template = TemplateParser::new(self.empty_elements).parse_nodes(Some(children));

			if template.is_empty() {
				None
			} else {
				Some(Children::Template(template))
			}
		}
	}

	fn visit_component(&mut self, element: NodeElement, path: Path) {
		// TODO: improve
		fn is_valid_identifier(value: &str) -> bool {
			// let chars = value.as_bytes().iter().copied();
			!value.is_empty()
				&& value.is_ascii()
				&& value.chars().next().unwrap().is_ascii_alphabetic()
				&& value[1..]
					.chars()
					.all(|c| c.is_ascii_alphanumeric() || c == '_')
		}

		let children = if !element.children.is_empty() {
			self.visit_component_children(element.children)
		} else {
			None
		};

		let NodeElement { open_tag, .. } = element;

		let props = open_tag
			.attributes
			.into_iter()
			.filter_map(|attr| {
				let NodeAttribute::Attribute(KeyedAttribute  {
					key,
					possible_value,
				}) = attr else {
					self.diagnostics.push(attr.span().error("Only keyed attributes are supported"));
					return None;
				};

				let name = match key {
					NodeName::Path(path) => {
						if path.qself.is_some()
							|| path.path.leading_colon.is_some()
							|| path.path.segments.len() != 1
						{
							self.diagnostics.push(
								path
									.span()
									.error("Only simple identifiers are supported as component prop-names"),
							);
							return None;
						}

						path.path.segments[0].ident.clone()
					}
					NodeName::Punctuated(name) => {
						let mut s = NodeName::Punctuated(name.clone()).to_string();
						s.retain(|c| !c.is_ascii_whitespace());
						let s = s.replace('-', "_");

						if is_valid_identifier(&s) {
							Ident::new(&s, name.span())
						} else {
							self
								.diagnostics
								.push(name.span().error(format!("Invalid prop-name `{}`", s)));
							return None;
						}
					}
					NodeName::Block(block) => {
						self
							.diagnostics
							.push(block.span().error("Dynamic attributes are not supported"));
						return None;
					}
				};

				let value = match possible_value {
					KeyedAttributeValue::Binding(binding) => {
						self.diagnostics.push(
							binding
								.span()
								.error("Component prop-functions are not supported."),
						);
						return None;
					}
					KeyedAttributeValue::Value(value) => value.value,
					KeyedAttributeValue::None => Expr::Lit(ExprLit {
						attrs: vec![],
						lit: Lit::Bool(LitBool {
							value: true,
							span: Span::call_site(),
						}),
					}),
				};

				if &name == "children" {
					self.diagnostics.push(
						name
							.span()
							.error("The `children` prop is reserved for components."),
					);

					return None;
				}

				Some(ComponentProp { name, value })
			})
			.collect::<Vec<_>>();

		self
			.instructions
			.push(TemplateWriteInstruction::Component(Component {
				path,
				props,
				children,
			}));
	}

	fn visit_block_element(&mut self, element: NodeElement) {
		self.diagnostics.push(
			element
				.name()
				.span()
				.error("Dynamic elements are not supported. Use a component or an HTML element instead."),
		);
	}

	fn visit_html_element(&mut self, element: NodeElement) {
		let element_span = element.span();
		let NodeElement {
			open_tag,
			close_tag,
			children,
		} = element;

		self.ide_helper.mark_open_tag(open_tag.name.clone());
		if let Some(close_tag) = close_tag {
			self.ide_helper.mark_close_tag(close_tag.name);
		}

		let name = open_tag.name;
		self
			.instructions
			.push(TemplateWriteInstruction::OpenTagStart(name.clone()));

		// attributes
		self.visit_html_attributes(&name, open_tag.attributes);

		if self.empty_elements.contains(&*name.to_string()) {
			// special empty tags that can't have children (for instance <br>)
			self
				.instructions
				.push(TemplateWriteInstruction::SelfCloseTag);

			if !children.is_empty() {
				self
					.diagnostics
					.push(element_span.error("Empty elements cannot have children."));
			}
		} else {
			// normal tags
			self.instructions.push(TemplateWriteInstruction::OpenTagEnd);

			// children
			self.visit_nodes(children);

			// end tag
			self
				.instructions
				.push(TemplateWriteInstruction::EndTag(name.clone()));
		}
	}

	fn visit_html_attributes(
		&mut self,
		element_name: &NodeName,
		attributes: impl IntoIterator<Item = NodeAttribute>,
	) {
		for attribute in attributes {
			// TODO: Special handling of class? and duplicates?
			self.visit_html_attribute(element_name, attribute);
		}
	}

	fn visit_html_attribute(&mut self, element_name: &NodeName, attribute: NodeAttribute) {
		match attribute {
			NodeAttribute::Block(block) => self.visit_html_block_attribute(element_name, block),
			NodeAttribute::Attribute(attribute) => {
				self.visit_html_static_attribute(element_name, attribute)
			}
		}
	}

	fn visit_html_block_attribute(&mut self, _element_name: &NodeName, block: NodeBlock) {
		self
			.instructions
			.push(TemplateWriteInstruction::DynamicAttributes(block));
	}

	fn visit_html_static_attribute(&mut self, element_name: &NodeName, attribute: KeyedAttribute) {
		let KeyedAttribute {
			key,
			possible_value,
		} = attribute;
		self.ide_helper.mark_attr_name(key.clone());

		self
			.instructions
			.push(TemplateWriteInstruction::AttributeName(key.clone()));

		match possible_value {
			KeyedAttributeValue::Binding(binding) => {
				self.visit_attribute_binding(element_name, &key, binding)
			}
			KeyedAttributeValue::Value(value) => self.visit_attribute_value(element_name, &key, value),
			KeyedAttributeValue::None => (),
		}
	}

	fn visit_attribute_binding(
		&mut self,
		_element_name: &NodeName,
		_attribute_name: &NodeName,
		binding: FnBinding,
	) {
		self.diagnostics.push(
			binding
				.span()
				.error("Attribute bindings are not supported."),
		);
	}

	fn visit_attribute_value(
		&mut self,
		_element_name: &NodeName,
		_attribute_name: &NodeName,
		value: AttributeValueExpr,
	) {
		if let Some(value) = value.value_literal_string() {
			self
				.instructions
				.push(TemplateWriteInstruction::AttributeValue(
					AttributeValue::Constant(value),
				));
		} else {
			self
				.instructions
				.push(TemplateWriteInstruction::AttributeValue(
					AttributeValue::Expression(value.value.clone()),
				));
		}
	}

	fn visit_text(&mut self, text: NodeText) {
		self.instructions.push(TemplateWriteInstruction::Text(text));
	}

	fn visit_raw_text(&mut self, raw_text: RawText) {
		self
			.instructions
			.push(TemplateWriteInstruction::RawText(raw_text));
	}

	fn visit_fragment(&mut self, fragment: NodeFragment) {
		self.visit_nodes(fragment.children);
	}

	fn visit_comment(&mut self, comment: NodeComment) {
		self
			.instructions
			.push(TemplateWriteInstruction::Comment(comment));
	}

	fn visit_block(&mut self, block: NodeBlock) {
		self
			.instructions
			.push(TemplateWriteInstruction::DynamicContent(block));
	}
}
