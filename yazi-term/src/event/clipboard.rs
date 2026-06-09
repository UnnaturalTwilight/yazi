use std::str::SplitWhitespace;

use base64::Engine;
use base64::engine::general_purpose;
use strum::{FromRepr, IntoStaticStr};

use crate::parser::{Osc5522Status, Osc5522Type, StateOsc5522};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClipboardEvent {
	ReadMimetypes(ClipboardPaste),
	ReadData(ClipboardRead),
	ReadError(ClipboardError),
	WriteSuccess,
	WriteError(ClipboardError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardData {
	pub mime: Vec<u8>,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardRead {
	pub mimes: ClipboardMimeList,
	pub data: Vec<ClipboardData>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardPaste {
	pub primary: bool,
	pub name: Vec<u8>,
	pub pw: Vec<u8>,
	pub data: ClipboardMimeList,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardError {
	pub name: String,
}

impl ClipboardEvent {
	pub fn r#type(&self) -> &'static str {
		match self {
			Self::ReadMimetypes(_) => "read_mimetypes",
			Self::ReadData(_) => "read_data",
			Self::ReadError(_) => "read_error",
			Self::WriteSuccess => "write_success",
			Self::WriteError(_) => "write_error",
		}
	}

	pub fn mimes(&self) -> Option<&ClipboardMimeList> {
		match self {
			Self::ReadMimetypes(e) => Some(&e.data),
			Self::ReadData(e) => Some(&e.mimes),
			_ => None,
		}
	}

	pub fn primary(&self) -> Option<bool> {
		match self {
			Self::ReadMimetypes(e) => Some(e.primary),
			_ => None,
		}
	}

	pub fn name(&self) -> Option<String> {
		match self {
			Self::ReadMimetypes(e) => Some(String::from_utf8_lossy(&e.name).into_owned()),
			_ => None,
		}
	}

	pub fn pw(&self) -> Option<String> {
		match self {
			Self::ReadMimetypes(e) => Some(String::from_utf8_lossy(&e.pw).into_owned()),
			_ => None,
		}
	}

	pub fn text(&self) -> Option<String> {
		match self {
			Self::ReadData(e) if let Some(t) = e.data.iter().find(|e| e.mime == b"text/plain") => {
				Some(String::from_utf8_lossy(&t.data).into_owned())
			}
			_ => None,
		}
	}

	pub fn is_paste_offer(&self) -> bool {
		match self {
			Self::ReadMimetypes(_) => true,
			_ => false,
		}
	}

	pub fn is_read(&self) -> bool {
		match self {
			Self::ReadError(_) | Self::ReadData(_) => true,
			_ => false,
		}
	}

	pub(crate) fn from_state(s: StateOsc5522) -> Option<Self> {
		Some(match s.r#type.unwrap_or_default() {
			Osc5522Type::Read if s.status == Some(Osc5522Status::DONE) => {
				let mime = general_purpose::STANDARD.decode(&s.mime.first()?).ok()?;
				if mime == b"." {
					return Some(ClipboardEvent::ReadMimetypes(ClipboardPaste {
						primary: s.primary,
						name: general_purpose::STANDARD.decode(&s.name).ok()?,
						pw: general_purpose::STANDARD.decode(&s.pw).ok()?,
						data: ClipboardMimeList::new(
							general_purpose::STANDARD.decode(&s.payload.first()?).ok()?,
						)?,
					}));
				}
				let mut data = Vec::new();
				let mut mimes = Vec::new();
				for (mime, payload) in s.mime.iter().zip(s.payload.iter()) {
					data.push(ClipboardData {
						mime: general_purpose::STANDARD.decode(mime).ok()?,
						data: general_purpose::STANDARD.decode(payload).ok()?,
					});
					mimes.extend(general_purpose::STANDARD.decode(mime).ok()?);
					mimes.push(b' ');
				}
				ClipboardEvent::ReadData(ClipboardRead { mimes: ClipboardMimeList::new(mimes)?, data })
			}
			Osc5522Type::Read => {
				let name = parse_error()?;
				Self::ReadError(ClipboardError { name })
			}
			Osc5522Type::Write if s.status == Some(Osc5522Status::DONE) => ClipboardEvent::WriteSuccess,
			Osc5522Type::Write => {
				let name = parse_error()?;
				Self::WriteError(ClipboardError { name })
			}
			_ => return None,
		})
	}
}

// --- Operation
#[derive(Clone, Copy, Debug, Eq, FromRepr, IntoStaticStr, PartialEq)]
#[repr(u8)]
pub enum ClipboardType {
	Read = 1,
	Write = 2,
}

// --- MIME list
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardMimeList(String);

impl ClipboardMimeList {
	pub fn new(b: Vec<u8>) -> Option<Self> {
		Some(Self(String::from_utf8(b).ok()?))
	}

	pub fn iter(&self) -> SplitWhitespace<'_> {
		self.0.split_whitespace()
	}
}

// --- Error payload parsing
fn parse_error() -> Option<String> {
	todo!("parse da clipboard errors");
}
