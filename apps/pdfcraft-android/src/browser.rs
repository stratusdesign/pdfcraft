//! PdfCraft's file dialogs on Android: one folder browser for Open, Save As, Export and folder
//! choices, drawn over the app. ChromeOS shows its own picker to Android apps only for
//! opening files, so the app browses the user's folders itself and every dialog looks and works the
//! same: locations and recent folders on the left, the folder's contents on the right, and when
//! saving a name box. The app's file dialogs (rfd, with the Android backend in `vendor/rfd`) reach
//! it as a [`Request`] and get back local paths, as from a desktop dialog ([`crate::files`] does
//! the rest).

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use rfd::android::{Kind, Request};

use crate::files::{Files, Item};
use crate::page;

/// PdfCraft's colour for the page's left column (GridCraft's is green); see [`page::View::accent`].
const ACCENT: [u32; 3] = [0xB7472A, 0x4A2620, 0xF6DED8];

type Answer = Vec<PathBuf>;

/// The dialog the app asked for and where its answer goes; shared with rfd's Android host.
pub type Pending = Arc<Mutex<Option<(Request, Sender<Answer>)>>>;

/// Registers the browser as rfd's Android host: queue the request, answer later from
/// [`Browser::show`].
pub fn install(pending: Pending, ctx: egui::Context) {
    rfd::android::set_host(Box::new(move |req, tx| {
        // A new dialog replaces one still open; dropping its sender answers it as cancelled.
        if let Ok(mut p) = pending.lock() {
            *p = Some((req, tx));
        }
        ctx.request_repaint();
    }));
}

/// Where the browser is; it starts afresh for every dialog.
#[derive(Default)]
pub struct Browser {
    /// The dialog being shown (to notice a new one).
    shown: Option<Request>,
    /// The folders from the starting location down to the one shown: (id, name).
    trail: Vec<(String, String)>,
    listing: Option<Result<Vec<Item>, String>>,
    name: String,
    /// The existing file the user has been asked about replacing.
    replace: Option<String>,
    /// Something to tell the user inside the dialog.
    note: Option<String>,
}


/// The file types a dialog lists: `None` lists every file.
fn wanted(req: &Request) -> Option<Vec<String>> {
    match req.kind {
        Kind::Open | Kind::OpenMany => {
            let exts: Vec<String> = req.filters.iter().flat_map(|(_, e)| e.iter().cloned()).collect();
            if exts.is_empty() || exts.iter().any(|e| e == "*") { None } else { Some(exts) }
        }
        // Saving lists every file, so a name already taken asks before it is replaced.
        Kind::Save => None,
        // Choosing a folder lists folders only.
        Kind::Folder => Some(Vec::new()),
    }
}

fn shown(item: &Item, exts: &Option<Vec<String>>) -> bool {
    item.folder
        || exts.as_ref().is_none_or(|exts| {
            std::path::Path::new(&item.name).extension().and_then(std::ffi::OsStr::to_str).is_some_and(|e| exts.iter().any(|x| x.eq_ignore_ascii_case(e)))
        })
}

impl Browser {
    fn go(&mut self, trail: Vec<(String, String)>) {
        self.trail = trail;
        self.listing = None;
        self.replace = None;
    }

