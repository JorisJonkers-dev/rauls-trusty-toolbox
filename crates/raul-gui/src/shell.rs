// SPDX-License-Identifier: GPL-3.0-only
//! The app shell of `docs/design/mockup.html` (UI-RULES P1, A1, A2, G1 to G6): the top bar with the setup strip
//! and Play, the sidebar of two groups, the page header, the footer status line, routing by number keys, and the
//! dialog layer. Below 860 px the sidebar becomes a wrapping bar under the top bar.
//!
//! It also holds the one error widget every error surface draws through ([`error_widget`]): selectable text and a
//! "Copy details" button whose text [`copy_text`] builds.

use crate::catalogue as msg;
use crate::dialog::{Answer, Dialog, Dialogs};
use crate::jobs::Jobs;
use crate::pages::{self, Page, PageCx, ShellState};
use crate::theme::{self, Tokens};
use eframe::egui::{self, Color32, RichText, Stroke};
use raul_ui_model::job::{JobId, JobState};
use raul_ui_model::pages::{Group, PageId, Severity, View};
use std::collections::HashSet;
use std::path::PathBuf;

/// Below this width the sidebar wraps into a bar under the top bar (UI-RULES P1).
pub const NARROW: f32 = 860.0;
/// The sidebar's width.
const SIDEBAR: f32 = 224.0;

/// The whole window.
pub struct App {
    tokens: Tokens,
    pages: Vec<Box<dyn Page>>,
    view: View,
    shell: ShellState,
    dialogs: Dialogs,
    jobs: Jobs,
    styled: bool,
    smoke: Option<u32>,
    instance: InstanceLookup,
}

/// How the shell keeps [`ShellState::instance`] current: one `raul config get instance` at start, and another after
/// each `raul config set` or `unset` job ends.
#[derive(Debug, Default)]
struct InstanceLookup {
    /// Whether the first lookup was started.
    asked: bool,
    /// The Instance is given, as in a fixture: nothing looks it up.
    fixed: bool,
    /// The lookup that runs.
    job: Option<JobId>,
    /// Configuration changes already seen to end.
    seen: HashSet<JobId>,
    /// A configuration change ended while a lookup ran: look again once it ends.
    again: bool,
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("view", &self.view)
            .finish_non_exhaustive()
    }
}

impl App {
    /// The shell on Home with every registered page, running its jobs through `jobs`.
    pub fn new(jobs: Jobs) -> App {
        App {
            tokens: theme::DARK,
            pages: pages::registry(),
            view: View::Page(PageId::Home),
            shell: ShellState::default(),
            dialogs: Dialogs::default(),
            jobs,
            styled: false,
            smoke: None,
            instance: InstanceLookup::default(),
        }
    }

    /// The shell showing only `page` with `shell` around it, as the snapshot tests render a fixture.
    ///
    /// # Panics
    /// When the fixture's view is not in the page registry, so a fixture never renders as a blank page.
    pub fn fixture(fixture: pages::Fixture, jobs: Jobs) -> App {
        let view = fixture.page.view();
        let mut app = App::new(jobs);
        let slot = app
            .pages
            .iter_mut()
            .find(|p| p.view() == view)
            .expect("a fixture's view is in the page registry");
        *slot = fixture.page;
        app.view = view;
        app.shell = fixture.shell;
        // A fixture's shell state is given; nothing looks the Instance up behind it.
        app.instance.fixed = true;
        app
    }

    /// Starts `command` with `rest` as a page would; for tests.
    ///
    /// # Errors
    /// As [`Jobs::start`].
    pub fn start_job(
        &mut self,
        command: &raul_ui_service::registry::Command,
        rest: Vec<String>,
    ) -> Result<JobId, raul_ui_model::job::JobError> {
        self.jobs
            .start(command.action, command.args(rest), command.writes)
    }

    /// Waits until every job started so far has ended, as tests do before they look.
    pub fn wait_for_jobs(&mut self, timeout: std::time::Duration) -> bool {
        self.jobs.wait_all(timeout)
    }

    /// Closes the window after `frames` frames: the headless smoke test of the release build.
    #[must_use]
    pub fn closing_after(mut self, frames: u32) -> App {
        self.smoke = Some(frames);
        self
    }

    /// What is shown.
    pub fn view(&self) -> View {
        self.view
    }

