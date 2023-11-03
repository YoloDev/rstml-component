mod component;
mod template;
mod func;
mod write;

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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

/// Turns a function into a component function, arguments to the attribute macro are the visibility
/// for the generated struct and the name of the struct. The function should return `impl HtmlContent`.
///
/// # Usage
///
/// ```
/// # use rstml_component::{html, component, HtmlContent};
/// #[component(pub MyComponent)]
/// fn my_component(title: impl Into<String>) -> impl HtmlContent {
///   html! {
///     <div>{title.into()}</div>
///   }
/// }
#[proc_macro_attribute]
pub fn component(
	attr: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	func::component(attr.into(), input.into()).into()
}
