use quote::{quote, quote_spanned, ToTokens};
use rstml::node::NodeName;
use syn::spanned::Spanned;

#[derive(Default)]
pub struct IdeHelper {
	open_tag_names: Vec<NodeName>,
	close_tag_names: Vec<NodeName>,
	attr_names: Vec<NodeName>,
}

impl IdeHelper {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn mark_open_tag(&mut self, name: NodeName) {
		self.open_tag_names.push(name);
	}

	pub fn mark_close_tag(&mut self, name: NodeName) {
		self.close_tag_names.push(name);
	}

	pub fn mark_attr_name(&mut self, name: NodeName) {
		self.attr_names.push(name);
	}
}

impl ToTokens for IdeHelper {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		fn mark_as_element(name: &NodeName, open: bool) -> proc_macro2::TokenStream {
			let element = quote_spanned!(name.span() => enum);
			let mut name_string = name.to_string();
			if !open {
				name_string.push('/');
			}

			quote!({
				#[doc = #name_string]
				#element X{}
			})
		}

		fn mark_as_attribute(name: &NodeName) -> proc_macro2::TokenStream {
			let element = quote_spanned!(name.span() => A);
			quote!({enum X {#element}})
		}

		self
			.open_tag_names
			.iter()
			.for_each(|name| tokens.extend(mark_as_element(name, true)));
		self
			.close_tag_names
			.iter()
			.for_each(|name| tokens.extend(mark_as_element(name, false)));
		self
			.attr_names
			.iter()
			.for_each(|name| tokens.extend(mark_as_attribute(name)));
	}
}
