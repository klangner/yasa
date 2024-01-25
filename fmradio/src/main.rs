#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod radio;

use std::fs;

use eframe::{egui, CreationContext};
use radio::FMRadio;
use serde::Deserialize;


#[derive(Deserialize)]
struct Config {
   source: Source,
   bookmarks: Vec<Bookmark>,
}

#[derive(Deserialize)]
struct Source {
    gain: f64,
    rate: f64,
    args: String,
}

#[derive(Deserialize)]
struct Bookmark {
    name: String,
    frequency: f64,
}

struct YasaApp {
    radio: FMRadio,
    current_freq: f64,
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
            gain: 30.0, 
            rate: 1_000_000.0, 
            args: String::default(), 
        }
    }
}

impl<'a> YasaApp {
    fn new(cc: &CreationContext<'_>, config: Config) -> Self {
        // init GUI
        cc.egui_ctx.set_zoom_factor(1.5);
        let current_freq = config.bookmarks
            .first()
            .map(|b| b.frequency)
            .unwrap_or(100_000_000.0);
        let src = &config.source;
        let radio = FMRadio::start(current_freq, src.gain, src.rate, &src.args).unwrap();

        Self {
            radio,
            current_freq,
            config,
        }
    }
    
    // tune to given frequency
    fn tune_action(&mut self, new_freq: f64) {
        self.radio.tune_to(new_freq as f64).expect("Tune error"); 
        self.current_freq = new_freq;
    }
}


impl eframe::App for YasaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("bookmarks_panel")
            .resizable(true)
            .default_width(200.0)
            .width_range(100.0..=300.0)
            .show(ctx, |ui| {
                ui.label("Bookmarks");

                let mut new_freq = 0.0;
                for bookmark in &self.config.bookmarks {
                    if ui.link(&bookmark.name).clicked() {
                        new_freq = bookmark.frequency;
                    }
                }
                if new_freq > 0.0 { self.tune_action(new_freq); }
            });

        // Needs to be last
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.label(format!("f =  {}", self.current_freq));
            });
    }
}


fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let config = match fs::read_to_string("config.toml") {
        Ok(c) => toml::from_str(&c).unwrap(),
        Err(_) => Config::default(),
    };

    // Init GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FM Radio",
        options,
        Box::new(|cc| Box::new(YasaApp::new(cc, config))),
    )
}
