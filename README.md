# rstml-component: HTML Component Library for Rust

`rstml-component` is a Rust library that empowers developers to create dynamic HTML components efficiently. With this library, you can define HTML components using Rust structs and easily generate HTML on-the-fly, making it especially useful for server-side applications that need to produce HTML content.

## Features

- **Declarative Component Definition:** Define HTML components using Rust structs, making your code more organized and maintainable.

- **Effortless HTML Generation:** Generate HTML content dynamically by leveraging the power of Rust's expressive syntax.

- **Designed for Server-Side Applications:** Perfectly suited for server applications that need to generate HTML content on-the-fly.

- **Template Reusability:** Create reusable templates by structuring components, enhancing code reusability across your project.

## Installation

To use `rstml-component` in your Rust project, simply add it as a dependency in your `Cargo.toml`:

<!-- x-release-please-start-version -->

```toml
[dependencies]
rstml-component = "0.1.1"
```

<!-- x-release-please-end-version -->

## Usage

Here's a quick example to demonstrate how easy it is to define and use HTML components using `rstml-component`:

```rust
use rstml_component::{HtmlComponent, HtmlContent, HtmlFormatter};

#[derive(HtmlComponent)]
struct Page<T>
where
		T: Into<String>,
{
	title: T,
	heading: String,
}

impl<T> HtmlContent for Page<T>
where
		T: Into<String>,
{
	fn fmt(self, formatter: &mut HtmlFormatter) -> std::fmt::Result {
		write_html!(formatter,
			<Template title=self.title attribute="world">
				<h1>{self.heading}</h1>
				<p>"This is a test"</p>
			</Template>
		)
	}
}

// Example Axum handler - requires rstml-component-axum
async fn index() -> impl IntoResponse {
	Page {
		title: "My Title",
		heading: "Page Heading",
	}
	.into_html()
}
```

For more detailed information and examples, please refer to our [Documentation](https://docs.rs/rstml-component).

<!-- ## Contributing

We welcome contributions from the community! If you have suggestions, bug reports, or would like to contribute code, please follow our [Contribution Guidelines](CONTRIBUTING.md). -->

## License

This project is licensed under the [MIT License](LICENSE).
