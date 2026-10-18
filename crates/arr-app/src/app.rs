//! Pure-Rust `egui` desktop UI for `rust-arr-mcp`.
//!
//! Enforces the 10-Point Uncrashable Binary Contract:
//! - Zero synchronous network / keyring I/O on the UI thread (`self.spawn(...)`).
//! - Zero `ctx` reads inside `ctx.input(|i| ...)`; zoom applied outside.
//! - Strict glyph allowlist (`°`, `×`, `·`, `…`) verified by `glyph_source_scan`.
//! - No dead buttons: every button navigates, spawns a job, saves, or toggles.

use arr_core::command::{dispatch_auto, Command};
use eframe::egui::{self, Color32, RichText};
use serde_json::{json, Value};
use std::future::Future;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

const BG_DARK: Color32 = Color32::from_rgb(9, 13, 22);
const PANEL_DARK: Color32 = Color32::from_rgb(15, 23, 42);
const CARD_DARK: Color32 = Color32::from_rgb(22, 32, 56);
const BORDER_DARK: Color32 = Color32::from_rgb(45, 62, 96);
const ACCENT: Color32 = Color32::from_rgb(56, 189, 248);
const EMERALD: Color32 = Color32::from_rgb(52, 211, 153);
const AMBER: Color32 = Color32::from_rgb(251, 191, 36);
const ROSE: Color32 = Color32::from_rgb(251, 113, 133);

const LLM_PROVIDERS: &[(&str, &str, &str, &str)] = &[
    (
        "ollama",
        "Ollama (Local :11434)",
        "http://127.0.0.1:11434/v1",
        "gemma4:12b",
    ),
    (
        "lmstudio",
        "LM Studio (Local :1234)",
        "http://127.0.0.1:1234/v1",
        "local-model",
    ),
    (
        "openai",
        "OpenAI (Cloud)",
        "https://api.openai.com/v1",
        "gpt-4o-mini",
    ),
    (
        "anthropic",
        "Anthropic (Cloud)",
        "https://api.anthropic.com",
        "claude-sonnet-4-20250514",
    ),
    ("custom", "Custom / OpenAI-Compatible", "", ""),
];

const SERVICES: &[(&str, &str)] = &[
    ("sonarr", "Sonarr · TV"),
    ("radarr", "Radarr · Movies"),
    ("lidarr", "Lidarr · Music"),
    ("readarr", "Readarr · Books"),
    ("bazarr", "Bazarr · Subtitles"),
    ("prowlarr", "Prowlarr · Indexers"),
    ("overseerr", "Overseerr · Requests"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    // SERVICES
    Dashboard,
    Sonarr,
    Radarr,
    Lidarr,
    Readarr,
    Bazarr,
    Prowlarr,
    Overseerr,
    // DISCOVER
    Orchestrate,
    Inspector,
    Inbox,
    // MANAGE
    Health,
    // AI & FLEET
    Chat,
    Crossconnect,
    // SYSTEM
    Tools,
    Logs,
    Settings,
    Help,
    RawJson,
}

impl Page {
    fn title(&self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Sonarr => "Sonarr",
            Self::Radarr => "Radarr",
            Self::Lidarr => "Lidarr",
            Self::Readarr => "Readarr",
            Self::Bazarr => "Bazarr",
            Self::Prowlarr => "Prowlarr",
            Self::Overseerr => "Overseerr",
            Self::Orchestrate => "Orchestrate",
            Self::Inspector => "Inspector",
            Self::Inbox => "Inbox",
            Self::Health => "Health",
            Self::Chat => "Chat",
            Self::Crossconnect => "Crossconnect",
            Self::Tools => "Tools",
            Self::Logs => "Logs",
            Self::Settings => "Settings",
            Self::Help => "Help",
            Self::RawJson => "Raw JSON",
        }
    }
}

enum Job {
    Series(Value),
    Movies(Value),
    Artists(Value),
    Authors(Value),
    Subtitles(Value),
    Indexers(Value),
    Requests(Value),
    Health(Value),
    ChatReply(Value),
    StatusNote(String, Value),
    VaultLoaded {
        service_urls: Value,
        llm_api_key: String,
    },
    LlmModelsLoaded(Value),
    LlmTestResult(Value),
    RegistryLoaded(crate::registry::Registry),
    RegistrySaved(crate::registry::Registry),
}

