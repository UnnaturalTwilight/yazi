use std::fmt::{self, Display};

use base64::{Engine, engine::general_purpose};
use yazi_shim::BASE64_SANE;

use super::traits::Mimelist;

/// Set clipboard content via OSC 52
pub struct SetClipboard {
	content: String,
}

impl SetClipboard {
	pub fn new(content: impl AsRef<[u8]>) -> Self {
		Self { content: general_purpose::STANDARD.encode(content) }
	}
}

impl Display for SetClipboard {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]52;c;{}\x1b\\", self.content)
	}
}

/// Enable receiving unsolicited paste events via OSC 5522: `CSI ? 5522 h`
pub struct EnablePasteEvents;

impl Display for EnablePasteEvents {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b[?5522h")
	}
}

/// Disable receiving unsolicited paste events via OSC 5522: `CSI ? 5522 l`
pub struct DisablePasteEvents;

impl Display for DisablePasteEvents {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b[?5522l")
	}
}

/// Read data from clipboard: `OSC 5522 ; type=read ; <base64 MIME list> ST`
pub struct ReadClipboard<M>(pub M);

impl<M: Mimelist> Display for ReadClipboard<M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mime_list = ListClipboardMimes(self.0.clone());
		let b64 = BASE64_SANE.encode(mime_list.to_string()).into_bytes();
		let s = unsafe { String::from_utf8_unchecked(b64) };
		write!(f, "\x1b]5522;type=read;{}\x1b\\", s)
	}
}

/// Read available MIME types from clipboard: `OSC 5522 ; type=read ; <base64 [.]> ST`
pub struct ReadClipboardMimes;

impl Display for ReadClipboardMimes {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]5522;type=read;{}\x1b\\", BASE64_SANE.encode(b"."))
	}
}

/// Write data to clipboard:
/// `OSC 5522 ; type=write ST`
/// `OSC 5522 ; type=wdata : mime=<base64 MIME type> ; <base64 data chunk> ST`
/// `OSC 5522 ; type=wdata ST`
// TODO: Multiple MIME types
pub struct WriteClipboard<'a> {
	pub mime: &'a [u8],
	pub data: &'a [u8],
}

impl Display for WriteClipboard<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let b64_mime = BASE64_SANE.encode(self.mime).into_bytes();
		let mime_str = unsafe { String::from_utf8_unchecked(b64_mime) };

		write!(f, "\x1b]5522;type=write\x1b\\")?;
		for (_, chunk) in self.data.chunks(4096).enumerate() {
			let b64_chunk = BASE64_SANE.encode(chunk).into_bytes();
			let s = unsafe { String::from_utf8_unchecked(b64_chunk) };
			write!(f, "\x1b]5522;type=wdata:mime={};{s}\x1b\\", mime_str)?;
		}
		write!(f, "\x1b]5522;type=wdata\x1b\\")
	}
}

// TODO: walias packets

/// Write MIME types separated by spaces.
struct ListClipboardMimes<M>(pub M);

impl<M: Mimelist> Display for ListClipboardMimes<M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, m) in self.0.clone().into_iter().enumerate() {
			if i != 0 {
				write!(f, " ")?;
			}
			write!(f, "{m}")?;
		}
		Ok(())
	}
}
