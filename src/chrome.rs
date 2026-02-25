use std::cell::RefCell;
use std::rc::{Weak};
use std::sync::Arc;
use eframe::epaint::{FontFamily, FontId};
use egui::{Color32, Pos2, Rect};
use egui::WidgetText::Galley;
use crate::browser::Browser;

pub struct Chrome {
    pub(crate) browser: Weak<RefCell<Browser>>,
    font_id: Option<FontId>,
    line_height: f32,
    padding: f32,
    tabbar_top: f32,
    tabbar_bottom: f32,
    newtab_rect: Rect
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
            newtab_rect: Rect::ZERO
        }
    }

    pub fn init(&mut self, ctx: &egui::Context) {
        if self.font_id.is_some() {
            return;
        }

        let font_id = FontId::new(20.0, FontFamily::Name(Arc::from("sansbold".to_string())));

        let line_height = ctx.fonts_mut(|f| f.row_height(&font_id));
        
        self.font_id = Some(font_id.clone());
        self.line_height = line_height;
        self.tabbar_top = line_height + 10.0;

        let plus_galley = ctx.fonts_mut(|f| f.layout("+".into(),
                                                     font_id, Color32::BLACK, 0.0));
        let plus_width = plus_galley.size().x + 10.0;
        self.newtab_rect = Rect::from_two_pos(Pos2::new(self.padding, self.padding),
                                              Pos2::new(self.padding + plus_width,
                                                        self.padding + self.line_height))

    }

    fn tab_rect(&self, ctx: &egui::Context, i: usize) -> Rect {
        let tabs_start = self.newtab_rect.right() + self.padding;
        let text_width = ctx.fonts_mut(|f| f.layout("Tab X".into(),
                                                    self.font_id.clone().unwrap(), Color32::BLACK,
                                                    0.0)).size().x;
        let tab_width = text_width + 10.0;
        let tab_height = self.line_height;
        let tab_rect = Rect::from_two_pos(Pos2::new(tabs_start + i as f32 * tab_width, self.tabbar_top),
                                          Pos2::new(tabs_start + (i + 1) as f32 * tab_width, self.tabbar_top + tab_height));
        tab_rect
    }
}
