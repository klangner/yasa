#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod backend;

use eframe::{egui, CreationContext};
use backend::radio::FMRadio;

struct YasaApp<'a> {
    radio: FMRadio<'a>,
    is_running: bool,
}

impl<'a> YasaApp<'a> {
    fn new(cc: &CreationContext<'_>, radio: FMRadio<'a>) -> Self {
        // init GUI
        cc.egui_ctx.set_zoom_factor(1.5);

        Self {
            radio,
            is_running: false,
        }
    }
}

impl eframe::App for YasaApp<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        egui::TopBottomPanel::top("toolbar")
            .default_height(50.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let run_label = if self.is_running {"Stop"} else {"Run"};
                    let run_btn = ui.button(run_label);
                    if run_btn.clicked() {
                        if self.is_running {
                            self.radio.stop().unwrap();
                            self.is_running = false;
                        } else {
                            self.radio.start().unwrap();
                            self.is_running = true;
                        }

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

}


fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    let radio = FMRadio::init();

    eframe::run_native(
        "Yet Another SDR App",
        options,
        Box::new(|cc| Box::new(YasaApp::new(cc, radio))),
    )
}
