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

/// Makes a function that returns `std::fmt::Result` into a component function, argument to the
/// attribute is the name of the struct that will be generated.
///
/// # Usage
///
/// ```
/// #[component(MyComponent)]
/// pub fn my_component(title: String) -> std::fmt::Result {
///     component_html!(
///         <div>{title}</div>
///     )
/// }
/// ```
#[proc_macro_attribute]
pub fn component(
	attr: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	utils::component(attr.into(), input.into()).into()
}

/// Use inside component functions, expands to `write_html!(formatter, #macro_input)`
#[proc_macro]
pub fn component_html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	utils::component_html(input.into()).into()
}
