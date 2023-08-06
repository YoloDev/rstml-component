use bytes::{BufMut, Bytes, BytesMut};
use std::fmt;

mod escape;

pub struct HtmlAttributeFormatter<'a> {
	any_written: bool,
	buffer: &'a mut BytesMut,
}

impl<'a> HtmlAttributeFormatter<'a> {
	fn new(buffer: &'a mut BytesMut) -> Self {
		Self {
			any_written: false,
			buffer,
		}
	}

	pub fn write_bytes(&mut self, raw: &[u8]) {
		self.buffer.reserve(raw.len() + 3);
		if !self.any_written {
			self.any_written = true;
			self.buffer.put_slice(b"=\"");
		}

		self.buffer.put_slice(raw);
	}

	pub fn write(&mut self, value: &[u8]) {
		self.write_bytes(&escape::attribute(value))
	}

	pub fn reserve(&mut self, additional: usize) {
		self.buffer.reserve(additional);
	}

	fn write_value(buffer: &mut BytesMut, value: impl HtmlAttributeValue) -> fmt::Result {
		let mut attribute_formatter = HtmlAttributeFormatter::new(buffer);

		value.fmt(&mut attribute_formatter)?;
		if attribute_formatter.any_written {
			buffer.reserve(1);
			buffer.put_slice(b"\"");
		}

		Ok(())
	}
}

pub struct HtmlFormatter<'a> {
	buffer: &'a mut BytesMut,
}

impl<'a> AsMut<HtmlFormatter<'a>> for HtmlFormatter<'a> {
	fn as_mut(&mut self) -> &mut HtmlFormatter<'a> {
		self
	}
}

impl<'a> HtmlFormatter<'a> {
	pub fn new(buffer: &'a mut BytesMut) -> Self {
		Self { buffer }
	}

	pub fn write_bytes(&mut self, raw: &[u8]) {
		self.buffer.extend_from_slice(raw);
	}

	pub fn write(&mut self, value: &[u8]) {
		self.write_bytes(&escape::text(value))
	}

	pub fn write_doctype(&mut self, value: &[u8]) {
		const DOCTYPE_PREFIX: &[u8] = b"<!DOCTYPE ";
		const DOCTYPE_SUFFIX: &[u8] = b">";

		let escaped = escape::text(value);
		self
			.buffer
			.reserve(escaped.len() + DOCTYPE_PREFIX.len() + DOCTYPE_SUFFIX.len());

		self.write_bytes(DOCTYPE_PREFIX);
		self.write_bytes(&escaped);
		self.write_bytes(DOCTYPE_SUFFIX);
	}

	pub fn write_open_tag_start(&mut self, tag: &[u8]) {
		self.buffer.reserve(tag.len() + 1);
		self.write_bytes(b"<");
		self.write_bytes(tag);
	}

	pub fn write_attribute_name(&mut self, name: &[u8]) {
		self.buffer.reserve(name.len() + 1);
		self.write_bytes(b" ");
		self.write_bytes(name);
	}

	pub fn write_attribute_value(&mut self, value: impl HtmlAttributeValue) -> fmt::Result {
		HtmlAttributeFormatter::write_value(self.buffer, value)
	}

	pub fn write_self_close_tag(&mut self) {
		self.write_bytes(b" />");
	}

	pub fn write_open_tag_end(&mut self) {
		self.write_bytes(b">");
	}

	pub fn write_end_tag(&mut self, tag: &[u8]) {
		self.buffer.reserve(tag.len() + 3);
		self.write_bytes(b"</");
		self.write_bytes(tag);
		self.write_bytes(b">");
	}

	pub fn write_content(&mut self, content: impl HtmlContent) -> fmt::Result {
		content.fmt(self)
	}

	pub fn write_comment(&mut self, comment: &[u8]) {
		const COMMENT_PREFIX: &[u8] = b"<!--";
		const COMMENT_SUFFIX: &[u8] = b"-->";

		let escaped = escape::text(comment);
		self
			.buffer
			.reserve(escaped.len() + COMMENT_PREFIX.len() + COMMENT_SUFFIX.len());

		self.write_bytes(COMMENT_PREFIX);
		self.write_bytes(&escaped);
		self.write_bytes(COMMENT_SUFFIX);
	}

	pub fn reserve(&mut self, additional: usize) {
		self.buffer.reserve(additional);
	}
}

pub trait HtmlContent: Sized {
	fn fmt(self, formatter: &mut HtmlFormatter) -> fmt::Result;

	fn write_to(self, buffer: &mut BytesMut) -> fmt::Result {
		let mut formatter = HtmlFormatter::new(buffer);
		self.fmt(&mut formatter)
	}

	fn into_bytes(self) -> Result<Bytes, fmt::Error> {
		let mut buffer = BytesMut::new();

		self.write_to(&mut buffer)?;
		Ok(buffer.freeze())
	}

	fn into_string(self) -> Result<String, fmt::Error> {
		let bytes = self.into_bytes()?;
		String::from_utf8(bytes.to_vec()).map_err(|_| fmt::Error)
	}
}