pub struct ArrApp {
    runtime: Arc<tokio::runtime::Runtime>,
    tx: Sender<Job>,
    rx: Receiver<Job>,
    pending: usize,
    theme: Theme,
    sidebar_open: bool,
    zoom: f32,
    page: Page,
    status_line: String,
    series_val: Value,
    movies_val: Value,
    artists_val: Value,
    authors_val: Value,
    subtitles_val: Value,
    indexers_val: Value,
    requests_val: Value,
    health_val: Value,
    last_raw: Value,
    q_text: String,
    chat_provider: String,
    chat_model: String,
    chat_prompt: String,
    chat_reply: String,
    llm_base_url: String,
    llm_api_key: String,
    available_models: Vec<String>,
    llm_test_status: String,
    svc_select: String,
    svc_url: String,
    svc_key: String,
    vault_loaded: bool,
    vault_status_note: String,
    pinned: crate::registry::Registry,
    event_logs: Vec<String>,
    log_filter: String,
}

fn apply_theme(ctx: &egui::Context, theme: Theme) {
    let mut visuals = match theme {
        Theme::Dark => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
    };
    if theme == Theme::Dark {
        visuals.panel_fill = PANEL_DARK;
        visuals.extreme_bg_color = BG_DARK;
    }
    ctx.set_visuals(visuals);
}

impl ArrApp {
    pub fn new(cc: &eframe::CreationContext<'_>, runtime: Arc<tokio::runtime::Runtime>) -> Self {
        apply_theme(&cc.egui_ctx, Theme::Dark);
        let (tx, rx) = mpsc::channel();
        let mut app = Self {
            runtime,
            tx,
            rx,
            pending: 0,
            theme: Theme::Dark,
            sidebar_open: true,
            zoom: 1.0,
            page: Page::Dashboard,
            status_line: "Loading Arr stack…".to_string(),
            series_val: json!({}),
            movies_val: json!({}),
            artists_val: json!({}),
            authors_val: json!({}),
            subtitles_val: json!({}),
            indexers_val: json!({}),
            requests_val: json!({}),
            health_val: json!({}),
            last_raw: json!({}),
            q_text: String::new(),
            chat_provider: "ollama".to_string(),
            chat_model: "gemma4:12b".to_string(),
            chat_prompt: "Summarize what is downloading across my Arr stack.".to_string(),
            chat_reply: String::new(),
            llm_base_url: "http://127.0.0.1:11434/v1".to_string(),
            llm_api_key: String::new(),
            available_models: Vec::new(),
            llm_test_status: String::new(),
            svc_select: "sonarr".to_string(),
            svc_url: String::new(),
            svc_key: String::new(),
            vault_loaded: false,
            vault_status_note: "Keyring: loading…".to_string(),
            pinned: crate::registry::Registry::default(),
            event_logs: vec!["[+0s] Arr Rust desktop app initialized.".to_string()],
            log_filter: String::new(),
        };
        app.spawn_initial();
        app
    }

