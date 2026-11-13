//! The host side of the Android backend (`backend/android.rs`; added for PDFCraft, not upstream):
//! the app's own folder browser registers here and answers every file dialog.

use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::mpsc::{Receiver, channel};

use crate::FileDialog;

/// What a dialog asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Open,
    OpenMany,
    Folder,
    Save,
}

/// A file dialog, as the host sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub kind: Kind,
    /// The file types offered: (name, extensions); empty offers every file.
    pub filters: Vec<(String, Vec<String>)>,
    /// The name a save starts with.
    pub file_name: Option<String>,
    pub title: Option<String>,
}

/// The host: shows the dialog, and sends the chosen local paths (none = cancelled).
pub type Host = Box<dyn Fn(Request, std::sync::mpsc::Sender<Vec<PathBuf>>) + Send + Sync>;

static HOST: OnceLock<Host> = OnceLock::new();

/// Registers the app's file dialog host (once).
pub fn set_host(host: Host) {
    let _ = HOST.set(host);
}

pub(crate) fn ask(dialog: &FileDialog, kind: Kind) -> Receiver<Vec<PathBuf>> {
    let (tx, rx) = channel();
    let request = Request {
        kind,
        filters: dialog.filters.iter().map(|f| (f.name.clone(), f.extensions.clone())).collect(),
        file_name: dialog.file_name.clone(),
        title: dialog.title.clone(),
    };
    match HOST.get() {
        Some(host) => host(request, tx),
        // No host: answered as cancelled (the sender is dropped).
        None => drop(tx),
    }
    rx
}
