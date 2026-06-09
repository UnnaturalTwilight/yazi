use anyhow::{Result, bail};
use yazi_macro::{act, succ};
use yazi_parser::mgr::CopyForm;
use yazi_shared::{data::Data, strand::ToStrand, url::UrlLike};
use yazi_widgets::{CLIPBOARD, ClipboardData};

use crate::{Actor, Ctx};

pub struct Copy;

impl Actor for Copy {
	type Form = CopyForm;

	const NAME: &str = "copy";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let mut s = Vec::<u8>::new();
		let mut it = if form.hovered {
			Box::new(cx.hovered().map(|h| &h.url).into_iter())
		} else {
			cx.tab().selected_or_hovered()
		}
		.peekable();

		while let Some(u) = it.next() {
			match form.r#type.as_ref() {
				// TODO: rename to "url"
				"path" => {
					s.extend_from_slice(&form.separator.transform(&u.to_strand()));
				}
				"dirname" => {
					if let Some(p) = u.parent() {
						s.extend_from_slice(&form.separator.transform(&p.to_strand()));
					}
				}
				"filename" => {
					s.extend_from_slice(&form.separator.transform(&u.name().unwrap_or_default()));
				}
				"name_without_ext" => {
					s.extend_from_slice(&form.separator.transform(&u.stem().unwrap_or_default()));
				}
				"uri_list" => {
					s.extend_from_slice(b"file://");
					s.extend_from_slice(&form.separator.transform(&u.to_strand()));
					s.push(b'\r');
				}
				_ => bail!("Unknown copy type: {}", form.r#type),
			};
			if it.peek().is_some() {
				s.push(b'\n');
			}
		}

		// Copy the CWD path regardless even if the directory is empty
		if s.is_empty() && form.r#type == "dirname" {
			s.extend_from_slice(&form.separator.transform(&cx.cwd().to_strand()));
		}

		let mut data = Vec::<ClipboardData>::new();
		match form.r#type.as_ref() {
			"uri_list" => {
				let gnome = [b"copy\n".to_vec(), s.clone()].concat();
				data.push(ClipboardData {
					mime:    b"text/uri-list".to_vec(),
					payload: s,
					alias:   b"text/plain".to_vec(),
				});
				// Because Thunar (and likely others) won't reconize `text/uri-list`
				data.push(ClipboardData {
					mime:    b"x-special/gnome-copied-files".to_vec(),
					payload: gnome,
					alias:   vec![],
				});
			}
			_ => {
				data.push(ClipboardData { mime: b"text/plain".to_vec(), payload: s, alias: vec![] });
			}
		}

		// futures::executor::block_on(CLIPBOARD.set(s));
		futures::executor::block_on(CLIPBOARD.write(data));
		succ!();
	}
}
