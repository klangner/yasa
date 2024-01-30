#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod power_sink;

use eframe::{egui, CreationContext};
use futuresdr::anyhow::Result;
use futuresdr::blocks::seify::SourceBuilder;
use futuresdr::blocks::Fft;
use futuresdr::macros::connect;
use futuresdr::runtime::{Flowgraph, Runtime};

use crate::power_sink::PowerSink;


pub struct Radio {
}

impl Radio {
    pub fn start() -> Result<Self> {
        let frequency = 91.8 * 1e6;
        let source = SourceBuilder::new()
            .frequency(frequency)
            .sample_rate(1e6)
            .gain(30.0)
            .build()?;
        
        let fft = Fft::new(1024);
        let sink = PowerSink::new();

        // Create the `Flowgraph` and add `Block`s
        let runtime = Runtime::new();
        let mut fg = Flowgraph::new();
        connect!(fg, source > fft > sink);

        // Start the flowgraph
        let (_res, _handle) = runtime.start_sync(fg);
        
        Ok(Self {})
    }
}


struct YasaApp {
    _radio: Radio,
}

impl<'a> YasaApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        // init GUI
        cc.egui_ctx.set_zoom_factor(1.5);
        let radio = Radio::start().unwrap();

        Self {
            _radio: radio,
        }
    }
}


impl eframe::App for YasaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("bookmarks_panel")
            .resizable(true)
            .default_width(200.0)
            .width_range(100.0..=300.0)
            .show(ctx, |ui| {
                let v: f32 = 0.0;
                ui.label(format!("Power =  {} dBFS (-30 for Antyradio)", v));
            });

        // Needs to be last
        egui::CentralPanel::default()
            .show(ctx, |_ui| {
            });
    }
}


fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // Init GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Spectrum",
        options,
        Box::new(|cc| Box::new(YasaApp::new(cc))),
    )
}