/// A trait representing a value that can be used as an attribute value in HTML components.
///
/// Types that implement this trait allow customization of how their values are formatted
/// when used as attribute values in HTML tags. This trait is primarily used in conjunction
/// with the [HtmlAttributeFormatter] to control the serialization of attribute values.
///
/// This trait is particularly useful when you need to handle complex attribute values, such
/// as custom data types, enums, or values that require special formatting.
pub trait HtmlAttributeValue {
	/// Formats the value and writes it to the provided [HtmlAttributeFormatter].
	///
	/// This method is used to customize how the implementing type's value is serialized as an
	/// attribute value in HTML. Implementations of this method should write the formatted value
	/// to the provided [HtmlAttributeFormatter] using the appropriate formatting syntax.
	///
	/// # Arguments
	///
	/// - `formatter`: A mutable reference to the [HtmlAttributeFormatter] that handles the output.
	///
	/// # Returns
	///
	/// A [std::fmt::Result] indicating the success or failure of the formatting operation.
	fn fmt(self, formatter: &mut HtmlAttributeFormatter) -> fmt::Result;
}

pub struct RawText<V>(V);

impl<V: AsRef<[u8]>> HtmlContent for RawText<V> {
	fn fmt(self, formatter: &mut HtmlFormatter) -> fmt::Result {
		formatter.write_bytes(self.0.as_ref());
		Ok(())
	}
}

impl<V: AsRef<[u8]>> HtmlAttributeValue for RawText<V> {
	fn fmt(self, formatter: &mut HtmlAttributeFormatter) -> fmt::Result {
		formatter.write_bytes(self.0.as_ref());
		Ok(())
	}
}

impl<F> HtmlContent for F
where
	F: FnOnce(&mut HtmlFormatter) -> fmt::Result,
{
	fn fmt(self, formatter: &mut HtmlFormatter) -> fmt::Result {
		self(formatter)
	}
}

impl HtmlContent for () {
	fn fmt(self, _formatter: &mut HtmlFormatter) -> fmt::Result {
		Ok(())
	}
}

impl HtmlAttributeValue for () {
	fn fmt(self, _formatter: &mut HtmlAttributeFormatter) -> fmt::Result {
		Ok(())
	}
}

impl<T: HtmlContent> HtmlContent for Option<T> {
	fn fmt(self, formatter: &mut HtmlFormatter) -> fmt::Result {
		match self {
			None => Ok(()),
			Some(template) => template.fmt(formatter),
		}
	}
}

impl<T: HtmlAttributeValue> HtmlAttributeValue for Option<T> {
	fn fmt(self, formatter: &mut HtmlAttributeFormatter) -> fmt::Result {
		match self {
			None => Ok(()),
			Some(template) => template.fmt(formatter),
		}
	}
}

fn display(value: fmt::Arguments, mut write: impl FnMut(&[u8])) -> fmt::Result {
	match value.as_str() {
		Some(s) => {
			write(s.as_bytes());
			Ok(())
		}

		None => {
			use fmt::Write;
			struct Writer<F> {
				writer: F,
			}

			impl<F> Write for Writer<F>
			where
				F: FnMut(&[u8]),
			{
				fn write_str(&mut self, s: &str) -> fmt::Result {
					(self.writer)(s.as_bytes());
					Ok(())
				}
			}

			let mut writer = Writer { writer: &mut write };

			write!(&mut writer, "{}", value)
		}
	}
}

macro_rules! impl_simple_write {
	($ty:ty, as_ref) => {
		impl HtmlAttributeValue for $ty {
			fn fmt(self, formatter: &mut HtmlAttributeFormatter) -> fmt::Result {
				formatter.write(self.as_ref());
				Ok(())
			}
		}

		impl HtmlContent for $ty {
			fn fmt(self, formatter: &mut HtmlFormatter) -> fmt::Result {
				formatter.write(self.as_ref());
				Ok(())
			}
		}
	};
	($ty:ty, raw Display) => {
		impl HtmlAttributeValue for $ty {
			fn fmt(self, formatter: &mut HtmlAttributeFormatter) -> fmt::Result {
				display(format_args!("{}", self), |value| {
					formatter.write_bytes(value)
				})
			}
		}

		impl HtmlContent for $ty {
			fn fmt(self, formatter: &mut HtmlFormatter) -> fmt::Result {
				display(format_args!("{}", self), |value| {
					formatter.write_bytes(value)
				})
			}
		}
	};
}

impl_simple_write!(String, as_ref);
impl_simple_write!(&str, as_ref);
impl_simple_write!(Bytes, as_ref);
impl_simple_write!(bool, raw Display);
impl_simple_write!(u8, raw Display);
impl_simple_write!(u16, raw Display);
impl_simple_write!(u32, raw Display);
impl_simple_write!(u64, raw Display);
impl_simple_write!(u128, raw Display);
impl_simple_write!(usize, raw Display);
impl_simple_write!(i8, raw Display);
impl_simple_write!(i16, raw Display);
impl_simple_write!(i32, raw Display);
impl_simple_write!(i64, raw Display);
impl_simple_write!(i128, raw Display);
impl_simple_write!(isize, raw Display);
impl_simple_write!(f32, raw Display);
impl_simple_write!(f64, raw Display);
