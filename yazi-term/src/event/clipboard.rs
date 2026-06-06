use std::str::SplitWhitespace;

use base64::Engine;
use strum::{FromRepr, IntoStaticStr};
use yazi_shim::BASE64_SANE;

use crate::parser::{Osc5522Status, Osc5522Type, StateOsc5522};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClipboardEvent {
	ReadStart(ClipboardReadStart),
	ReadData(ClipboardReadData),
	ReadEnd(ClipboardData),
	ReadError(ClipboardError),
	WriteSuccess(ClipboardData),
	WriteError(ClipboardError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardReadStart {
	pub pw:	     Vec<u8>,
	pub name:    Vec<u8>,
	pub primary: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardReadData {
	pub mime: ClipboardMimeList,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardData {
	pub mimes: ClipboardMimeList,
	pub primary: bool,
	pub name: Vec<u8>,
	pub pw: Vec<u8>,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardError {
	pub name: String,
	pub desc: String,
}

impl ClipboardEvent {
	pub fn r#type(&self) -> &'static str {
		match self {
			Self::ReadStart(_) => "read_start",
			Self::ReadData(_) => "read_data",
			Self::ReadEnd(_) => "read_end",
			Self::ReadError(_) => "read_error",
			Self::WriteSuccess(_) => "write_success",
			Self::WriteError(_) => "write_error",
		}
	}

	pub fn mimes(&self) -> Option<&ClipboardMimeList> {
		match self {
			Self::ReadData(e) => Some(&e.mime),
			Self::ReadEnd(e) => Some(&e.mimes),
			_ => None,
		}
	}

	pub fn primary(&self) -> Option<bool> {
		match self {
			Self::ReadEnd(e) => Some(e.primary),
			_ => None,
		}
	}

	pub fn name(&self) -> Option<String> {
		match self {
			Self::ReadEnd(e) => Some(String::from_utf8_lossy(&e.name).into_owned()),
			_ => None,
		}
	}

	pub fn pw(&self) -> Option<String> {
		match self {
			Self::ReadEnd(e) => Some(String::from_utf8_lossy(&e.pw).into_owned()),
			_ => None,
		}
	}

	pub fn text(&self) -> Option<String> {
		match self {
			Self::ReadEnd(e) if e.mimes.0.contains("text/plain") => Some(String::from_utf8_lossy(&e.data).into_owned()),
			_ => None,
		}
	}

	pub fn is_mimelist(&self) -> bool {
		match self {
			Self::ReadEnd(e) => e.mimes == ClipboardMimeList(".".to_string()),
			_ => false,
		}
	}

	pub fn is_read(&self) -> bool {
		matches!(
			self,
			Self::ReadStart(_)
				| Self::ReadData(_)
				| Self::ReadEnd(_)
				| Self::ReadError(_)
		)
	}

	pub(crate) fn from_state(s: StateOsc5522) -> Option<Self> {
		Some(match s.r#type.unwrap_or_default() {
			Osc5522Type::Read if s.status == Some(Osc5522Status::OK) => {todo!("clipboard read start")}
			Osc5522Type::Read if s.status == Some(Osc5522Status::DATA) => {todo!("clipboard read start")}
			Osc5522Type::Read if s.status == Some(Osc5522Status::DONE) => ClipboardEvent::ReadEnd(ClipboardData {
				mimes: ClipboardMimeList::new(BASE64_SANE.decode(&s.mime).ok()?)?,
				primary: s.primary,
				name: BASE64_SANE.decode(&s.name).ok()?,
				pw: BASE64_SANE.decode(&s.pw).ok()?,
				data: BASE64_SANE.decode(&s.payload).ok()?,
			}),
			Osc5522Type::Read => {
				let (name, desc) = parse_error(s.payload)?;
				Self::ReadError(ClipboardError { name, desc })
			}
			Osc5522Type::Write if s.status == Some(Osc5522Status::DONE) => ClipboardEvent::WriteSuccess(ClipboardData {
				mimes: ClipboardMimeList::new(BASE64_SANE.decode(&s.mime).ok()?)?,
				primary: s.primary,
				name: BASE64_SANE.decode(&s.name).ok()?,
				pw: BASE64_SANE.decode(&s.pw).ok()?,
				data: BASE64_SANE.decode(&s.payload).ok()?,
			}),
			Osc5522Type::Write => {
				let (name, desc) = parse_error(s.payload)?;
				Self::WriteError(ClipboardError { name, desc })
			}
			_ => return None,
		})
	}
}

// --- Operation
#[derive(Clone, Copy, Debug, Eq, FromRepr, IntoStaticStr, PartialEq)]
#[repr(u8)]
pub enum ClipboardType {
	Read   = 1,
	Write  = 2,
}

// --- MIME list
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardMimeList(String);

impl ClipboardMimeList {
	pub fn new(b: Vec<u8>) -> Option<Self> { Some(Self(String::from_utf8(b).ok()?)) }

	pub fn iter(&self) -> SplitWhitespace<'_> { self.0.split_whitespace() }
}

// --- Error payload parsing
fn parse_error(payload: Vec<u8>) -> Option<(String, String)> {
	todo!("parse da clipboard errors");
}
