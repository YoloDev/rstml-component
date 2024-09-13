use rstml_component::HtmlContent;
use rstml_component_macro::{component, html};

#[test]
fn simple() {
	#[component(pub MyComponent)]
	fn my_component(title: String) -> impl HtmlContent {
		html! {
			<div>{title}</div>
		}
	}

	let component = MyComponent {
		title: "Hello there".to_string(),
	};
	let html = component
		.into_string()
		.expect("formatting works and produces valid utf-8");

	let expected = r#"<div>Hello there</div>"#;

	assert_eq!(html, expected);
}

#[test]
fn with_impl_param() {
	#[component(MyGenericComponent)]
	fn my_generic_component(
		title: impl Into<String>,
		descriptions: (impl Into<String>, impl Into<String>),
	) -> impl HtmlContent {
		html! {
			<div>{title.into()}</div>
			<div>{descriptions.0.into()}</div>
			<div>{descriptions.1.into()}</div>
		}
	}

	let html = MyGenericComponent {
		title: "Hello there",
		descriptions: ("desc1", "desc2"),
	}
	.into_string()
	.expect("formatting works and produces valid utf-8");

	let expected = r#"<div>Hello there</div><div>desc1</div><div>desc2</div>"#;

	assert_eq!(html, expected);
}

#[test]
fn with_where_clause() {
	#[component(WhereGenericComp)]
	#[expect(clippy::multiple_bound_locations)]
	fn where_generic_comp<T: Clone>(title: T) -> impl HtmlContent
	where
		T: Into<String>,
	{
		html! {
			<div>{title.clone().into()}</div>
		}
	}

	let html = WhereGenericComp {
		title: "Hello there",
	}
	.into_string()
	.expect("formatting works and produces valid utf-8");

	let expected = r#"<div>Hello there</div>"#;

	assert_eq!(html, expected);
}

#[test]
fn mixed_args() {
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

	#[component(Inner)]
	fn inner(name: impl Into<String>) -> impl HtmlContent {
		html! {
			<div>"Hello, "{name.into()}</div>
		}
	}

	let html = MixedGenericComp {
		title: "Hello there",
		description: "desc",
		children: Inner { name: "Test" },
	}
	.into_string()
	.expect("formatting works and produces valid utf-8");

	let expected = r#"<div>Hello there</div><div>desc</div><div>Hello, Test</div>"#;

	assert_eq!(html, expected);
}
