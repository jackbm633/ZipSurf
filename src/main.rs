use std::cell::RefCell;
use std::env::args;
use std::rc::Rc;
use eframe::Frame;
use egui::Ui;
use crate::{url::Url};
use crate::browser::Browser;

mod url;
mod tab;
mod node;
mod layout;
mod html_parser;
mod css_parser;
mod selector;
mod browser;
mod chrome;
mod rect;
mod js_context;

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
            let browser = Browser::new(cc);
            browser.borrow_mut().load_first_tab(Url::new(&url).unwrap());
            Ok(Box::new(BrowserAppWrapper { browser: browser.clone()}))
        }))
}

struct BrowserAppWrapper {
    browser: Rc<RefCell<Browser>>,
}

impl eframe::App for BrowserAppWrapper {
    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
        self.browser.borrow_mut().ui(ui, frame);
    }

}


