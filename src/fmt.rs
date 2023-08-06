use bytes::{BufMut, Bytes, BytesMut};
use std::fmt;

mod escape;

/// A formatter for serializing HTML attribute values.
pub struct HtmlAttributeFormatter<'a> {
	any_written: bool,
	buffer: &'a mut BytesMut,
}

impl<'a> HtmlAttributeFormatter<'a> {
	/// Creates a new `HtmlAttributeFormatter` instance with the provided buffer.
	///
	/// # Arguments
	///
	/// - `buffer`: A mutable reference to the [BytesMut] buffer where the formatted content will be written.
	///
	/// # Returns
	///
	/// A new `HtmlAttributeFormatter` instance associated with the provided buffer.
	fn new(buffer: &'a mut BytesMut) -> Self {
		Self {
			any_written: false,
			buffer,
		}
	}

	/// Write raw bytes to the attribute formatter. Note that this method does no sanitization or escaping of
	/// the values. If you want the values to be sanitized, use the [write] method insted.
	///
	/// # Arguments
	///
	/// - `raw`: A reference to the raw byte slice that will be written to the buffer.
	///
	/// [write]: Self::write
	pub fn write_bytes(&mut self, raw: &[u8]) {
		self.buffer.reserve(raw.len() + 3);
		if !self.any_written {
			self.any_written = true;
			self.buffer.put_slice(b"=\"");
		}

		self.buffer.put_slice(raw);
	}

	/// Writes escaped bytes to the attribute formatter, ensuring valid HTML attribute characters.
	///
	/// This method accepts a reference to a byte slice containing the content to be written to the
	/// formatter's buffer. The provided `value` is escaped to ensure that it only contains valid HTML
	/// attribute characters, preventing any unintentional issues with attribute values. The escaped
	/// content is then written to the formatter's buffer.
	///
	/// # Arguments
	///
	/// - `value`: A reference to the raw byte slice containing the content to be escaped and written.
	pub fn write(&mut self, value: &[u8]) {
		self.write_bytes(&escape::attribute(value))
	}

	/// Reserves space in the buffer for writing additional bytes without re-allocation.
	///
	/// This method ensures that enough space is reserved in the formatter's buffer to accommodate
	/// the writing of `additional` bytes without needing to reallocate memory. It's useful to call
	/// this method before writing a significant amount of content, as it can help prevent frequent
	/// reallocations and improve performance.
	///
	/// # Arguments
	///
	/// - `additional`: The number of additional bytes to reserve space for in the buffer.
	pub fn reserve(&mut self, additional: usize) {
		// add 3 for the opening and closing constant parts
		self.buffer.reserve(additional + 3);
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

/// A formatter for serializing HTML nodes and content.
///
/// The `HtmlFormatter` struct provides a versatile way to serialize HTML nodes and content,
/// ensuring proper spacing, indentation, and formatting. It's designed to handle various
/// types of HTML content and produce well-structured and readable HTML output.
///
/// NOTE: Currently, no indentation/readibility is supported. The plan is to implement that
/// later.
pub struct HtmlFormatter<'a> {
	buffer: &'a mut BytesMut,
}

impl<'a> AsMut<HtmlFormatter<'a>> for HtmlFormatter<'a> {
	fn as_mut(&mut self) -> &mut HtmlFormatter<'a> {
		self
	}
}

impl<'a> HtmlFormatter<'a> {
	/// Creates a new `HtmlFormatter` instance with the provided buffer.
	///
	/// # Arguments
	///
	/// - `buffer`: A mutable reference to the [BytesMut] buffer where the formatted content will be written.
	///
	/// # Returns
	///
	/// A new `HtmlFormatter` instance associated with the provided buffer.
	pub fn new(buffer: &'a mut BytesMut) -> Self {
		Self { buffer }
	}

	/// Writes raw bytes to the formatter's buffer without escaping.
	///
	/// This method appends the specified raw bytes to the formatter's buffer without performing any
	/// additional escaping or modification. It provides a low-level, raw API for directly writing
	/// content to the buffer, which can be useful for situations where the content is already
	/// properly formatted and safe to include as-is.
	///
	/// # Arguments
	///
	/// - `raw`: A reference to the raw byte slice that will be written to the buffer.
	pub fn write_bytes(&mut self, raw: &[u8]) {
		self.buffer.extend_from_slice(raw);
	}

