use std::env::args;


use crate::{url::Url};

mod url;
mod browser;
mod node;
mod layout;
mod html_parser;


fn main() -> eframe::Result<(), eframe::Error> {
    let url = args().skip(1).next().expect("No URL provided.");

    let window_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "ZipSurf Browser",
        window_options,
        Box::new(|cc| {
            let mut browser = browser::Browser::new(cc);
            browser.load(Url::new(&url).unwrap());
            Ok(Box::new(browser))
        }))
        
    
}
