use crate::HtmlContent;

pub trait HtmlComponent {
	type Content: HtmlContent;

	fn into_content(self) -> Self::Content;
}