	/// Writes escaped bytes to the formatter's buffer, ensuring valid HTML characters.
	///
	/// This method accepts a reference to a byte slice containing the content to be written to
	/// the formatter's buffer. The provided `value` is escaped to ensure that it only contains
	/// valid HTML characters, preventing any unintentional issues with the produced HTML content.
	///
	/// # Arguments
	///
	/// - `value`: A reference to the raw byte slice containing the content to be escaped and written.
	pub fn write(&mut self, value: &[u8]) {
		self.write_bytes(&escape::text(value))
	}

	// Writes a DOCTYPE declaration to the formatter's buffer.
	///
	/// This method appends a DOCTYPE declaration to the formatter's buffer. The provided `value` is
	/// escaped to ensure that it only contains valid characters for a DOCTYPE declaration. The resulting
	/// DOCTYPE declaration is properly formatted and follows the standard syntax of "<!DOCTYPE ...>".
	///
	/// # Arguments
	///
	/// - `value`: A reference to the raw byte slice containing the content for the DOCTYPE declaration.
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

	/// Writes the start of an opening HTML tag to the formatter's buffer.
	///
	/// This method appends the start of an opening HTML tag to the formatter's buffer. The provided `tag`
	/// is used as the tag name, and the tag is not closed. is commonly followed by either [write_attribute_name],
	/// [write_self_close_tag], or [write_open_tag_end].
	///
	/// # Arguments
	///
	/// - `tag`: A reference to the raw byte slice containing the tag name for the opening tag.
	///
	/// [write_attribute_name]: Self::write_attribute_name
	/// [write_self_close_tag]: Self::write_self_close_tag
	/// [write_open_tag_end]: Self::write_open_tag_end
	pub fn write_open_tag_start(&mut self, tag: &[u8]) {
		self.buffer.reserve(tag.len() + 1);
		self.write_bytes(b"<");
		self.write_bytes(tag);
	}

	/// Writes an HTML attribute name to the formatter's buffer.
	///
	/// This method appends an HTML attribute name to the formatter's buffer. The provided `name` is
	/// used as the attribute name.
	///
	/// # Arguments
	///
	/// - `name`: A reference to the raw byte slice containing the attribute name.
	pub fn write_attribute_name(&mut self, name: &[u8]) {
		self.buffer.reserve(name.len() + 1);
		self.write_bytes(b" ");
		self.write_bytes(name);
	}

	/// Writes an HTML attribute value to the formatter's buffer.
	///
	/// This method appends an HTML attribute value to the formatter's buffer. The provided `value` is
	/// an instance of a type implementing the [HtmlAttributeValue] trait. The value is written to the
	/// buffer, ensuring proper formatting and escaping if required.
	///
	/// # Arguments
	///
	/// - `value`: An instance implementing the [HtmlAttributeValue] trait, representing the attribute value.
	///
	/// # Returns
	///
	/// A [std::fmt::Result] indicating the success or failure of the writing operation.
	pub fn write_attribute_value(&mut self, value: impl HtmlAttributeValue) -> fmt::Result {
		HtmlAttributeFormatter::write_value(self.buffer, value)
	}

	/// Writes a self-closing indicator to the formatter's buffer.
	///
	/// This method appends a self-closing indicator " />" to the formatter's buffer. It's commonly used
	/// after writing an opening tag to indicate that the tag is self-closing and has no associated content.
	pub fn write_self_close_tag(&mut self) {
		self.write_bytes(b" />");
	}

	/// Writes the end of an opening HTML tag to the formatter's buffer.
	///
	/// This method appends the end of an opening HTML tag ">" to the formatter's buffer. It's commonly
	/// used after writing the tag name and its attributes to indicate the completion of the tag's opening.
	pub fn write_open_tag_end(&mut self) {
		self.write_bytes(b">");
	}

	/// Writes an HTML end tag to the formatter's buffer.
	///
	/// This method appends an HTML end tag "</tag>" to the formatter's buffer. The provided `tag` is used
	/// as the tag name for the end tag.
	///
	/// # Arguments
	///
	/// - `tag`: A reference to the raw byte slice containing the tag name for the end tag.
	pub fn write_end_tag(&mut self, tag: &[u8]) {
		self.buffer.reserve(tag.len() + 3);
		self.write_bytes(b"</");
		self.write_bytes(tag);
		self.write_bytes(b">");
	}

