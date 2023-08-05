use crate::{HtmlComponent, HtmlContent, HtmlFormatter};
use std::fmt;

pub struct For<I, F>
where
	I: IntoIterator,
	F: FnMut(&mut HtmlFormatter, <I as IntoIterator>::Item) -> fmt::Result,
{
	pub items: I,
	pub children: F,
}

impl<I, F> HtmlComponent for For<I, F>
where
	I: IntoIterator,
	F: FnMut(&mut HtmlFormatter, <I as IntoIterator>::Item) -> fmt::Result,
{
	type Content = Self;

	fn into_content(self) -> Self::Content {
		self
	}
}

impl<I, F> HtmlContent for For<I, F>
where
	I: IntoIterator,
	F: FnMut(&mut HtmlFormatter, <I as IntoIterator>::Item) -> fmt::Result,
{
	fn fmt(self, formatter: &mut HtmlFormatter) -> std::fmt::Result {
		let For {
			items,
			children: mut template,
		} = self;

		for item in items {
			template(formatter, item)?;
		}

		Ok(())
	}
}
