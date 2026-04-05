use std::cell::{RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;
use eframe::emath::Vec2;
use eframe::epaint::{FontFamily, FontId, Stroke, StrokeKind};
use egui::{Color32, Context, Pos2, Rect};
use egui::WidgetText::Galley;
use crate::browser::Browser;
use crate::layout::{LayoutNode, HEIGHT, WIDTH};
use crate::tab::{DrawCommand, DrawLine, DrawOutline, DrawRect, DrawText, Tab};
use crate::url::Url;

pub struct Chrome {
    pub(crate) browser: Weak<RefCell<Browser>>,
    font_id: Option<FontId>,
    line_height: f32,
    padding: f32,
    tabbar_top: f32,
    tabbar_bottom: f32,
    newtab_rect: Rect,
    pub(crate) draw_commands: Vec<DrawCommand>,
    urlbar_top: f32,
    urlbar_bottom: f32,
    back_rect: Rect,
    address_rect: Rect,
    focus: Focus,
    pub(crate) address_bar: String,
}
#[derive(PartialEq)]
pub enum Focus {
    None,
    AddressBar,
}

pub enum ChromeAction {
    NewTab,
    SelectTab(usize),
    GoBack,
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
            newtab_rect: Rect::ZERO,
            draw_commands: vec!(),
            urlbar_top: 0.0,
            urlbar_bottom: 0.0,
            address_rect: Rect::ZERO,
            back_rect: Rect::ZERO,
            focus: Focus::None,
            address_bar: String::new(),
        }
    }
    
    pub fn bottom(&self) -> f32 {
        self.urlbar_bottom
    }

    pub fn init(&mut self, ctx: &egui::Context) {
        if self.font_id.is_some() {
            return;
        }

        let font_id = FontId::new(10.0, FontFamily::Name(Arc::from("sansbold".to_string())));

        let line_height = ctx.fonts_mut(|f| f.row_height(&font_id));
        
        self.font_id = Some(font_id.clone());
        self.line_height = line_height;;
        self.tabbar_bottom = line_height + 10.0;
        self.urlbar_top = self.tabbar_bottom;
        self.urlbar_bottom = self.urlbar_top + line_height + 2.0 * self.padding;
        let plus_galley = ctx.fonts_mut(|f| f.layout("+".into(),
                                                     font_id.clone(), Color32::BLACK, 0.0));
        let plus_width = plus_galley.size().x + 10.0;
        self.newtab_rect = Rect::from_two_pos(Pos2::new(self.padding, self.padding),
                                              Pos2::new(self.padding + plus_width,
                                                        self.padding + self.line_height));
        let back_galley =  ctx.fonts_mut(|f| f.layout("<".into(),
                                                     font_id, Color32::BLACK, 0.0));
        let back_width = back_galley.size().x + 10.0;
        self.back_rect = Rect::from_two_pos(Pos2::new(self.padding, self.tabbar_bottom + self.padding),
                                           Pos2::new(self.padding + back_width, self.tabbar_bottom + self.padding + self.line_height));
        self.address_rect = Rect::from_two_pos(Pos2::new(self.newtab_rect.right() + self.padding, self.tabbar_bottom
                                                          + self.padding),
                                             Pos2::new(WIDTH - self.padding, self.tabbar_bottom + self.padding + self.line_height));



    }

    fn tab_rect(&self, ctx: &egui::Context, i: usize) -> Rect {
        let tabs_start = self.newtab_rect.right() + self.padding;
        let text_width = ctx.fonts_mut(|f| f.layout_no_wrap("Tab X".into(),
                                                    self.font_id.clone().unwrap(), Color32::BLACK)).size().x;
        let tab_width = text_width + 10.0;
        let tab_height = self.line_height;
        let tab_rect = Rect::from_two_pos(Pos2::new(tabs_start + i as f32 * tab_width, self.tabbar_top),
                                          Pos2::new(tabs_start + (i + 1) as f32 * tab_width, self.tabbar_bottom));
        tab_rect
    }

    pub fn draw(&mut self, ctx: &egui::Context, tabs: &[Rc<RefCell<Tab>>], current_tab: &Rc<RefCell<Tab>>) {
        self.draw_commands.clear();
        // Chrome-specific drawing logic would go here,
        // potentially using the passed 'ui' or its painter.

        self.draw_commands.push(DrawCommand::DrawRect(
            DrawRect {
                rect: Rect::from_two_pos(
                    Pos2::new(0.0, 0.0), Pos2::new(WIDTH, self.bottom())),
                color: Color32::WHITE,
            }
        ));

        self.draw_commands.push(DrawCommand::DrawOutline(
            DrawOutline {
                rect: self.newtab_rect,
                color: Color32::BLACK,
                thickness: 1.0
            }
        ));
        self.draw_commands.push(
            DrawCommand::DrawText(
                DrawText {
                    x: self.newtab_rect.left() + self.padding,
                    y: self.newtab_rect.top(),
                    galley: ctx.fonts_mut(|f| f.layout("+".into(),
                    self.font_id.clone().unwrap(), Color32::BLACK,
                    0.0))
                }
            )
        );

        // Draw each tab
        for (i, tab_rc) in tabs.iter().enumerate() {
            let bounds= self.tab_rect(ctx, i);
            self.draw_commands.push(DrawCommand::DrawLine(
                DrawLine {
                    from: bounds.left_top(),
                    to: bounds.left_bottom(),
                    color: Color32::BLACK,
                    thickness: 1.0
                }
            ));
            self.draw_commands.push(DrawCommand::DrawLine(
                DrawLine {
                    from: bounds.right_top(),
                    to: bounds.right_bottom(),
                    color: Color32::BLACK,
                    thickness: 1.0
                }
            ));

            self.draw_commands.push(
                DrawCommand::DrawText(
                    DrawText {
                        x: bounds.left() + self.padding,
                        y: bounds.top() + self.padding,
                        galley: ctx.fonts_mut(|f| f.layout_no_wrap(format!("Tab {:?}", i).into(),
                                                           self.font_id.clone().unwrap(), Color32::BLACK))
                    }
                )
            );

            if (Rc::ptr_eq(tab_rc, current_tab)) {
                self.draw_commands.push(DrawCommand::DrawLine(
                    DrawLine {
                        from: Pos2::new(0.0, bounds.bottom()),
                        to: bounds.left_bottom(),
                        color: Color32::BLACK,
                        thickness: 1.0
                    }
                ));

                self.draw_commands.push(DrawCommand::DrawLine(
                    DrawLine {
                        from: bounds.right_bottom(),
                        to: Pos2::new(WIDTH, bounds.bottom()),
                        color: Color32::BLACK,
                        thickness: 1.0
                    }
                ));
            }


        }

        self.draw_commands.push(DrawCommand::DrawOutline(
            DrawOutline{
                rect: self.back_rect,
                color: Color32::BLACK,
                thickness: 1.0,
            }
        ));

        self.draw_commands.push(DrawCommand::DrawText(
            DrawText {
                x: self.back_rect.left() + self.padding,
                y: self.back_rect.top(),
                galley: ctx.fonts_mut(|f| f.layout_no_wrap("<".parse().unwrap(),
                                                           self.font_id.clone().unwrap(), Color32::BLACK))
            }
        ));

        self.draw_commands.push(DrawCommand::DrawOutline(
            DrawOutline{
                rect: self.address_rect,
                color: Color32::BLACK,
                thickness: 1.0,
            }

        ));

        let url = current_tab.borrow().url.clone().unwrap().to_string();

        if self.focus == Focus::AddressBar {
            let galley = ctx.fonts_mut(|f| f.layout_no_wrap(self.address_bar.parse().unwrap(),
                                                            self.font_id.clone().unwrap(), Color32::BLACK));
            self.draw_commands.push(DrawCommand::DrawText(
                DrawText {
                    x: self.address_rect.left() + self.padding,
                    y: self.address_rect.top(),
                    galley: ctx.fonts_mut(|f| galley.clone())
                }
            ));

            let w =  galley.clone().size().x;
            self.draw_commands.push(DrawCommand::DrawLine(
                DrawLine {
                    from: Pos2::new(self.address_rect.left() + w + self.padding, self.address_rect.top()),
                    color: Color32::BLUE,
                    thickness: 2.0,
                    to: Pos2::new(self.address_rect.left() + w + self.padding, self.address_rect.bottom()),
                }
            ))
        } else {
            self.draw_commands.push(DrawCommand::DrawText(
                DrawText {
                    x: self.address_rect.left() + self.padding,
                    y: self.address_rect.top(),
                    galley: ctx.fonts_mut(|f| f.layout_no_wrap(url.parse().unwrap(),
                                                               self.font_id.clone().unwrap(), Color32::BLACK))
                }
            ))
        }

    }

    pub fn on_enter(&mut self,  tab: Rc<RefCell<Tab>>)
    {
        if (self.focus == Focus::AddressBar)
        {
            Tab::load(tab, Url::new(&*self.address_bar).unwrap(), None);
            self.focus = Focus::None;
        }

    }

    pub fn click(&mut self, ctx: &Context, pos: Pos2, tab_count: usize) -> Option<ChromeAction> {
        self.focus = Focus::None;
        if self.newtab_rect.contains(pos) {
            return Some(ChromeAction::NewTab);
        }
        if self.back_rect.contains(pos)
        {
            return Some(ChromeAction::GoBack);
        }
        if self.address_rect.contains(pos) {
            self.focus = Focus::AddressBar;
            self.address_bar.clear();
        }

        // Check if any existing tab was clicked
        for i in 0..tab_count {
            if self.tab_rect(ctx, i).contains(pos) {
                return Some(ChromeAction::SelectTab(i));
            }
        }

        None
    }
    pub fn keypress(&mut self, keypress: &String)
    {
        self.address_bar.push_str(keypress);
    }
    
    pub fn blur(&mut self) {
        self.focus = Focus::None;
    }
}
