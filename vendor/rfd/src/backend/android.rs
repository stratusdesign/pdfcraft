//! Android (added for PDFCraft's Android app; not in upstream rfd).
//!
//! Android has no file dialog an app can show by path. The app registers a host with
//! [`crate::android::set_host`] — its own folder browser — and every dialog is handed to it as a
//! [`crate::android::Request`]; the host answers with local paths (empty = cancelled). The
//! asynchronous dialogs wait for that answer; message dialogs answer at once.

use std::path::PathBuf;

use super::{
    AsyncFilePickerDialogImpl, AsyncFileSaveDialogImpl, AsyncFolderPickerDialogImpl, AsyncMessageDialogImpl, DialogFutureType, FilePickerDialogImpl,
    FileSaveDialogImpl, FolderPickerDialogImpl, MessageDialogImpl,
};
use crate::android::{Kind, ask};
use crate::message_dialog::MessageDialogResult;
use crate::{FileDialog, FileHandle, MessageDialog};

fn wait(dialog: FileDialog, kind: Kind) -> Vec<PathBuf> {
    ask(&dialog, kind).recv().unwrap_or_default()
}

fn waiting(dialog: FileDialog, kind: Kind) -> DialogFutureType<Vec<PathBuf>> {
    // Asked now, on the caller's thread; the answer is waited for wherever the future is polled.
    let rx = ask(&dialog, kind);
    Box::pin(async move { rx.recv().unwrap_or_default() })
}

impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        wait(self, Kind::Open).into_iter().next()
    }
    fn pick_files(self) -> Option<Vec<PathBuf>> {
        Some(wait(self, Kind::OpenMany)).filter(|v| !v.is_empty())
    }
}

impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        wait(self, Kind::Save).into_iter().next()
    }
}

impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        wait(self, Kind::Folder).into_iter().next()
    }
    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        Some(wait(self, Kind::Folder)).filter(|v| !v.is_empty())
    }
}

impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let f = waiting(self, Kind::Open);
        Box::pin(async move { f.await.into_iter().next().map(FileHandle::wrap) })
    }
    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let f = waiting(self, Kind::OpenMany);
        Box::pin(async move { Some(f.await.into_iter().map(FileHandle::wrap).collect::<Vec<_>>()).filter(|v| !v.is_empty()) })
    }
}

impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let f = waiting(self, Kind::Save);
        Box::pin(async move { f.await.into_iter().next().map(FileHandle::wrap) })
    }
}

impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        let f = waiting(self, Kind::Folder);
        Box::pin(async move { f.await.into_iter().next().map(FileHandle::wrap) })
    }
    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let f = waiting(self, Kind::Folder);
        Box::pin(async move { Some(f.await.into_iter().map(FileHandle::wrap).collect::<Vec<_>>()).filter(|v| !v.is_empty()) })
    }
}

impl MessageDialogImpl for MessageDialog {
    fn show(self) -> MessageDialogResult {
        log::warn!("{}: {}", self.title, self.description);
        MessageDialogResult::Cancel
    }
}

impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        let r = MessageDialogImpl::show(self);
        Box::pin(async move { r })
    }
}