    /// Draws the dialog the app is waiting on, if any, and answers it.
    pub fn show(&mut self, ctx: &egui::Context, pending: &Pending, files: &Rc<RefCell<Files>>) {
        let Some(req) = pending.lock().ok().and_then(|p| p.as_ref().map(|(r, _)| r.clone())) else {
            self.shown = None;
            return;
        };
        if self.shown.as_ref() != Some(&req) {
            *self = Browser::default();
            if req.kind == Kind::Save {
                self.name = req.file_name.clone().unwrap_or_default();
            }
            self.shown = Some(req.clone());
        }
        let saving = req.kind == Kind::Save;
        let choosing_folder = req.kind == Kind::Folder;
        let (places, recent) = {
            let f = files.borrow();
            (f.places(), f.recent())
        };
        if self.trail.is_empty()
            && let Some(start) = recent.first().or(places.first())
        {
            self.go(vec![(start.id.clone(), start.name.clone())]);
        }
        if self.listing.is_none()
            && let Some((id, _)) = self.trail.last()
        {
            let exts = wanted(&req);
            self.listing = Some(Files::list(id).map(|mut items| {
                items.retain(|i| shown(i, &exts));
                items.sort_by(|a, b| b.folder.cmp(&a.folder).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
                items
            }));
        }

        let mut answer: Option<Answer> = None;
        let mut choose_folder = false;
        let title = req.title.clone().unwrap_or_else(|| (if saving { "Save As" } else if choosing_folder { "Choose a folder" } else { "Open" }).to_string());
        let mut add_place = false;
        let mut go = None;
        let mut picked = None;
        let mut save_now = false;
        let mut up = None;
        let typed = self.name.clone();
        let view = page::View {
            title: &title,
            app: "PdfCraft",
            saving,
            choosing_folder: choosing_folder,
            places: &places,
            recent: &recent,
            trail: &self.trail,
            listing: self.listing.as_ref(),
            replacing: self.replace.is_some(),
            note: self.note.as_deref(),
            accent: ACCENT,
        };
        let mut name = typed.clone();
        let actions = page::show(ctx, &view, &mut name);
        if name != typed {
            // A different name drops the question about replacing the old one.
            self.replace = None;
        }
        self.name = name;
        for action in actions {
            match action {
                page::Action::Cancel => answer = Some(Vec::new()),
                page::Action::AddPlace => add_place = true,
                page::Action::Go(item) => go = Some(item),
                page::Action::Pick(item) => picked = Some(item),
                page::Action::Up(i) => up = Some(i),
                page::Action::Save | page::Action::Replace => save_now = true,
                page::Action::ChooseFolder => choose_folder = true,
            }
        }
        if let Some(i) = up {
            let trail = self.trail[..=i].to_vec();
            self.go(trail);
        }
        if choose_folder && let Some((folder, folder_name)) = self.trail.last().cloned() {
            match files.borrow_mut().begin_folder(&folder, &folder_name) {
                Ok(path) => answer = Some(vec![PathBuf::from(path)]),
                Err(e) => self.note = Some(format!("{folder_name}: {e}")),
            }
        }
        if add_place {
            Files::add_place();
        }
        if let Some(place) = go {
            self.go(vec![(place.id, place.name)]);
        }
        if let Some(item) = picked {
            if item.folder {
                let mut trail = self.trail.clone();
                trail.push((item.id, item.name));
                self.go(trail);
            } else if saving {
                self.name = item.name;
                self.replace = None;
            } else {
                let (folder, folder_name) = self.trail.last().cloned().unwrap_or_default();
                match files.borrow_mut().open(&folder, &folder_name, &item) {
                    Ok(path) => answer = Some(vec![PathBuf::from(path)]),
                    Err(e) => self.note = Some(format!("{}: {e}", item.name)),
                }
            }
        }
        if save_now {
            let name = self.name.trim().to_string();
            let existing = match &self.listing {
                Some(Ok(items)) => items.iter().find(|i| !i.folder && i.name == name).cloned(),
                _ => None,
            };
            if let Some(e) = &existing
                && self.replace.as_deref() != Some(e.id.as_str())
            {
                self.replace = Some(e.id.clone());
            } else if let Some((folder, folder_name)) = self.trail.last().cloned() {
                match files.borrow_mut().begin_save(&folder, &folder_name, &name, existing.as_ref()) {
                    Ok(path) => answer = Some(vec![PathBuf::from(path)]),
                    Err(e) => self.note = Some(format!("{name}: {e}")),
                }
            }
        }
        if let Some(answer) = answer
            && let Some((_, tx)) = pending.lock().ok().and_then(|mut p| p.take())
        {
            let _ = tx.send(answer);
            ctx.request_repaint();
        }
    }
}
