use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use eframe::epaint::{FontFamily, FontId};
use egui::Color32;
use crate::browser::Browser;

pub struct Chrome {
    pub(crate) browser: Weak<RefCell<Browser>>,
    font_id: Option<FontId>,
    line_height: f32,
    padding: f32,
    tabbar_top: f32,
    tabbar_bottom: f32
}

impl Chrome {
    pub fn new(browser: Weak<RefCell<Browser>>, ctx: &egui::Context) -> Self {


        Self {
            browser,
            font_id: None,
            line_height: 0.0,
            padding: 5.0,
            tabbar_top: 0.0,
            tabbar_bottom: 10.0,
        }
    }

    pub fn init(&mut self, ctx: &egui::Context) {
        if self.font_id.is_some() {
            return;
        }

        let font_id = FontId::new(20.0, FontFamily::Name(Arc::from("sansbold".to_string())));

        let line_height = ctx.fonts_mut(|f| f.row_height(&font_id));
        
        self.font_id = Some(font_id);
        self.line_height = line_height;
        self.tabbar_top = line_height + 10.0;
    }
}
