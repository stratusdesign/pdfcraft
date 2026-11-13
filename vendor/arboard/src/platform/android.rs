//! Android (added for PDFCraft's Android app; not in upstream arboard): no system clipboard is
//! reachable from here, so every call reports `ClipboardNotSupported` and the app falls back to
//! egui's own clipboard.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

#[cfg(feature = "image-data")]
use crate::ImageData;
use crate::common::Error;

pub(crate) struct Clipboard;

impl Clipboard {
	pub(crate) fn new() -> Result<Clipboard, Error> {
		Err(Error::ClipboardNotSupported)
	}
}

pub(crate) struct Get<'clipboard> {
	_clipboard: &'clipboard mut Clipboard,
}

impl<'clipboard> Get<'clipboard> {
	pub(crate) fn new(clipboard: &'clipboard mut Clipboard) -> Self {
		Self { _clipboard: clipboard }
	}
	pub(crate) fn text(self) -> Result<String, Error> {
		Err(Error::ClipboardNotSupported)
	}
	pub(crate) fn html(self) -> Result<String, Error> {
		Err(Error::ClipboardNotSupported)
	}
	#[cfg(feature = "image-data")]
	pub(crate) fn image(self) -> Result<ImageData<'static>, Error> {
		Err(Error::ClipboardNotSupported)
	}
	pub(crate) fn file_list(self) -> Result<Vec<PathBuf>, Error> {
		Err(Error::ClipboardNotSupported)
	}
}

pub(crate) struct Set<'clipboard> {
	_clipboard: &'clipboard mut Clipboard,
}

impl<'clipboard> Set<'clipboard> {
	pub(crate) fn new(clipboard: &'clipboard mut Clipboard) -> Self {
		Self { _clipboard: clipboard }
	}
	pub(crate) fn text(self, _data: Cow<'_, str>) -> Result<(), Error> {
		Err(Error::ClipboardNotSupported)
	}
	pub(crate) fn html(self, _html: Cow<'_, str>, _alt: Option<Cow<'_, str>>) -> Result<(), Error> {
		Err(Error::ClipboardNotSupported)
	}
	#[cfg(feature = "image-data")]
	pub(crate) fn image(self, _data: ImageData) -> Result<(), Error> {
		Err(Error::ClipboardNotSupported)
	}
	pub(crate) fn file_list(self, _file_list: &[impl AsRef<Path>]) -> Result<(), Error> {
		Err(Error::ClipboardNotSupported)
	}
}

pub(crate) struct Clear<'clipboard> {
	_clipboard: &'clipboard mut Clipboard,
}

impl<'clipboard> Clear<'clipboard> {
	pub(crate) fn new(clipboard: &'clipboard mut Clipboard) -> Self {
		Self { _clipboard: clipboard }
	}
	pub(crate) fn clear(self) -> Result<(), Error> {
		Err(Error::ClipboardNotSupported)
	}
}
