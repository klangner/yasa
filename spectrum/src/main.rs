#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// mod power_sink;
mod channel_sink;

use crossbeam::channel::{self, Receiver};
use dsp::num_complex::Complex32;
use eframe::{egui, CreationContext};
use egui_plot::{Line, Plot, PlotPoints};
use futuresdr::anyhow::Result;
use futuresdr::blocks::seify::SourceBuilder;
use futuresdr::blocks::{Apply, Fft};
use futuresdr::macros::connect;
use futuresdr::runtime::{Block, Flowgraph, Runtime};

use crate::channel_sink::CrossbeamSink;

const FFT_SIZE: usize = 4096;

pub struct Radio {
    receiver: Receiver<Box<[f32]>>,
}

impl Radio {
    pub fn start() -> Result<Self> {
        let source_rate: usize = 3_000_000; 
        let frequency = 91.8 * 1e6;
        let source = SourceBuilder::new()
            .frequency(frequency)
            .sample_rate(source_rate as f64)
            .gain(30.0)
            .build()?;
        
        // let resample = FirBuilder::new_resampling::<Complex32, Complex32>(1, 4);
        let fft = Fft::with_options(
            FFT_SIZE,
            futuresdr::blocks::FftDirection::Forward,
            true,
            None,
        );
        let power = Self::lin2power_db(); 
        let (tx, rx) = channel::unbounded::<Box<[f32]>>();
        let sink = CrossbeamSink::new(tx.clone());

        // Create the `Flowgraph` and add `Block`s
        let runtime = Runtime::new();
        let mut fg = Flowgraph::new();
        connect!(fg, source > fft > power > sink);

        // Start the flowgraph
        let (_res, _handle) = runtime.start_sync(fg);
        
        Ok(Self {receiver: rx})
    }

    pub fn lin2power_db() -> Block {
        Apply::new(|x: &Complex32| 20.0 * (x.norm() / i8::MAX as f32).log10())
    }

    pub fn items(&mut self) -> Vec<f32> {
        let data = self.receiver.recv().unwrap();
        data.into_vec()
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
            radio: radio,
        }
    }
}


impl eframe::App for YasaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                let data = self.radio.items();
                let points: PlotPoints = data.iter().enumerate().map(|(i, v)| [i as f64, *v as f64]).collect();
                let line = Line::new(points);
                Plot::new("spectrum")
                    .view_aspect(3.0)
                    .show(ui, |plot_ui| plot_ui.line(line));
            });

        ctx.request_repaint();
    }
}


fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // Init GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Spectrum",
        options,
        Box::new(|cc| Box::new(YasaApp::new(cc))),
    )
}