    /// Opens `view`.
    pub fn open(&mut self, view: View) {
        self.view = view;
    }

    /// The dialog layer.
    pub fn dialogs(&self) -> &Dialogs {
        &self.dialogs
    }

    /// The shared shell state.
    pub fn shell_mut(&mut self) -> &mut ShellState {
        &mut self.shell
    }

    /// Draws one frame.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        if !self.styled {
            self.jobs.repaint_with(&ctx);
            self.styled = true;
        }
        if !crate::is_set_up(&ctx) {
            // Fonts and styles apply from the next frame; draw from then on.
            crate::set_up(&ctx);
            ctx.request_repaint();
            return;
        }
        self.jobs.poll();
        self.track_instance();
        let narrow = ctx.content_rect().width() < NARROW;
        let t = self.tokens;
        let bar = egui::Frame::new()
            .fill(t.bg.color())
            .inner_margin(egui::Margin::symmetric(if narrow { 14 } else { 20 }, 8));
        egui::Panel::top("top")
            .frame(bar)
            .show(ui, |ui| self.top_bar(ui));
        egui::Panel::bottom("foot")
            .frame(bar.inner_margin(egui::Margin::symmetric(20, 6)))
            .show(ui, |ui| self.footer(ui));
        if narrow {
            egui::Panel::top("nav")
                .frame(bar.inner_margin(egui::Margin::symmetric(8, 6)))
                .show(ui, |ui| self.nav(ui, true));
        } else {
            egui::Panel::left("side")
                .frame(bar.inner_margin(egui::Margin::symmetric(10, 12)))
                .resizable(false)
                .exact_size(SIDEBAR)
                .show(ui, |ui| self.nav(ui, false));
        }
        let main = egui::Frame::new()
            .fill(t.bg.color())
            .inner_margin(if narrow {
                egui::Margin::symmetric(16, 16)
            } else {
                egui::Margin {
                    left: 32,
                    right: 32,
                    top: 22,
                    bottom: 32,
                }
            });
        egui::CentralPanel::default_margins()
            .frame(main)
            .show(ui, |ui| self.content(ui));
        self.dialogs.show(&ctx, &t);
        self.answers();
        // After the frame's UI, so a dialog or a text field it opened already holds the keys.
        self.keys(&ctx);
        if let Some(view) = self.shell.go_to.take() {
            self.view = view;
        }
        if let Some(left) = &mut self.smoke {
            if *left == 0 {
                crate::SMOKE_FRAMES_DRAWN.store(true, std::sync::atomic::Ordering::SeqCst);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                *left -= 1;
                ctx.request_repaint();
            }
        }
    }

    /// Keeps [`ShellState::instance`] current (see [`InstanceLookup`]).
    fn track_instance(&mut self) {
        let lookup = &mut self.instance;
        if lookup.fixed {
            return;
        }
        if let Some(id) = lookup.job
            && self.jobs.is_over(id)
        {
            lookup.job = None;
            if let Some(found) = self
                .jobs
                .output(id)
                .filter(|o| o.code == Some(0))
                .and_then(|o| instance_of(&o.stdout))
            {
                self.shell.instance = found;
            }
        }
        let changed = self
            .jobs
            .table()
            .jobs()
            .iter()
            .filter(|j| {
                j.op == raul_ui_service::registry::Action::Config.command()
                    && j.writes
                    && matches!(j.state, JobState::Finished(_))
            })
            .filter(|j| lookup.seen.insert(j.id))
            .count()
            > 0;
        if changed {
            lookup.again = true;
        }
        let first = !lookup.asked;
        if lookup.job.is_none() && (first || lookup.again) {
            lookup.asked = true;
            lookup.again = false;
            if let Some(cmd) = raul_ui_service::registry::command("config-get") {
                lookup.job = self
                    .jobs
                    .start(cmd.action, cmd.args(["instance".to_owned()]), cmd.writes)
                    .ok();
            }
        }
    }

    /// Number keys open the pages, unless a text field or a dialog has the keyboard (UI-RULES P1, K1).
    fn keys(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() || self.dialogs.is_open() {
            return;
        }
        let keys = [
            egui::Key::Num1,
            egui::Key::Num2,
            egui::Key::Num3,
            egui::Key::Num4,
            egui::Key::Num5,
            egui::Key::Num6,
        ];
        let pressed = ctx.input(|i| {
            keys.iter()
                .position(|k| i.key_pressed(*k) && i.modifiers.is_none())
        });
        if let Some(index) = pressed
            && let Some(page) = u8::try_from(index + 1).ok().and_then(PageId::from_key)
        {
            self.view = View::Page(page);
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let t = self.tokens;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.label(
                RichText::new(msg::BRAND)
                    .color(t.brand.color())
                    .family(egui::FontFamily::Name(theme::SEMIBOLD.into())),
            );
            let nv = self.shell.new_vegas.clone();
            if setup_entry(ui, &t, msg::NEW_VEGAS, nv.as_deref()).clicked() {
                self.view = View::Page(PageId::Settings);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let blocked = self.play_block();
                let mut play = ui.add(primary_button(&t, msg::PLAY));
                if let Some(why) = blocked {
                    play = play.on_hover_text(why.what);
                }
                if play.clicked() {
                    self.play(blocked);
                }
            });
        });
    }

    /// Why Play is blocked: what a page said, or that there is no game folder.
    fn play_block(&self) -> Option<msg::ErrorText> {
        self.shell
            .play_blocked
            .or(if self.shell.new_vegas.is_none() {
                Some(msg::PLAY_NO_GAME)
            } else {
                None
            })
    }

    /// Play: blocked, it explains why and offers the fix (UI-RULES G3); otherwise Home starts the game.
    fn play(&mut self, blocked: Option<msg::ErrorText>) {
        if let Some(why) = blocked {
            self.dialogs.open(
                Dialog::confirm(
                    "play-blocked",
                    msg::PLAY_BLOCKED_TITLE,
                    why.detail,
                    msg::PAGE_SETTINGS,
                )
                .with_error(why),
            );
        } else {
            self.shell.play_requested = true;
            self.view = View::Page(PageId::Home);
        }
    }

    /// Acts on the shell's own dialogs.
    fn answers(&mut self) {
        if self.dialogs.take_answer("play-blocked") == Some(Answer::Confirmed) {
            self.view = View::Page(PageId::Settings);
        }
        let _ = self.dialogs.take_answer("about");
    }

    fn nav(&mut self, ui: &mut egui::Ui, narrow: bool) {
        let t = self.tokens;
        let draw = |ui: &mut egui::Ui, app: &mut App| {
            for group in [Group::Play, Group::System] {
                if !narrow {
                    if group == Group::System {
                        ui.add_space(8.0);
                    }
                    section_label(ui, &t, group_title(group), false);
                }
                for page in PageId::ALL.into_iter().filter(|p| p.group() == group) {
                    let view = View::Page(page);
                    let attention = app
                        .pages
                        .iter()
                        .find(|p| p.view() == view)
                        .and_then(|p| p.attention());
                    let title = page_title(page);
                    if nav_entry(ui, &t, title, app.view == view, attention.as_ref(), !narrow)
                        .clicked()
                    {
                        app.view = view;
                    }
                }
            }
        };
        if narrow {
            ui.horizontal_wrapped(|ui| draw(ui, self));
        } else {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                draw(ui, self);
            });
        }
    }

    fn content(&mut self, ui: &mut egui::Ui) {
        let t = self.tokens;
        let view = self.view;
        let Some(page) = self.pages.iter_mut().find(|p| p.view() == view) else {
            return;
        };
        let mut cx = PageCx {
            tokens: &t,
            shell: &mut self.shell,
            dialogs: &mut self.dialogs,
            jobs: &mut self.jobs,
            repaint: Some(ui.ctx().clone()),
        };
        // The header (UI-RULES A1): title, one-line state, tools on the right, then a hairline.
        let mut shown_line = String::new();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(page.title()).text_style(theme::title_style()));
            shown_line = page.state_line();
            ui.label(RichText::new(&shown_line).color(t.muted.color()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                page.tools(ui, &mut cx);
            });
        });
        ui.add_space(10.0);
        hairline(ui, &t);
        ui.add_space(14.0);
        page.pinned(ui, &mut cx);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| page.ui(ui, &mut cx));
        // The header is drawn before the page, so a state the page changed while drawing shows in the next frame:
        // ask for it, or the line under the title keeps the old state until the person moves the mouse.
        if page.state_line() != shown_line {
            ui.ctx().request_repaint();
        }
    }

    fn footer(&mut self, ui: &mut egui::Ui) {
        let t = self.tokens;
        // Wrapped, so a long status line and its "Copy details" button move onto the next line rather than past
        // the right edge at the smallest window (UI-RULES A5, A6).
        ui.horizontal_wrapped(|ui| {
            let status = self
                .shell
                .status
                .clone()
                .unwrap_or_else(|| msg::READY.to_owned());
            let running = self
                .jobs
                .table()
                .jobs()
                .iter()
                .filter(|j| matches!(j.state, raul_ui_model::job::JobState::Running))
                .count();
            // A status line that reports a failure is an error surface too: it can be selected and copied.
            let failed = self.shell.status_error;
            let colour = if failed.is_some() {
                t.err.color()
            } else {
                t.text.color()
            };
            let line = ui.add(
                egui::Label::new(RichText::new(status).color(colour))
                    .truncate()
                    .selectable(failed.is_some()),
            );
            if let Some(why) = failed {
                line.on_hover_text(why.detail);
                if ui.button(msg::COPY_DETAILS).clicked() {
                    ui.ctx()
                        .copy_text(copy_text(why.what, why.what, why.detail));
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new(msg::ABOUT).color(t.muted.color()))
                            .frame(false),
                    )
                    .clicked()
                {
                    self.dialogs.open(about());
                }
                if running > 0 {
                    ui.label(
                        RichText::new(format!("{running} {}", msg::JOB_RUNNING))
                            .color(t.info.color()),
                    );
                }
            });
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

/// The About dialog: what the Toolbox is, its own version, and every bundled asset with its licence (UI-RULES L1).
pub fn about() -> Dialog {
    let mut body = vec![
        msg::ABOUT_BODY.to_owned(),
        format!("{} {}", msg::ABOUT_VERSION, env!("CARGO_PKG_VERSION")),
        msg::ABOUT_ASSETS.to_owned(),
    ];
    body.extend(
        crate::fonts::NOTICES
            .iter()
            .map(|n| format!("{}, {}, {}", n.name, n.license, n.source)),
    );
    Dialog::info("about", msg::ABOUT_TITLE, body)
}

/// The Instance a `raul config get instance --json` document names: `Some(None)` when it is not set, `None` when
/// the output is not such a document, carries an error or has no `instance` entry (the shell then keeps what it
/// had).
pub fn instance_of(stdout: &str) -> Option<Option<String>> {
    let doc: serde_json::Value = serde_json::from_str(stdout).ok()?;
    if doc.get("contract").and_then(serde_json::Value::as_str) != Some("raul-config")
        || doc
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            != Some(1)
        || !doc.get("error").is_none_or(serde_json::Value::is_null)
    {
        return None;
    }
    let entry = doc
        .get("entries")?
        .as_array()?
        .iter()
        .find(|e| e.get("key").and_then(serde_json::Value::as_str) == Some("instance"))?;
    match entry.get("value")? {
        serde_json::Value::Null => Some(None),
        serde_json::Value::String(path) => Some(Some(path.clone())),
        _ => None,
    }
}

/// The title of a sidebar page.
pub fn page_title(page: PageId) -> &'static str {
    match page {
        PageId::Home => msg::PAGE_HOME,
        PageId::Mods => msg::PAGE_MODS,
        PageId::LoadOrder => msg::PAGE_LOAD_ORDER,
        PageId::Lod => msg::PAGE_LOD,
        PageId::Versions => msg::PAGE_VERSIONS,
        PageId::Settings => msg::PAGE_SETTINGS,
    }
}

