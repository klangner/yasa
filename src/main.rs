#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod backend;

use std::fs;

use eframe::{egui::{self, Key}, CreationContext};
use backend::radio::FMRadio;
use serde::Deserialize;


#[derive(Deserialize)]
struct Config {
   source: Source,
}

#[derive(Deserialize)]
struct Source {
    frequency: f64,
    gain: f64,
    rate: f64,
    args: String,
}

struct YasaApp<'a> {
    radio: FMRadio<'a>,
    is_running: bool,
    config: Config,
}

impl Default for Config {
    fn default() -> Self {
        Self { source: Default::default() }
    }
}

impl Default for Source {
    fn default() -> Self {
        Self { 
            frequency: 100_000_000.0, 
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

        // egui::SidePanel::left("source_panel")
        //     .resizable(true)
        //     .default_width(150.0)
        //     .width_range(80.0..=200.0)
        //     .show(ctx, |ui| {
        //         ui.label("Source")
        //     });

        // egui::SidePanel::right("output_panel")
        //     .resizable(true)
        //     .default_width(500.0)
        //     // .width_range(100.0..=300.0)
        //     .show(ctx, |ui| {
        //         ui.label("Output")
        //     });

        // Needs to be last
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.label("Bookmarks")
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
            self.radio.start(source.frequency, source.gain, source.rate, &source.args)
                .expect("Can't start radio");
            self.is_running = true;
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
