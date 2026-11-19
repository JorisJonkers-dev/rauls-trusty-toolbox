// SPDX-License-Identifier: GPL-3.0-only
//! The Toolbox GUI on egui (ADR-0012), built into the release `raul` (ADR-0004).
//!
//! * [`shell`]: the window: top bar, sidebar, page header, footer, routing and the dialog layer.
//! * [`pages`]: the page registry, one file per page or flow.
//! * [`theme`], [`fonts`] and [`catalogue`]: the colours, the bundled Inter and every string.
//! * [`dialog`] and [`jobs`]: modal dialogs, and `raul` commands running in the background.
//!
//! The GUI renders the toolkit-free view model of `raul-ui-model` and changes nothing itself: every write is a
//! `raul` command started through `raul-ui-service`.

#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod catalogue;
pub mod dialog;
pub mod engine_check;
pub mod fonts;
pub mod jobs;
pub mod pages;
pub mod shell;
pub mod theme;

use std::process::ExitCode;

/// Set once the window drew the frames the smoke test waits for; `open` reports success only then.
pub static SMOKE_FRAMES_DRAWN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// The environment variable that makes the window close after its first frames: the headless smoke test.
pub const SMOKE_ENV: &str = "RAUL_GUI_SMOKE";

/// Installs the fonts and the dark theme in `ctx`. They apply from the next frame, so a host calls this before the
/// first one (eframe's creation closure, a test harness's context setup); [`shell::App`] does it itself otherwise.
pub fn set_up(ctx: &eframe::egui::Context) {
    fonts::install(ctx);
    theme::apply(ctx, &theme::DARK);
    ctx.data_mut(|d| d.insert_temp(set_up_id(), true));
}

/// Whether [`set_up`] ran on `ctx`.
pub fn is_set_up(ctx: &eframe::egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(set_up_id()).unwrap_or(false))
}

fn set_up_id() -> eframe::egui::Id {
    eframe::egui::Id::unique("raul-gui-set-up")
}

/// The window's native options: 1280 by 800 (the Steam Deck), down to the narrow layout's 402 px.
pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(catalogue::BRAND)
            .with_app_id("dev.jorisjonkers.raul")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([402.0, 480.0]),
        renderer: eframe::Renderer::Glow,
        ..eframe::NativeOptions::default()
    }
}

/// Opens the window and returns when it is closed. With [`SMOKE_ENV`] set, it closes itself after a few frames
/// and says so on standard output.
pub fn open() -> ExitCode {
    let smoke = std::env::var_os(SMOKE_ENV).is_some();
    let result = eframe::run_native(
        catalogue::BRAND,
        native_options(),
        Box::new(move |cc| {
            set_up(&cc.egui_ctx);
            let app = shell::App::new(jobs::Jobs::real());
            Ok(Box::new(if smoke { app.closing_after(3) } else { app }))
        }),
    );
    match result {
        Ok(()) if smoke && !SMOKE_FRAMES_DRAWN.load(std::sync::atomic::Ordering::SeqCst) => {
            eprintln!("raul: the window closed before it drew a frame");
            ExitCode::from(raul_cli::FOUND)
        }
        Ok(()) => {
            if smoke {
                println!("raul: the window opened and closed");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("raul: the window could not be opened: {e}");
            eprintln!("raul: every command still works from a terminal; `raul help` lists them");
            ExitCode::from(raul_cli::FOUND)
        }
    }
}