fn group_title(group: Group) -> &'static str {
    match group {
        Group::Play => msg::GROUP_PLAY,
        Group::System => msg::GROUP_SYSTEM,
    }
}

/// The colour of a severity (UI-RULES C3).
pub fn severity_colour(t: &Tokens, s: Severity) -> Color32 {
    match s {
        Severity::Info => t.info.color(),
        Severity::Warning => t.warn.color(),
        Severity::Problem => t.err.color(),
    }
}

fn setup_entry(ui: &mut egui::Ui, t: &Tokens, what: &str, value: Option<&str>) -> egui::Response {
    let base = theme::base(ui.ctx());
    let value_text = value.unwrap_or(msg::NOT_SET);
    let mut job = egui::text::LayoutJob::default();
    job.append(
        what,
        0.0,
        egui::TextFormat::simple(
            egui::FontId::proportional(base * 0.84),
            t.muted.color(),
        ),
    );
    let value_font = if value.is_some() {
        egui::FontId::monospace(base * 0.9)
    } else {
        egui::FontId::proportional(base)
    };
    job.append(
        value_text,
        8.0,
        egui::TextFormat::simple(
            value_font,
            if value.is_some() {
                t.text.color()
            } else {
                t.warn.color()
            },
        ),
    );
    let response = ui.add(egui::Button::new(job).frame(false).truncate());
    if let Some(full) = value {
        response.on_hover_text(full)
    } else {
        response
    }
}