    fn spawn<F, Fut>(&mut self, f: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Job> + Send + 'static,
    {
        self.pending = self.pending.saturating_add(1);
        let tx = self.tx.clone();
        self.runtime.spawn(async move {
            let job = f().await;
            let _ = tx.send(job);
        });
    }

    fn spawn_initial(&mut self) {
        self.status_line = "Loading Arr stack…".to_string();
        self.spawn(|| async {
            Job::Series(
                dispatch_auto(Command::SonarrSeriesList)
                    .await
                    .unwrap_or(Value::Null),
            )
        });
        self.spawn(|| async {
            Job::Movies(
                dispatch_auto(Command::RadarrMoviesList)
                    .await
                    .unwrap_or(Value::Null),
            )
        });
        self.spawn(|| async {
            Job::Artists(
                dispatch_auto(Command::LidarrArtistsList { mb_id: None })
                    .await
                    .unwrap_or(Value::Null),
            )
        });
        self.spawn(|| async {
            Job::Authors(
                dispatch_auto(Command::ReadarrAuthorsList)
                    .await
                    .unwrap_or(Value::Null),
            )
        });
        self.spawn(|| async {
            Job::Subtitles(
                dispatch_auto(Command::BazarrWanted {
                    kind: "all".to_string(),
                })
                .await
                .unwrap_or(Value::Null),
            )
        });
        self.spawn(|| async {
            Job::Indexers(
                dispatch_auto(Command::ProwlarrIndexersList)
                    .await
                    .unwrap_or(Value::Null),
            )
        });
        self.spawn(|| async {
            Job::Requests(
                dispatch_auto(Command::OverseerrRequestsList)
                    .await
                    .unwrap_or(Value::Null),
            )
        });
        self.spawn(vault_load_job);
        self.spawn(|| async { Job::RegistryLoaded(crate::registry::load()) });
    }

    fn drain_rx(&mut self) {
        let mut done = 0;
        while let Ok(job) = self.rx.try_recv() {
            done += 1;
            match job {
                Job::Series(v) => {
                    self.series_val = v.clone();
                    self.last_raw = v;
                    self.status_line = "Series loaded.".to_string();
                }
                Job::Movies(v) => {
                    self.movies_val = v.clone();
                    self.last_raw = v;
                }
                Job::Artists(v) => {
                    self.artists_val = v.clone();
                    self.last_raw = v;
                }
                Job::Authors(v) => {
                    self.authors_val = v.clone();
                    self.last_raw = v;
                }
                Job::Subtitles(v) => {
                    self.subtitles_val = v.clone();
                    self.last_raw = v;
                }
                Job::Indexers(v) => {
                    self.indexers_val = v.clone();
                    self.last_raw = v;
                }
                Job::Requests(v) => {
                    self.requests_val = v.clone();
                    self.last_raw = v;
                }
                Job::Health(v) => {
                    self.health_val = v.clone();
                    self.last_raw = v;
                    self.status_line = "Health refreshed.".to_string();
                }
                Job::ChatReply(v) => {
                    self.chat_reply = v
                        .get("reply")
                        .and_then(|s| s.as_str())
                        .unwrap_or("No reply text.")
                        .to_string();
                    self.last_raw = v;
                    self.status_line = "Chat reply received.".to_string();
                }
                Job::StatusNote(note, v) => {
                    self.status_line = note;
                    self.last_raw = v;
                }
                Job::VaultLoaded {
                    service_urls,
                    llm_api_key,
                } => {
                    self.vault_loaded = true;
                    self.vault_status_note = "Keyring: loaded".to_string();
                    if let Some(url) = service_urls.get(&self.svc_select).and_then(|v| v.as_str()) {
                        self.svc_url = url.to_string();
                    }
                    self.llm_api_key = llm_api_key;
                    self.last_raw = service_urls;
                }
                Job::LlmModelsLoaded(v) => {
                    self.available_models = v
                        .get("data")
                        .and_then(|d| d.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|m| {
                                    m.get("id").and_then(|i| i.as_str()).map(str::to_string)
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    self.last_raw = v;
                    self.status_line = "Models listed.".to_string();
                }
                Job::LlmTestResult(v) => {
                    let ok = v.get("ok").and_then(Value::as_bool).unwrap_or(false);
                    self.llm_test_status = if ok {
                        "Connected.".to_string()
                    } else {
                        "Failed.".to_string()
                    };
                    self.last_raw = v;
                }
                Job::RegistryLoaded(reg) | Job::RegistrySaved(reg) => {
                    self.pinned = reg;
                    self.status_line = "Profiles updated.".to_string();
                }
            }
        }
        self.pending = self.pending.saturating_sub(done);
    }

    fn log(&mut self, line: String) {
        self.event_logs.push(line);
    }
}

async fn vault_load_job() -> Job {
    let mut urls = serde_json::Map::new();
    for (service, _) in SERVICES {
        let slot = format!("{service}_url");
        if let Ok(Some(url)) = crate::vault::get(&slot) {
            urls.insert(service.to_string(), Value::String(url));
        }
    }
    let llm_api_key = crate::vault::get("llm_api_key")
        .ok()
        .flatten()
        .unwrap_or_default();
    Job::VaultLoaded {
        service_urls: Value::Object(urls),
        llm_api_key,
    }
}

fn count_items(val: &Value) -> String {
    if let Some(arr) = val.as_array() {
        return arr.len().to_string();
    }
    for key in ["total", "count"] {
        if let Some(n) = val.get(key).and_then(Value::as_u64) {
            return n.to_string();
        }
    }
    if val.is_null() {
        return "·".to_string();
    }
    "1".to_string()
}

fn hero_kpi(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    egui::Frame::NONE
        .fill(CARD_DARK)
        .stroke(egui::Stroke::new(1.0, BORDER_DARK))
        .corner_radius(8.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(value).size(28.0).color(color).strong());
                ui.label(RichText::new(label).size(12.0).color(Color32::GRAY));
            });
        });
}

