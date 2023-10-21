use rstml_component::HtmlContent;
use rstml_component_macro::{component, html};

#[component(pub MyComponent)]
fn my_component(title: String) -> impl HtmlContent {
	html! {
		<div>{title}</div>
	}
}

#[component(MyGenericComponent)]
fn my_generic_component(title: impl Into<String>) -> impl HtmlContent {
	html! {
		<div>{title.into()}</div>
	}
}

#[test]
fn test_utils() {
	let component = my_component("Hello there".to_string());
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