fn nav_entry(
    ui: &mut egui::Ui,
    t: &Tokens,
    title: &str,
    current: bool,
    attention: Option<&raul_ui_model::pages::Attention>,
    full_width: bool,
) -> egui::Response {
    let base = theme::base(ui.ctx());
    let font = if current {
        egui::FontId::new(base, egui::FontFamily::Name(theme::SEMIBOLD.into()))
    } else {
        egui::FontId::proportional(base)
    };
    let colour = if current { t.text } else { t.muted };
    let galley = ui
        .painter()
        .layout_no_wrap(title.to_owned(), font, colour.color());
    let chip = attention.map(|a| {
        ui.painter().layout_no_wrap(
            a.count.to_string(),
            egui::FontId::proportional(base * 0.78),
            severity_colour(t, a.severity),
        )
    });
    let chip_width = chip.as_ref().map_or(0.0, |c| c.size().x + 20.0);
    let width = if full_width {
        ui.available_width()
    } else {
        galley.size().x + 20.0 + chip_width
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 34.0), egui::Sense::click());
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, current, title));
    let p = ui.painter();
    if current {
        p.rect_filled(rect, 6.0, t.sel.color());
    } else if response.hovered() {
        p.rect_filled(rect, 6.0, t.hover.color());
    }
    if response.has_focus() {
        p.rect_stroke(
            rect,
            6.0,
            Stroke::new(2.0, t.accent.color()),
            egui::StrokeKind::Inside,
        );
    }
    let text_pos = egui::pos2(rect.left() + 10.0, rect.center().y - galley.size().y / 2.0);
    p.galley(text_pos, galley, colour.color());
    if let (Some(chip), Some(a)) = (chip, attention) {
        let size = chip.size() + egui::vec2(12.0, 6.0);
        let chip_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 8.0 - size.x / 2.0, rect.center().y),
            size,
        );
        p.rect_filled(
            chip_rect,
            size.y / 2.0,
            severity_colour(t, a.severity).gamma_multiply(0.16),
        );
        p.galley(chip_rect.center() - chip.size() / 2.0, chip, t.text.color());
    }
    if current {
        // The active marker (mockup: a 3 px accent bar at the left, or under the entry when narrow).
        let bar = if full_width {
            egui::Rect::from_min_max(
                egui::pos2(rect.left() - 10.0, rect.top() + 8.0),
                egui::pos2(rect.left() - 7.0, rect.bottom() - 8.0),
            )
        } else {
            egui::Rect::from_min_max(
                egui::pos2(rect.left() + 8.0, rect.bottom() - 2.0),
                egui::pos2(rect.right() - 8.0, rect.bottom()),
            )
        };
        p.rect_filled(bar, 2.0, t.accent.color());
    }
    response
}

