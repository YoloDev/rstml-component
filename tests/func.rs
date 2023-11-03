use rstml_component::HtmlContent;
use rstml_component_macro::{component, html};

#[component(pub MyComponent)]
fn my_component(title: String) -> impl HtmlContent {
	html! {
		<div>{title}</div>
	}
}

struct Foo(String);

#[component(MyGroupComponent)]
fn my_group_component(Foo(title): Foo) -> impl HtmlContent {
	html! {
		<div>{title}</div>
	}
}

#[test]
fn test_func() {
	let component = MyComponent {
		title: "Hello there".to_string(),
	};
	let out = component
		.into_string()
		.expect("formatting works and produces valid utf-8");

	let group_out = my_group_component(Foo("Hello there".to_string()))
		.into_string()
		.expect("formatting works and produces valid utf-8");

	let expected = r#"<div>Hello there</div>"#;

	assert_eq!(out, expected);
	assert_eq!(group_out, expected);
}

#[component(MyGenericComponent)]
fn my_generic_component(title: impl Into<String>) -> impl HtmlContent {
	html! {
		<div>{title.into()}</div>
	}
}

#[component(WhereGenericComp)]
fn where_generic_comp<T>(title: T) -> impl HtmlContent
where
	T: Into<String>,
{
	html! {
		<div>{title.into()}</div>
	}
}

#[component(InlineGenericComp)]
fn inline_generic_comp<T: Into<String>>(title: T) -> impl HtmlContent {
	html! {
		<div>{title.into()}</div>
	}
}

#[component(MixedGenericComp)]
fn mixed_generic_comp<T, C: HtmlContent>(
	title: T,
	description: impl Into<String>,
	children: C,
) -> impl HtmlContent
where
	T: Into<String>,
{
	html! {
		<div>{title.into()}</div><div>{description.into()}</div>{children}
	}
}

#[test]
fn test_generic() {
	let generic_out = my_generic_component("Hello there")
		.into_string()
		.expect("formatting works and produces valid utf-8");

	let where_generic_out = where_generic_comp("Hello there")
		.into_string()
		.expect("formatting works and produces valid utf-8");

	let inline_generic_out = inline_generic_comp("Hello there")
		.into_string()
		.expect("formatting works and produces valid utf-8");

	let expected = r#"<div>Hello there</div>"#;

	assert_eq!(generic_out, expected);
	assert_eq!(where_generic_out, expected);
	assert_eq!(inline_generic_out, expected);

	let mixed_generic_out = mixed_generic_comp(
		"Hello there",
		"Hello there",
		my_generic_component("Hello there"),
	)
	.into_string()
	.expect("formatting works and produces valid utf-8");

	let expected = r#"<div>Hello there</div><div>Hello there</div><div>Hello there</div>"#;

	assert_eq!(mixed_generic_out, expected);
}
