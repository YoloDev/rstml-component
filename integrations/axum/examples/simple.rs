use axum::{response::IntoResponse, routing::get, Router};
use rstml_component::{html, write_html, For, HtmlComponent, HtmlContent};
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

	html!(
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
