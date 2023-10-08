use rstml_component::HtmlContent;
use rstml_component_macro::{component, write_html};

#[component(MyComponent)]
fn my_component(title: String) -> std::fmt::Result {
	write_html!(formatter,
		<div>{title}</div>
	)
}

// generics must be specified in this format
#[component(MyGenericComponent)]
fn my_generic_component<T>(title: T) -> std::fmt::Result
where
	T: Into<String>,
{
	write_html!(formatter,
		<div>{title.into()}</div>
	)
}

#[test]
fn test_utils() {
	let component = MyComponent {
		title: "Hello there".into(),
	};
	let out = component
		.into_string()
		.expect("formatting works and produces valid utf-8");

	let generic_component = MyGenericComponent {
		title: "Hello there",
	};
	let generic_out = generic_component
		.into_string()
		.expect("formatting works and produces valid utf-8");

	let expected = r#"<div>Hello there</div>"#;

	assert_eq!(out, expected);
	assert_eq!(generic_out, expected);
}
