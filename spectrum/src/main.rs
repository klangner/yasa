#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod channel_sink;
mod radio;

use eframe::{egui, CreationContext};
use egui_plot::{Line, Plot, PlotPoints};

use radio::Radio;


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