	/// Writes HTML content to the formatter's buffer.
	///
	/// This method appends HTML content to the formatter's buffer. The provided `content` is an
	/// instance of a type implementing the [HtmlContent] trait. The content is formatted and written
	/// to the buffer according to the implementation of the [HtmlContent] trait.
	///
	/// # Arguments
	///
	/// - `content`: An instance implementing the [HtmlContent] trait, representing the HTML content to write.
	///
	/// # Returns
	///
	/// A [std::fmt::Result] indicating the success or failure of the writing operation.
	pub fn write_content(&mut self, content: impl HtmlContent) -> fmt::Result {
		content.fmt(self)
	}

	/// Writes an HTML comment to the formatter's buffer.
	///
	/// This method appends an HTML comment to the formatter's buffer. The provided `comment` is escaped
	/// to ensure that it only contains valid characters for an HTML comment.
	///
	/// # Arguments
	///
	/// - `comment`: A reference to the raw byte slice containing the content for the HTML comment.
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

	/// Reserves space in the buffer for writing additional bytes without reallocation.
	///
	/// This method ensures that enough space is reserved in the formatter's buffer to accommodate
	/// the writing of `additional` bytes without needing to reallocate memory. It's useful to call
	/// this method before writing a significant amount of content, as it can help prevent frequent
	/// reallocations and improve performance.
	///
	/// # Arguments
	///
	/// - `additional`: The number of additional bytes to reserve space for in the buffer.
	pub fn reserve(&mut self, additional: usize) {
		self.buffer.reserve(additional);
	}
}

/// A trait representing content that can be formatted into HTML representation.
///
/// Types that implement this trait define how they should be formatted as HTML content.
/// This trait provides methods to write the formatted content to various output formats,
/// such as a byte buffer or a string.
pub trait HtmlContent: Sized {
	/// Formats the content and writes it to the provided [HtmlFormatter].
	///
	/// This method defines how the implementing type's content should be formatted as HTML.
	/// Implementations of this method should use the provided [HtmlFormatter] to write the
	/// formatted content according to the desired syntax and structure.
	///
	/// # Arguments
	///
	/// - `formatter`: A mutable reference to the [HtmlFormatter] that handles the output.
	///
	/// # Returns
	///
	/// A [std::fmt::Result] indicating the success or failure of the formatting operation.
	fn fmt(self, formatter: &mut HtmlFormatter) -> fmt::Result;

	/// Writes the formatted content to the provided byte buffer.
	///
	/// This method creates an [HtmlFormatter] that writes to the given `buffer` and uses
	/// the `fmt` method to write the formatted content into the buffer.
	///
	/// # Arguments
	///
	/// - `buffer`: A mutable reference to the byte buffer where the formatted content will be written.
	///
	/// # Returns
	///
	/// A [std::fmt::Result] indicating the success or failure of the formatting operation.
	fn write_to(self, buffer: &mut BytesMut) -> fmt::Result {
		let mut formatter = HtmlFormatter::new(buffer);
		self.fmt(&mut formatter)
	}

	/// Converts the formatted content into a [Bytes] buffer.
	///
	/// This method writes the formatted content to a byte buffer and returns it as a [Bytes] object.
	///
	/// # Returns
	///
	/// A [Result] containing the [Bytes] object if successful, or a [std::fmt::Error] if formatting fails.
	fn into_bytes(self) -> Result<Bytes, fmt::Error> {
		let mut buffer = BytesMut::new();

		self.write_to(&mut buffer)?;
		Ok(buffer.freeze())
	}

	/// Converts the formatted content into a [String].
	///
	/// This method writes the formatted content to a byte buffer, then attempts to convert it into
	/// a [String].
	///
	/// # Returns
	///
	/// A [Result] containing the [String] if successful, or a [std::fmt::Error] if formatting or
	/// conversion to [String] fails.
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

/// A struct for embedding raw, unsanitized HTML content.
///
/// The `RawText` struct allows you to include raw HTML content without any sanitization or
/// modification. This is useful when you need to merge multiple HTML fragments that are known
/// to be safe or pre-sanitized. The `RawText` content is intended for situations where you have
/// direct control over the content being embedded and ensure its safety.
pub struct RawText<V>(V);

impl<V: AsRef<[u8]>> RawText<V> {
	/// Creates a new `RawText` instance with the given raw HTML content.
	///
	/// # Arguments
	///
	/// - `value`: The raw HTML content as a byte slice.
	///
	/// # Returns
	///
	/// A `RawText` instance wrapping the raw HTML content.
	pub fn new(value: V) -> Self {
		Self(value)
	}
}

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
