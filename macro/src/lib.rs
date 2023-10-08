mod component;
mod template;
mod utils;
mod write;

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	write::html(input.into(), false).into()
}

#[proc_macro]
pub fn move_html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	write::html(input.into(), true).into()
}

#[proc_macro]
pub fn write_html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	write::write_html(input.into()).into()
}

#[proc_macro_derive(HtmlComponent, attributes(html))]
pub fn derive_html_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	component::derive_html_component(input.into()).into()
}

#[proc_macro_attribute]
pub fn component(
	attr: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	utils::component(attr.into(), input.into()).into()
}
