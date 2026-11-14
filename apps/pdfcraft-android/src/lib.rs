//! PdfCraft as an Android app (phones, tablets and ChromeOS, which runs Android apps natively).
//!
//! Runs the same [`pdfcraft_ui_egui::PdfCraftApp`] as the desktop app through eframe's Android
//! runner (winit NativeActivity, wgpu on Vulkan or GLES). File dialogs are PdfCraft's own folder
//! browser ([`browser`], reached through rfd's Android backend in `vendor/rfd`) over the folders
//! the user let it use ([`files`]). Build: `.github/workflows/android.yml`.
#![cfg(target_os = "android")]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod browser;
mod clipboard;
mod files;
mod page;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use pdfcraft_ui_egui::PdfCraftApp;
use winit::platform::android::activity::AndroidApp;

struct App {
    app: PdfCraftApp,
    browser: browser::Browser,
    pending: browser::Pending,
    files: Rc<RefCell<files::Files>>,
    clip: clipboard::Bridge,
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.clip.input(raw);
        self.app.raw_input_hook(ctx, raw);
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.app.save(storage);
    }
    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        let _ = self.files.borrow_mut().sync_back();
        self.app.on_exit(gl);
    }
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Documents ChromeOS handed over ("Open with", or PdfCraft as the default app).
        let opened = self.files.borrow_mut().take_opened();
        for o in opened {
            match o {
                Ok((path, _, name, writable)) => {
                    let _ = self.app.open_path(&path);
                    if !writable {
                        self.app.notify(format!("{name} was opened read only. Use Save As to keep your changes."));
                    }
                }
                Err(e) => self.app.notify(e),
            }
        }
        let problems = self.files.borrow_mut().logic();
        if let Some(p) = problems.into_iter().last() {
            self.app.notify(p);
        }
        self.app.logic(ctx, frame);
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
        self.browser.show(ui.ctx(), &self.pending, &self.files);
        self.clip.output(ui.ctx());
    }
}

#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("pdfcraft"));
    files::set_activity(&android_app);
    let data = android_app.internal_data_path();
    let files = Rc::new(RefCell::new(files::Files::new(data.clone())));
    let pending: browser::Pending = Arc::new(Mutex::new(None));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("PdfCraft"),
        persistence_path: data.as_ref().map(|d| d.join("app.ron")),
        android_app: Some(android_app),
        ..Default::default()
    };
    let result = eframe::run_native(
        "PdfCraft",
        options,
        Box::new(move |cc| {
            files::set_context(&cc.egui_ctx);
            browser::install(pending.clone(), cc.egui_ctx.clone());
            let mut app = PdfCraftApp::new();
            app.units = pdfcraft_ui_egui::units::Unit::system();
            if let Some(json) = cc.storage.and_then(|s| s.get_string("pdfcraft")) {
                app.restore(&json);
            }
            Ok(Box::new(App { app, browser: browser::Browser::default(), pending, files, clip: clipboard::Bridge::default() }))
        }),
    );
    if let Err(e) = result {
        log::error!("PdfCraft stopped: {e}");
    }
}
