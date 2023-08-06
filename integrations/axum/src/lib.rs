use axum::{headers::ContentType, http::StatusCode, response::IntoResponse, TypedHeader};

pub use rstml_component::{
	html, move_html, write_html, For, HtmlAttributeFormatter, HtmlAttributeValue, HtmlComponent,
	HtmlContent, HtmlFormatter, RawText,
};

#[cfg(feature = "sanitize")]
pub use rstml_component::{SanitizeConfig, Sanitized};

pub struct Html<C>(pub C);

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
