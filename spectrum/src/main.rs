#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, CreationContext};
use futuresdr::anyhow::Result;
use futuresdr::blocks::seify::SourceBuilder;
use futuresdr::blocks::{Apply, Fft, VectorSink};
use futuresdr::macros::connect;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::{Flowgraph, FlowgraphHandle, Runtime};


pub struct Radio {
    handle: FlowgraphHandle,
    sink: usize,
}

impl Radio {
    pub fn start() -> Result<Self> {
        let frequency = 92.1 * 1e6;
        let source = SourceBuilder::new()
            .frequency(frequency)
            .sample_rate(1e6)
            .gain(30.0)
            .build()?;
        
        let fft = Fft::new(1024);
        let norm = Apply::new(|c: &Complex32| -> f32 {c.norm()});
        let sink = VectorSink::<f32>::new(1024);

        // Create the `Flowgraph` and add `Block`s
        let runtime = Runtime::new();
        let mut fg = Flowgraph::new();
        connect!(fg, source > fft > norm > sink);

        // Start the flowgraph
        let (_res, handle) = runtime.start_sync(fg);
        
        Ok(Self {handle, sink})
    }

    pub fn get_samples(&self) -> &Vec<f32> {
        let kernel = self.fg.kernel::<VectorSink<f32>>(self.sink).unwrap();
        kernel.items()
    }
}


struct YasaApp {
    radio: Radio,
}

impl<'a> YasaApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        // init GUI
        cc.egui_ctx.set_zoom_factor(1.5);
        let radio = Radio::start().unwrap();

        Self {
            radio,
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
                let xs = self.radio.get_samples();
                let v: f32 = xs.iter().sum::<f32>() / xs.len() as f32;
                ui.label(format!("f =  {}", v));
            });

        // Needs to be last
        egui::CentralPanel::default()
            .show(ctx, |ui| {
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