fn value_table(ui: &mut egui::Ui, val: &Value, max_rows: usize) {
    let items: Vec<&Value> = match val {
        Value::Array(items) => items.iter().collect(),
        Value::Object(_) => vec![val],
        _ => {
            ui.label(RichText::new("No data yet - press Refresh.").color(Color32::GRAY));
            return;
        }
    };
    if items.is_empty() {
        ui.label(RichText::new("Empty.").color(Color32::GRAY));
        return;
    }
    let mut columns: Vec<String> = Vec::new();
    if let Some(Value::Object(first)) = items.first() {
        for key in first.keys().take(5) {
            columns.push(key.clone());
        }
    }
    egui::ScrollArea::both().max_height(420.0).show(ui, |ui| {
        egui::Grid::new("value_table").striped(true).show(ui, |ui| {
            for column in &columns {
                ui.label(RichText::new(column).strong());
            }
            ui.end_row();
            for item in items.iter().take(max_rows) {
                for column in &columns {
                    let text = item
                        .get(column)
                        .map(|v| match v {
                            Value::String(s) => {
                                let mut short = String::new();
                                for (idx, ch) in s.chars().enumerate() {
                                    if idx >= 42 {
                                        short.push('…');
                                        break;
                                    }
                                    short.push(ch);
                                }
                                short
                            }
                            other => other.to_string(),
                        })
                        .unwrap_or_else(|| "·".to_string());
                    ui.label(text);
                }
                ui.end_row();
            }
        });
    });
}

impl eframe::App for ArrApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.drain_rx();
        if self.pending > 0 {
            ctx.request_repaint();
        }

        // Zoom input sampled first, applied outside `ctx.input` (no re-entrancy).
        let zoom_delta = ctx.input(|i| i.zoom_delta());
        if (zoom_delta - 1.0).abs() > 0.001 {
            self.zoom = (self.zoom * zoom_delta).clamp(0.5, 2.0);
            ctx.set_zoom_factor(self.zoom);
        }

        egui::Panel::top("arr_topbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Arr Stack");
                ui.separator();
                ui.label(&self.status_line);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.pending > 0 {
                        ui.label(format!("{} jobs…", self.pending));
                    } else {
                        ui.label("idle");
                    }
                });
            });
        });

        if self.sidebar_open {
            egui::Panel::left("arr_sidebar")
                .default_size(190.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if ui.button("Collapse").clicked() {
                            self.sidebar_open = !self.sidebar_open;
                        }
                        for (group, pages) in [
                            (
                                "SERVICES",
                                &[
                                    Page::Dashboard,
                                    Page::Sonarr,
                                    Page::Radarr,
                                    Page::Lidarr,
                                    Page::Readarr,
                                    Page::Bazarr,
                                    Page::Prowlarr,
                                    Page::Overseerr,
                                ][..],
                            ),
                            (
                                "DISCOVER",
                                &[Page::Orchestrate, Page::Inspector, Page::Inbox][..],
                            ),
                            ("MANAGE", &[Page::Health][..]),
                            ("AI & FLEET", &[Page::Chat, Page::Crossconnect][..]),
                            (
                                "SYSTEM",
                                &[
                                    Page::Tools,
                                    Page::Logs,
                                    Page::Settings,
                                    Page::Help,
                                    Page::RawJson,
                                ][..],
                            ),
                        ] {
                            ui.label(
                                RichText::new(group)
                                    .size(11.0)
                                    .color(Color32::GRAY)
                                    .strong(),
                            );
                            for page in pages {
                                let selected = self.page == *page;
                                if ui.selectable_label(selected, page.title()).clicked() {
                                    self.page = *page;
                                }
                            }
                            ui.separator();
                        }
                    });
                });
        } else if ui.button("Expand").clicked() {
            self.sidebar_open = true;
        }

        egui::Panel::bottom("arr_status").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_line);
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(self.page.title());
            ui.separator();
            match self.page {
                Page::Dashboard => self.view_dashboard(ui),
                Page::Sonarr => self.view_service(ui, "sonarr", &self.series_val.clone(), "Series"),
                Page::Radarr => self.view_service(ui, "radarr", &self.movies_val.clone(), "Movies"),
                Page::Lidarr => {
                    self.view_service(ui, "lidarr", &self.artists_val.clone(), "Artists")
                }
                Page::Readarr => {
                    self.view_service(ui, "readarr", &self.authors_val.clone(), "Authors")
                }
                Page::Bazarr => {
                    self.view_service(ui, "bazarr", &self.subtitles_val.clone(), "Subtitles")
                }
                Page::Prowlarr => {
                    self.view_service(ui, "prowlarr", &self.indexers_val.clone(), "Indexers")
                }
                Page::Overseerr => {
                    self.view_service(ui, "overseerr", &self.requests_val.clone(), "Requests")
                }
                Page::Orchestrate => self.view_orchestrate(ui),
                Page::Inspector => self.view_inspector(ui),
                Page::Inbox => self.view_inbox(ui),
                Page::Health => self.view_health(ui),
                Page::Chat => self.view_chat(ui),
                Page::Crossconnect => self.view_crossconnect(ui),
                Page::Tools => self.view_tools(ui),
                Page::Logs => self.view_logs(ui),
                Page::Settings => self.view_settings(ui),
                Page::Help => self.view_help(ui),
                Page::RawJson => self.view_raw(ui),
            }
        });
    }
}