/// A hairline across the available width (UI-RULES A2).
pub fn hairline(ui: &mut egui::Ui, t: &Tokens) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, t.line.color());
}

/// A section label: small uppercase text with the amber bracket tick (UI-RULES A4). The text is stored as written;
/// the uppercase is applied here.
pub fn section_label(ui: &mut egui::Ui, t: &Tokens, text: &str, tick: bool) {
    ui.horizontal(|ui| {
        if tick {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 8.0), egui::Sense::hover());
            let stroke = Stroke::new(2.0, t.brand.color());
            let p = ui.painter();
            p.line_segment([rect.left_bottom(), rect.left_top()], stroke);
            p.line_segment(
                [rect.left_top(), rect.left_top() + egui::vec2(9.0, 0.0)],
                stroke,
            );
        }
        ui.label(
            RichText::new(text.to_uppercase())
                .text_style(theme::section_style())
                .color(t.muted.color())
                .extra_letter_spacing(1.2),
        );
    });
}

/// A primary button: accent fill (UI-RULES T3).
pub fn primary_button<'a>(t: &Tokens, text: &'a str) -> egui::Button<'a> {
    egui::Button::new(
        RichText::new(text)
            .color(t.on_accent.color())
            .family(egui::FontFamily::Name(theme::SEMIBOLD.into())),
    )
    .fill(t.accent.color())
    .stroke(Stroke::new(1.0, t.accent.color()))
}

