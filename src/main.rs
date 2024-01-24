#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod backend;

use std::fs;

use eframe::{egui::{self, Key}, CreationContext};
use backend::radio::FMRadio;
use serde::Deserialize;


#[derive(Deserialize)]
struct Config {
   source: Source,
   bookmarks: Vec<Bookmark>,
}

#[derive(Deserialize)]
struct Source {
    frequency: u64,
    gain: f64,
    rate: f64,
    args: String,
}

#[derive(Deserialize)]
struct Bookmark {
    name: String,
    frequency: u64,
}

struct YasaApp<'a> {
    radio: FMRadio<'a>,
    is_running: bool,
    current_freq: u64,
    config: Config,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            source: Default::default(), 
            bookmarks: Vec::default() 
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Self { 
            frequency: 100_000_000, 
            gain: 30.0, 
            rate: 100_000.0, 
            args: String::default(), 
        }
    }
}

impl<'a> YasaApp<'a> {
    fn new(cc: &CreationContext<'_>, radio: FMRadio<'a>, config: Config) -> Self {
        // init GUI
        cc.egui_ctx.set_zoom_factor(1.5);

        Self {
            radio,
            is_running: false,
            current_freq: config.source.frequency,
            config,
        }
    }
    
    // Every frame we build full UI
    fn draw_ui(&mut self, ctx: &egui::Context) {
        
        egui::TopBottomPanel::top("toolbar")
            .default_height(50.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let run_label = if self.is_running {"Stop"} else {"Run"};
                    let run_btn = ui.button(run_label);
                    if run_btn.clicked() {
                        self.play_stop_action()
                    }
                });
            });

        egui::SidePanel::left("source_panel")
            .resizable(true)
            .default_width(150.0)
            .width_range(80.0..=200.0)
            .show(ctx, |ui| {
                let mut new_freq = 0;
                for bookmark in &self.config.bookmarks {
                    if ui.link(&bookmark.name).clicked() {
                        new_freq = bookmark.frequency;
                    }
                }
                if new_freq > 0 { self.tune_action(new_freq); }
            });

        // Needs to be last
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.label(format!("f =  {}", self.current_freq));
            });
    }

    // Application wide shortcuts
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(Key::P))  { 
            self.play_stop_action() 
        }
    }

    // Start/Stop radio action
    fn play_stop_action(&mut self) {
        if self.is_running {
            self.radio.stop().expect("Can't stop radio");
            self.is_running = false;
        } else {
            let source = &self.config.source;
            self.radio.start(self.current_freq as f64, source.gain, source.rate, &source.args)
                .expect("Can't start radio");
            self.is_running = true;
        }
    }

    // tune to given frequency
    fn tune_action(&mut self, new_freq: u64) {
        if new_freq > 10 && new_freq < 1_500_000_000 {
            self.radio.tune_to(new_freq as f64).expect("Tune error"); 
        }
    }
}


impl eframe::App for YasaApp<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw_ui(ctx);
        self.handle_shortcuts(ctx);
    }

}


fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let config = match fs::read_to_string("config.toml") {
        Ok(c) => toml::from_str(&c).unwrap(),
        Err(_) => Config::default(),
    };

    // Init backend
    let radio = FMRadio::init();
    
    // Init GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Yet Another SDR App",
        options,
        Box::new(|cc| Box::new(YasaApp::new(cc, radio, config))),
    )
}