impl ArrApp {
    fn view_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            hero_kpi(ui, "Sonarr series", &count_items(&self.series_val), ACCENT);
            hero_kpi(ui, "Radarr movies", &count_items(&self.movies_val), EMERALD);
            hero_kpi(ui, "Lidarr artists", &count_items(&self.artists_val), AMBER);
            hero_kpi(
                ui,
                "Readarr authors",
                &count_items(&self.authors_val),
                ACCENT,
            );
            hero_kpi(
                ui,
                "Wanted subtitles",
                &count_items(&self.subtitles_val),
                ROSE,
            );
            hero_kpi(
                ui,
                "Prowlarr indexers",
                &count_items(&self.indexers_val),
                EMERALD,
            );
            hero_kpi(
                ui,
                "Overseerr requests",
                &count_items(&self.requests_val),
                AMBER,
            );
        });
        ui.separator();
        if ui.button("Refresh all").clicked() {
            self.spawn_initial();
            self.log("Dashboard refresh requested.".to_string());
        }
    }

    fn view_service(&mut self, ui: &mut egui::Ui, service: &str, val: &Value, noun: &str) {
        let service = service.to_string();
        ui.horizontal(|ui| {
            ui.label(format!("{} {} · {}", count_items(val), noun, service));
            if ui.button("Refresh").clicked() {
                let command = match service.as_str() {
                    "sonarr" => Command::SonarrSeriesList,
                    "radarr" => Command::RadarrMoviesList,
                    "lidarr" => Command::LidarrArtistsList { mb_id: None },
                    "readarr" => Command::ReadarrAuthorsList,
                    "bazarr" => Command::BazarrWanted {
                        kind: "all".to_string(),
                    },
                    "prowlarr" => Command::ProwlarrIndexersList,
                    _ => Command::OverseerrRequestsList,
                };
                self.status_line = format!("Loading {service}…");
                let svc = service.clone();
                self.spawn(move || async move {
                    Job::StatusNote(
                        format!("{svc} loaded."),
                        dispatch_auto(command).await.unwrap_or(Value::Null),
                    )
                });
            }
        });
        ui.horizontal(|ui| {
            ui.label("Lookup:");
            ui.text_edit_singleline(&mut self.q_text);
            if ui.button("Search").clicked() {
                let term = self.q_text.clone();
                let command = match service.as_str() {
                    "sonarr" => Command::SonarrSeriesLookup { term },
                    "radarr" => Command::RadarrMoviesLookup { term },
                    "lidarr" => Command::LidarrArtistsLookup { term },
                    "readarr" => Command::ReadarrAuthorsLookup { term },
                    _ => Command::Status,
                };
                self.spawn(move || async move {
                    Job::StatusNote(
                        "Lookup done.".to_string(),
                        dispatch_auto(command).await.unwrap_or(Value::Null),
                    )
                });
            }
        });
        value_table(ui, val, 200);
    }

    fn view_orchestrate(&mut self, ui: &mut egui::Ui) {
        ui.label("Cross-service orchestration lives in the MCP layer (arr_orchestrate / arr_calendar / arr_stats).");
        ui.label("This page routes there - no placeholder actions:");
        for (label, page) in [
            ("TV -> Sonarr", Page::Sonarr),
            ("Movies -> Radarr", Page::Radarr),
            ("Requests -> Overseerr", Page::Overseerr),
        ] {
            if ui.button(label).clicked() {
                self.page = page;
            }
        }
    }

    fn view_inspector(&mut self, ui: &mut egui::Ui) {
        ui.label("Last result received (any page):");
        value_table(ui, &self.last_raw.clone(), 50);
    }

    fn view_inbox(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{} events this session.", self.event_logs.len()));
        egui::ScrollArea::vertical()
            .max_height(420.0)
            .show(ui, |ui| {
                for line in &self.event_logs.clone() {
                    ui.label(line);
                }
            });
        if ui.button("Clear").clicked() {
            self.event_logs.clear();
        }
    }

    fn view_health(&mut self, ui: &mut egui::Ui) {
        if ui.button("Refresh health").clicked() {
            self.status_line = "Checking services…".to_string();
            self.spawn(|| async {
                let mut services = serde_json::Map::new();
                for (service, command) in [
                    ("sonarr", Command::SonarrSeriesList),
                    ("radarr", Command::RadarrMoviesList),
                    ("lidarr", Command::LidarrArtistsList { mb_id: None }),
                    ("readarr", Command::ReadarrAuthorsList),
                    ("bazarr", Command::BazarrProviders),
                    ("prowlarr", Command::ProwlarrIndexersList),
                    ("overseerr", Command::OverseerrRequestsCount),
                ] {
                    let live = dispatch_auto(command).await.is_ok();
                    services.insert(service.to_string(), Value::Bool(live));
                }
                Job::Health(Value::Object(services))
            });
        }
        value_table(ui, &self.health_val.clone(), 50);
    }

    fn view_chat(&mut self, ui: &mut egui::Ui) {
        ui.label("Provider:");
        egui::ComboBox::from_id_salt("chat_provider")
            .selected_text(&self.chat_provider)
            .show_ui(ui, |ui| {
                for (id, label, url, model) in LLM_PROVIDERS {
                    if ui
                        .selectable_value(&mut self.chat_provider, (*id).to_string(), *label)
                        .clicked()
                    {
                        self.llm_base_url = (*url).to_string();
                        self.chat_model = (*model).to_string();
                    }
                }
            });
        ui.horizontal(|ui| {
            ui.label("Model:");
            ui.text_edit_singleline(&mut self.chat_model);
        });
        ui.label("Prompt:");
        ui.text_edit_multiline(&mut self.chat_prompt);
        if ui.button("Send").clicked() {
            let (base_url, api_key, model, prompt) = (
                self.llm_base_url.clone(),
                self.llm_api_key.clone(),
                self.chat_model.clone(),
                self.chat_prompt.clone(),
            );
            self.status_line = "Chatting…".to_string();
            self.spawn(move || async move {
                Job::ChatReply(chat_completion(&base_url, &api_key, &model, &prompt).await)
            });
        }
        ui.separator();
        ui.label(RichText::new(&self.chat_reply).size(13.0));
    }

    fn view_crossconnect(&mut self, ui: &mut egui::Ui) {
        ui.label("Fleet cross-connects for rust-arr-mcp:");
        ui.label(
            "· jellyfin-mcp reads Bazarr subtitles + Radarr/Sonarr libraries (RAG + playback).",
        );
        ui.label("· aiwatcher receives stack events via bridge webhook.");
        ui.label("Configure endpoints in Settings, then verify with Health -> Refresh.");
    }

    fn view_tools(&mut self, ui: &mut egui::Ui) {
        ui.label("80 MCP tools across 7 services (+ arr.status / arr.health / arr.help).");
        ui.label("Canonical names: <service>.<action> - e.g. sonarr.series_list.");
        ui.label("Legacy aliases: sonarr_series(operation=\"list\") keep working with zero edits.");
        if ui.button("Show raw catalog in Inspector").clicked() {
            self.last_raw = help_catalog_value();
            self.page = Page::Inspector;
        }
    }

    fn view_logs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.log_filter);
            if ui.button("Clear").clicked() {
                self.event_logs.clear();
            }
        });
        let filter = self.log_filter.clone();
        egui::ScrollArea::vertical()
            .max_height(420.0)
            .show(ui, |ui| {
                for line in &self.event_logs.clone() {
                    if filter.is_empty() || line.contains(&filter) {
                        ui.label(line);
                    }
                }
            });
    }

    fn view_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Theme");
        ui.horizontal(|ui| {
            if ui
                .selectable_value(&mut self.theme, Theme::Dark, "Dark")
                .clicked()
            {
                apply_theme(ui.ctx(), Theme::Dark);
            }
            if ui
                .selectable_value(&mut self.theme, Theme::Light, "Light")
                .clicked()
            {
                apply_theme(ui.ctx(), Theme::Light);
            }
        });
        ui.separator();
        ui.heading("Service credentials (OS keyring, never plain text)");
        egui::ComboBox::from_id_salt("svc_select")
            .selected_text(&self.svc_select)
            .show_ui(ui, |ui| {
                for (service, label) in SERVICES {
                    if ui
                        .selectable_value(&mut self.svc_select, (*service).to_string(), *label)
                        .clicked()
                    {
                        self.svc_url.clear();
                        self.svc_key.clear();
                    }
                }
            });
        ui.horizontal(|ui| {
            ui.label("URL:");
            ui.text_edit_singleline(&mut self.svc_url);
        });
        ui.horizontal(|ui| {
            ui.label("API key:");
            ui.text_edit_singleline(&mut self.svc_key);
        });
        ui.label(&self.vault_status_note);
        if ui.button("Save to vault").clicked() {
            let (service, url, key) = (
                self.svc_select.clone(),
                self.svc_url.clone(),
                self.svc_key.clone(),
            );
            self.spawn(move || async move {
                let url_note = crate::vault::set(&format!("{service}_url"), &url)
                    .err()
                    .unwrap_or_else(|| "saved".to_string());
                let key_note = crate::vault::set(&format!("{service}_key"), &key)
                    .err()
                    .unwrap_or_else(|| "saved".to_string());
                Job::StatusNote(
                    format!("Vault: url {url_note} · key {key_note}."),
                    json!({"service": service}),
                )
            });
        }
        ui.separator();
        ui.heading("Pinned profiles (name + URL only, secrets stay in vault)");
        for profile in self.pinned.services.clone() {
            ui.horizontal(|ui| {
                ui.label(format!("{} · {}", profile.name, profile.url));
                if ui.button("Unpin").clicked() {
                    let name = profile.name.clone();
                    self.spawn(move || async move {
                        let mut reg = crate::registry::load();
                        reg.services.retain(|s| s.name != name && s.url != name);
                        crate::registry::save(&reg);
                        Job::RegistrySaved(reg)
                    });
                }
            });
        }
        ui.horizontal(|ui| {
            if ui.button("Pin current service profile").clicked() {
                let (name, url) = (self.svc_select.clone(), self.svc_url.clone());
                self.spawn(move || async move {
                    let mut reg = crate::registry::load();
                    if !reg.services.iter().any(|s| s.name == name) {
                        reg.services.push(crate::registry::RegService { name, url });
                        crate::registry::save(&reg);
                    }
                    Job::RegistrySaved(reg)
                });
            }
        });
        ui.separator();
        ui.heading("LLM providers");
        egui::ComboBox::from_id_salt("llm_provider")
            .selected_text(&self.chat_provider)
            .show_ui(ui, |ui| {
                for (id, label, url, model) in LLM_PROVIDERS {
                    if ui
                        .selectable_value(&mut self.chat_provider, (*id).to_string(), *label)
                        .clicked()
                    {
                        self.llm_base_url = (*url).to_string();
                        self.chat_model = (*model).to_string();
                    }
                }
            });
        ui.horizontal(|ui| {
            ui.label("Base URL:");
            ui.text_edit_singleline(&mut self.llm_base_url);
        });
        ui.horizontal(|ui| {
            ui.label("Model:");
            ui.text_edit_singleline(&mut self.chat_model);
        });
        ui.horizontal(|ui| {
            ui.label("API key:");
            ui.text_edit_singleline(&mut self.llm_api_key);
        });
        ui.horizontal(|ui| {
            if ui.button("List models").clicked() {
                let (base_url, api_key) = (self.llm_base_url.clone(), self.llm_api_key.clone());
                self.spawn(move || async move {
                    Job::LlmModelsLoaded(list_models(&base_url, &api_key).await)
                });
            }
            if ui.button("Save key to vault").clicked() {
                let key = self.llm_api_key.clone();
                self.spawn(move || async move {
                    let note = crate::vault::set("llm_api_key", &key)
                        .err()
                        .unwrap_or_else(|| "saved".to_string());
                    Job::StatusNote(format!("LLM key: {note}."), json!({}))
                });
            }
            if ui.button("Test connection").clicked() {
                let (base_url, api_key) = (self.llm_base_url.clone(), self.llm_api_key.clone());
                self.spawn(move || async move {
                    Job::LlmTestResult(test_llm(&base_url, &api_key).await)
                });
            }
        });
        if !self.available_models.is_empty() {
            ui.label(format!("{} models", self.available_models.len()));
        }
        if !self.llm_test_status.is_empty() {
            ui.label(&self.llm_test_status);
        }
    }

    fn view_help(&mut self, ui: &mut egui::Ui) {
        ui.label("rust-arr-mcp - ArtCraft conversion of arr-mcp (Shadow-Then-Swap).");
        ui.label("CLI: rust-arr-mcp <sonarr|radarr|lidarr|readarr|bazarr|prowlarr|overseerr> <op> --params-json '{}' [--json].");
        ui.label("MCP: stdio 2024-11-05, 82 tools, dual aliases (dot + underscore + legacy).");
        ui.label("Docs: docs/CONVERSION_DOSSIER.md (wire truth) + HANDOVER.md (live state).");
    }

    fn view_raw(&mut self, ui: &mut egui::Ui) {
        let text = serde_json::to_string_pretty(&self.last_raw)
            .unwrap_or_else(|_| self.last_raw.to_string());
        egui::ScrollArea::both().show(ui, |ui| {
            ui.monospace(text);
        });
    }
}