/// A danger button: red text and border (UI-RULES T3).
pub fn danger_button<'a>(t: &Tokens, text: &'a str) -> egui::Button<'a> {
    egui::Button::new(RichText::new(text).color(t.err.color()))
        .fill(t.raise.color())
        .stroke(Stroke::new(1.0, t.err.color()))
}

/// The `raul` command an action runs, in monospace, so it can be typed in a terminal (UI-RULES G6).
pub fn command_line(ui: &mut egui::Ui, t: &Tokens, command: &str) {
    egui::Frame::new()
        .fill(t.bg.color())
        .stroke(Stroke::new(1.0, t.line.color()))
        .corner_radius(6)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(RichText::new(command).monospace().color(t.text.color())).wrap(),
            );
        });
}

/// The text an error's "Copy details" button puts on the clipboard: the shared shape of
/// [`raul_ui_model::copy_details`], with this build's version and the system it runs on.
pub fn copy_text(doing: &str, title: &str, detail: &str) -> String {
    let system = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    raul_ui_model::copy_details(
        doing,
        title,
        detail,
        env!("CARGO_PKG_VERSION"),
        &system,
        home().as_deref(),
    )
}

/// The user's home folder, so a copy writes it as `~`; `None` when the system does not say.
fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|h| !h.as_os_str().is_empty())
}

/// `line` without the command a tool names at its start (`raul update check: the list could not be read` shows as
/// `the list could not be read`); the copied details keep the whole line.
fn without_command(line: &str) -> &str {
    let mut words = line.split_whitespace();
    if matches!(words.next(), Some("raul" | "nvmod"))
        && let Some((_, rest)) = line.split_once(": ")
    {
        return rest;
    }
    line
}

/// An error anywhere in the window (UI-RULES W3, G4): what was being done, what went wrong, its detail lines as
/// selectable text, and a "Copy details" button that puts the plain, complete text on the clipboard.
///
/// A short error block, a dialog, the status line and a failed job all draw through here, so every one of them can
/// be read with the mouse and the keyboard and copied the same way. `what` is what was being done, `title` what
/// went wrong, and `detail` its lines.
pub fn error_widget(ui: &mut egui::Ui, t: &Tokens, what: &str, title: &str, detail: &str) {
    error_widget_copying(ui, t, what, title, detail, detail);
}

/// [`error_widget`] where the text on the clipboard (`copied`) is not the text on screen (`detail`): for a raw
/// line that is useful in a bug report but means nothing to the player.
pub fn error_widget_copying(
    ui: &mut egui::Ui,
    t: &Tokens,
    what: &str,
    title: &str,
    detail: &str,
    copied: &str,
) {
    ui.horizontal_top(|ui| {
        crate::pages::home::icon(ui, t, crate::pages::home::Mark::Problem);
        ui.vertical(|ui| {
            ui.label(RichText::new(what).color(t.muted.color()));
            ui.add(
                egui::Label::new(RichText::new(title).color(t.err.color()))
                    .wrap()
                    .selectable(true),
            );
            for line in detail.lines() {
                ui.add(
                    egui::Label::new(RichText::new(without_command(line)).color(t.muted.color()))
                        .wrap()
                        .selectable(true),
                );
            }
            ui.add_space(4.0);
            if ui.button(msg::COPY_DETAILS).clicked() {
                ui.ctx().copy_text(copy_text(what, title, copied));
            }
        });
    });
}

/// Raul's one short line, only in calm places: the wizard, the Home launch panel and all-clear states (UI-RULES G5).
/// `line` is one of the catalogue's `RAUL_*` strings.
pub fn raul_line(ui: &mut egui::Ui, t: &Tokens, line: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(msg::RAUL.to_uppercase())
                .text_style(theme::section_style())
                .color(t.brand.color())
                .extra_letter_spacing(1.2),
        );
        ui.label(RichText::new(line).color(t.muted.color()));
    });
}
