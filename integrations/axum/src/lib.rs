// only enables the `doc_cfg` feature when
// the `docsrs` configuration attribute is defined
#![cfg_attr(docsrs, feature(doc_cfg))]

use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::{headers::ContentType, TypedHeader};
use rstml_component::{HtmlContent, HtmlFormatter};

pub struct Html<C>(pub C);

impl<C> Html<C>
where
	C: FnOnce(&mut HtmlFormatter) -> std::fmt::Result,
{
	pub fn from_fn(f: C) -> Self {
		Html(f)
	}
}

impl<C: HtmlContent> From<C> for Html<C> {
	fn from(value: C) -> Self {
		Self(value)
	}
}

impl<C: HtmlContent> HtmlContent for Html<C> {
	fn fmt(self, formatter: &mut HtmlFormatter) -> std::fmt::Result {
		self.0.fmt(formatter)
	}
}

impl<C: HtmlContent> IntoResponse for Html<C> {
	fn into_response(self) -> axum::response::Response {
		match self.0.into_bytes() {
			Ok(bytes) => (TypedHeader(ContentType::html()), bytes).into_response(),
			Err(_e) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
		}
	}
}

pub trait HtmlContentAxiosExt: Sized {
	fn into_html(self) -> Html<Self>;

	fn into_response(self) -> axum::response::Response;
}

impl<C: HtmlContent> HtmlContentAxiosExt for C {
	fn into_html(self) -> Html<Self> {
		Html(self)
	}

	fn into_response(self) -> axum::response::Response {
		IntoResponse::into_response(self.into_html())
	}
}