async fn chat_completion(base_url: &str, api_key: &str, model: &str, prompt: &str) -> Value {
    let client = reqwest::Client::new();
    let mut req = client
        .post(format!(
            "{}/chat/completions",
            base_url.trim_end_matches('/')
        ))
        .json(&json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 500,
        }));
    if !api_key.trim().is_empty() {
        req = req.bearer_auth(api_key);
    }
    let resp = match req.send().await {
        Ok(resp) => resp,
        Err(e) => return json!({"reply": format!("chat transport error: {e}")}),
    };
    let body: Value = resp.json().await.unwrap_or(Value::Null);
    let reply = body
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("Empty reply.")
        .to_string();
    json!({"reply": reply})
}

async fn list_models(base_url: &str, api_key: &str) -> Value {
    let client = reqwest::Client::new();
    let mut req = client.get(format!("{}/models", base_url.trim_end_matches('/')));
    if !api_key.trim().is_empty() {
        req = req.bearer_auth(api_key);
    }
    match req.send().await {
        Ok(resp) => resp.json().await.unwrap_or(json!({"data": []})),
        Err(e) => json!({"data": [], "error": e.to_string()}),
    }
}

async fn test_llm(base_url: &str, api_key: &str) -> Value {
    let models = list_models(base_url, api_key).await;
    let ok = models
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    json!({"ok": ok})
}

