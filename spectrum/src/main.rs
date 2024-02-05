#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod channel_sink;
mod radio;

use eframe::{egui::{self, *}, CreationContext};
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

    fn plot_spectrum(&self, ui: &mut Ui, data: &[f32]) {
        let color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(196)
        } else {
            Color32::from_black_alpha(240)
        };

        Frame::canvas(ui.style()).show(ui, |ui| {
            let rect = ui.available_rect_before_wrap();

            let to_screen =
                emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=data.len() as f32, 0.0..=-100.0), rect);

            let points: Vec<Pos2> = data.iter().enumerate()
                .map(|(i, v)| {
                    to_screen * pos2(i as f32,*v)
                })
                .collect();

            let shape = epaint::Shape::line(points, Stroke::new(1., color));
            ui.painter().add(shape);
        });
    }
}


impl eframe::App for YasaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                let data = self.radio.items();
                self.plot_spectrum(ui, &data);
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
