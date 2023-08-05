use std::borrow::Cow;

/// Escapes an `&str` and replaces all xml special characters (`<`, `>`, `&`, `'`, `"`)
/// with their corresponding xml escaped value.
///
/// This function performs following replacements:
///
/// | Character | Replacement
/// |-----------|------------
/// | `<`       | `&lt;`
/// | `>`       | `&gt;`
/// | `&`       | `&amp;`
/// | `'`       | `&apos;`
/// | `"`       | `&quot;`
pub fn attribute(raw: &[u8]) -> Cow<[u8]> {
	_escape(raw, |ch| matches!(ch, b'<' | b'>' | b'&' | b'\'' | b'\"'))
}

/// Escapes an `&str` and replaces xml special characters (`<`, `>`, `&`)
/// with their corresponding xml escaped value.
///
/// Should only be used for escaping text content. In XML text content, it is allowed
/// (though not recommended) to leave the quote special characters `"` and `'` unescaped.
///
/// This function performs following replacements:
///
/// | Character | Replacement
/// |-----------|------------
/// | `<`       | `&lt;`
/// | `>`       | `&gt;`
/// | `&`       | `&amp;`
pub fn text(raw: &[u8]) -> Cow<[u8]> {
	_escape(raw, |ch| matches!(ch, b'<' | b'>' | b'&'))
}

/// Escapes an `&str` and replaces a subset of xml special characters (`<`, `>`,
/// `&`, `'`, `"`) with their corresponding xml escaped value.
pub(crate) fn _escape<F: Fn(u8) -> bool>(bytes: &[u8], escape_chars: F) -> Cow<[u8]> {
	let mut escaped = None;
	let mut iter = bytes.iter();
	let mut pos = 0;

	while let Some(i) = iter.position(|&b| escape_chars(b)) {
		if escaped.is_none() {
			escaped = Some(Vec::with_capacity(bytes.len() + 20));
		}
		let escaped = escaped.as_mut().expect("initialized");
		let new_pos = pos + i;
		escaped.extend_from_slice(&bytes[pos..new_pos]);
		match bytes[new_pos] {
			b'<' => escaped.extend_from_slice(b"&lt;"),
			b'>' => escaped.extend_from_slice(b"&gt;"),
			b'\'' => escaped.extend_from_slice(b"&apos;"),
			b'&' => escaped.extend_from_slice(b"&amp;"),
			b'"' => escaped.extend_from_slice(b"&quot;"),

			// This set of escapes handles characters that should be escaped
			// in elements of xs:lists, because those characters works as
			// delimiters of list elements
			b'\t' => escaped.extend_from_slice(b"&#9;"),
			b'\n' => escaped.extend_from_slice(b"&#10;"),
			b'\r' => escaped.extend_from_slice(b"&#13;"),
			b' ' => escaped.extend_from_slice(b"&#32;"),
			_ => unreachable!("Only '<', '>','\', '&', '\"', '\\t', '\\r', '\\n', and ' ' are escaped"),
		}
		pos = new_pos + 1;
	}

	if let Some(mut escaped) = escaped {
		if let Some(raw) = bytes.get(pos..) {
			escaped.extend_from_slice(raw);
		}
		// SAFETY: we operate on UTF-8 input and search for an one byte chars only,
		// so all slices that was put to the `escaped` is a valid UTF-8 encoded strings
		// TODO: Can be replaced with `unsafe { String::from_utf8_unchecked() }`
		// if unsafe code will be allowed
		Cow::Owned(escaped)
	} else {
		Cow::Borrowed(bytes)
	}
}