fn help_catalog_value() -> Value {
    json!({
        "services": SERVICES.iter().map(|(s, l)| json!({"id": s, "label": l})).collect::<Vec<_>>(),
        "tool_count": 82,
    })
}

/// Headless CI smoke check (`cargo run -p arr-app -- --smoke`).
pub fn smoke() -> eframe::Result<()> {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("smoke: runtime init failed: {e}");
            std::process::exit(1);
        }
    };
    let (status, series, movies) = rt.block_on(async {
        let s = dispatch_auto(Command::Status).await;
        let b = dispatch_auto(Command::SonarrSeriesList).await;
        let m = dispatch_auto(Command::RadarrMoviesList).await;
        (s, b, m)
    });
    let ok = status
        .map(|v| v.get("ok") == Some(&Value::Bool(true)))
        .unwrap_or(false)
        && series.map(|v| v.is_array()).unwrap_or(false)
        && movies.map(|v| v.is_array()).unwrap_or(false);
    if ok {
        println!("smoke: status + series_list + movies_list OK");
        Ok(())
    } else {
        eprintln!("smoke: parity check failed");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod parity_tests {
    use super::*;
    use egui_kittest::Harness;

    #[test]
    fn glyph_allowlist() {
        let mut harness = Harness::new_ui(|ui| {
            ui.label("4.20 MB · 3 × 2 · querying…");
        });
        harness.run();
    }

    #[test]
    fn glyph_source_scan() {
        const ALLOWED: &[char] = &['°', '×', '·', '…'];
        let source = include_str!("app.rs");
        let mut offenders = Vec::new();
        for (idx, ch) in source.char_indices() {
            if !ch.is_ascii() && !ALLOWED.contains(&ch) {
                offenders.push((idx, ch));
                if offenders.len() >= 8 {
                    break;
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "non-allowlisted glyphs in app.rs: {offenders:?}"
        );
    }

    #[test]
    fn kpi_and_table_render_cleanly() {
        let mut harness = Harness::new_ui(|ui| {
            hero_kpi(ui, "Sonarr series", "2", ACCENT);
            value_table(ui, &json!([{"id": 42, "title": "Demo Series"}]), 50);
        });
        harness.run();
    }
}
