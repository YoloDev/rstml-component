# rstml-component-axum: Integration with rstml-component for Axum

`rstml-component-axum` is a crate designed to streamline the usage of `rstml-component` within [Axum](https://github.com/tokio-rs/axum) projects. This crate provides glue code and helpers to make it easier to create dynamic HTML content using `rstml-component` within your Axum applications.

## Features

- **Simplified Integration:** Seamlessly integrate `rstml-component` into your Axum handlers for efficient HTML generation.

- **Optimized for Axum:** Enjoy the benefits of both `rstml-component` and Axum for building high-performance server applications.

## Installation

To use `rstml-component-axum` in your Axum project, add it as a dependency in your `Cargo.toml`:

<!-- x-release-please-start-version -->

```toml
[dependencies]
rstml-component-axum = "0.1"
```

<!-- x-release-please-end-version -->

## Usage

Here's a basic example demonstrating how to use `rstml-component-axum` to integrate `rstml-component` with an Axum handler:

```rust
use axum::{response::IntoResponse, routing::get, Router};
use rstml_component::{move_html, write_html, For, HtmlComponent, HtmlContent};
use rstml_component_axum::HtmlContentAxiosExt;
use std::net::SocketAddr;

#[derive(HtmlComponent)]
struct Book {
	title: &'static str,
	author: &'static str,
}

impl Book {
	fn new(title: &'static str, author: &'static str) -> Self {
		Self { title, author }
	}
}

impl HtmlContent for Book {
	fn fmt(self, formatter: &mut rstml_component::HtmlFormatter) -> std::fmt::Result {
		write_html!(formatter,
			<div>
				<h1>{self.title}</h1>
				<h2>"("{self.author}")"</h2>
			</div>
		)
	}
}

// Your Axum handler
async fn index() -> impl IntoResponse {
	let books = [
		("Moby Dick", "Herman Melville"),
		("Lord of the Rings", "John Ronald Reuel Tolkien"),
	];

	move_html!(
		<div class="books">
			<For items={books}>
				{ |f, book| Book::new(book.0, book.1).fmt(f) }
			</For>
		</div>
	)
	.into_html()
}

#[tokio::main]
async fn main() {
	let app = Router::new().route("/", get(index));

	let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
	println!("listening on {}", addr);

	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.await
		.unwrap();
}
```

For a more detailed walkthrough and additional examples, refer to the [documentation for `rstml-component-axum`](https://docs.rs/rstml-component-axum).

<!-- ## Contributing

We welcome your contributions! If you encounter issues, have suggestions, or would like to contribute code, please follow our [Contribution Guidelines](CONTRIBUTING.md). -->

## License

This project is licensed under the [MIT License](../../LICENSE).
