use crate::chrome::{Chrome, ChromeAction};
use crate::layout::HEIGHT;
use crate::measure_time::MeasureTime;
use crate::tab::{self, DrawCommand, Tab, TabMessage};
use crate::url::Url;
use eframe::emath::Pos2;
use eframe::epaint::{Color32, Stroke, StrokeKind};
use egui::{Context, Painter, Ui, Vec2};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::thread;
use eframe::Frame;

/// A `Browser` structure that simulates a web browser with multiple tabs.
///
/// # Fields
///
/// * `tabs` - A vector of `Rc<RefCell<Tab>>`, representing the list of all tabs
///   currently open in the browser. Each tab is wrapped in a `Rc<RefCell<Tab>>`
///   to enable shared ownership and interior mutability.
///
/// * `current_tab` - An `Rc<RefCell<Tab>>` representing the currently active
///   tab in the browser. This allows the browser to keep track of the user's
///   interaction focus.
///
/// # Usage
///
/// This struct is intended to be used to manage and interact with multiple
/// browser tabs, enabling features such as switching between tabs, closing
/// tabs, or interacting with the contents of the current tab.
///
/// # Example
/// ```
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// struct Tab {
///     url: String,
/// }
///
/// let first_tab = Rc::new(RefCell::new(Tab { url: String::from("https://example.com") }));
/// let second_tab = Rc::new(RefCell::new(Tab { url: String::from("https://rust-lang.org") }));
///
/// let mut browser = Browser {
///     tabs: vec![first_tab.clone(), second_tab.clone()],
///     current_tab: first_tab.clone(),
/// };
///
/// assert_eq!(browser.tabs.len(), 2);
/// assert_eq!(browser.current_tab.borrow().url, "https://example.com");
/// ```
pub struct Browser {
    pub(crate) tabs: Vec<Arc<RwLock<Tab>>>,
    current_tab: Option<Arc<RwLock<Tab>>>,
    chrome: Rc<RefCell<Chrome>>,
    focus: Option<String>,
    pub(crate) cookie_jar: Arc<RwLock<HashMap<String, (String, HashMap<String, String>)>>>,
    pub(crate) measure: Arc<std::sync::Mutex<MeasureTime>>,
    active_tab_scroll: f32,
}

impl Browser {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Arc<RwLock<Self>> {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Self::setup_custom_fonts(&cc.egui_ctx);
        let cookie_jar = Arc::new(RwLock::new(HashMap::new()));
        //let tab = Tab::new(&cc.egui_ctx, 0.0, cookie_jar.clone(), None);
        let browser_obj = Browser { tabs: vec![], current_tab: None,
            chrome: Rc::new(RefCell::new(Chrome::new())),
                focus: None,
                cookie_jar: cookie_jar.clone(),
                measure: Arc::new(std::sync::Mutex::new(MeasureTime::new())),
                active_tab_scroll: 0.0
            };
        let browser = Arc::new(RwLock::new(browser_obj));

        // 2. Now update Chrome with a real weak pointer to the browser
        let chrome = Rc::new(RefCell::new(Chrome::new()));


        browser.write().unwrap().chrome = chrome.clone();
        Browser::new_tab(browser.clone(), &cc.egui_ctx,   Url::new("https://browser.engineering").unwrap());
        browser.clone()
    } 

    pub fn new_tab(this: Arc<RwLock<Self>>, cc: &Context, url: Url)
    {
        let mut browser = this.write().unwrap();
        browser.new_tab_internal(cc, url);
    }

    pub fn new_tab_internal(&mut self, cc: &Context, url: Url)
    {
        let tab = Tab::new(cc, HEIGHT - self.chrome.borrow().bottom(), self.cookie_jar.clone(), Some(self.measure.clone()));
        Tab::send_message(tab.clone(), TabMessage::Load { url, body: None });
        self.tabs.push(tab.clone());
        self.current_tab = Some(tab.clone());
    }

    pub fn load_first_tab(&mut self, url: Url) {
        Tab::send_message(self.tabs[0].clone(), TabMessage::Load { url, body: None });
        self.tabs[0].write().unwrap().tab_height = HEIGHT - self.chrome.borrow().bottom();
    }

    fn setup_custom_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "droid-sans-fallback".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!("../assets/DroidSansFallbackFull.ttf"))),
        );

        fonts.font_data.insert(
            "sans".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!("../assets/Roboto-Regular.ttf")))
        );

        fonts.font_data.insert(
            "sansitalic".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!("../assets/Roboto-Italic.ttf")))
        );

        fonts.font_data.insert(
            "sansbolditalic".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!("../assets/Roboto-BoldItalic.ttf")))
        );

        fonts.font_data.insert(
            "sansbold".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!("../assets/Roboto-Bold.ttf")))
        );

        fonts.families.insert(
            egui::FontFamily::Name("sansbold".into()),
            vec!["sansbold".to_owned()],
        );
        fonts.families.insert(
            egui::FontFamily::Name("sansitalic".into()),
            vec!["sansitalic".to_owned()],
        );
        fonts.families.insert(
            egui::FontFamily::Name("sans".into()),
            vec!["sans".to_owned()],
        );
        fonts.families.insert(
            egui::FontFamily::Name("sansbolditalic".into()),
            vec!["sansbolditalic".to_owned()],
        );




        // Append to existing families to serve as a fallback
        fonts.families.get_mut(&egui::FontFamily::Name("sans".into()))
            .unwrap()
            .push("droid-sans-fallback".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Name("sansbold".into()))
            .unwrap()
            .push("droid-sans-fallback".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Name("sansitalic".into()))
            .unwrap()
            .push("droid-sans-fallback".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Name("sansbolditalic".into()))
            .unwrap()
            .push("droid-sans-fallback".to_owned());

        ctx.set_fonts(fonts);
    }

    fn draw_on_screen(painter: &Painter, scroll_y: f32, cmd: &DrawCommand) {
        match cmd {
            DrawCommand::DrawText(text) => {
                painter.galley(
                    Pos2::new(text.x, text.y - scroll_y),
                    text.galley.clone(),
                    Color32::BLACK,
                );
            }
            DrawCommand::DrawRect(rect) => {
                painter.rect(
                    rect.rect.translate(Vec2::new(0.0, -scroll_y)),
                    rect.radius,
                    rect.color,
                    Stroke::new(0.0, Color32::BLACK),
                    StrokeKind::Middle,
                );
            },
            DrawCommand::DrawOutline(outline) => {
                painter.rect(
                    outline.rect,
                    0.0,
                    Color32::TRANSPARENT,
                    Stroke::new(outline.thickness, outline.color),
                    StrokeKind::Middle
                );
            }
            DrawCommand::DrawLine(line) => {
                painter.line(
                    vec![
                        Pos2::new(line.from.x, line.from.y - scroll_y),
                        Pos2::new(line.to.x, line.to.y - scroll_y)
                    ],
                    Stroke::new(line.thickness, line.color)
                );
            }
        }
    }

    fn clamp_scroll(&mut self) {
        if let Some(tab) = &self.current_tab {
            let tab_read = tab.read().unwrap();
            let height = tab_read.tab_height - self.chrome.borrow().bottom();
            let max_scroll = height;
            print!("max_scroll: {}, active_tab_scroll: {}\n", max_scroll, self.active_tab_scroll);
            if self.active_tab_scroll < 0.0 {
                self.active_tab_scroll = 0.0;
            } else if self.active_tab_scroll > max_scroll {
                self.active_tab_scroll = max_scroll;
            }
        }
    }
    
    fn raster(&mut self, ui: &mut Ui) {
        self.measure.lock().unwrap().time("raster", thread::current().id());
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::WHITE))
            .show_inside(ui, |ui| {
                let painter = ui.painter();
                if let Some(tab) = &self.current_tab {
                    let tab_read = tab.read().unwrap();
                    let scroll_y = self.active_tab_scroll;
                    for cmd in &*tab_read.draw_commands {
                        if (cmd.top() < scroll_y + HEIGHT) || (cmd.bottom() > self.active_tab_scroll) {
                            Self::draw_on_screen(painter, self.active_tab_scroll - self.chrome.borrow().bottom(), &cmd);
                        }
                    }
                }
                let chrome = self.chrome.borrow();
    
                for cmd in &*chrome.draw_commands
                {
                    Self::draw_on_screen(painter, 0.0, &cmd);
                }
            });
        self.measure.lock().unwrap().stop("raster", thread::current().id());
    }

}

impl eframe::App for Browser {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        self.chrome.borrow_mut().init(ui.ctx());
        self.chrome.borrow_mut().draw(ui.ctx(), &*self.tabs, self.current_tab.as_ref());

        if let Some(tab) = self.current_tab.clone() {
            let mut has_raf = false;
            {
                let mut tab_write = tab.write().unwrap();
                if tab_write.has_raf_request {
                    tab_write.has_raf_request = false;
                    has_raf = true;
                }
            }
            if has_raf {
                Tab::send_message(tab.clone(), TabMessage::AnimationFrame);
            }

            if tab.read().unwrap().needs_redraw {
                ui.ctx().request_repaint();
            }

            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                print!("ArrowDown pressed\n");
                Tab::send_message(tab.clone(), TabMessage::ScrollDown);
                self.active_tab_scroll += 20.0;
                self.clamp_scroll();
                self.raster(ui);
                
            }

            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.chrome.borrow_mut().on_enter(tab.clone());
                
            }
        }

        if ui.input(|i| i.pointer.primary_clicked()) {
            let pos = ui.input(|i| i.pointer.interact_pos()).unwrap();

            if pos.y < self.chrome.borrow().bottom() {
                self.focus = None;
                // 1. Get the action and drop the chrome borrow immediately
                let action = self.chrome.borrow_mut().click(ui, pos, self.tabs.len());

                // 2. Now handle the action. 'self' is no longer borrowed!
                if let Some(action) = action {
                    match action {
                        ChromeAction::NewTab => {
                            let ctx = ui.ctx().clone();
                            self.new_tab_internal(&ctx, Url::new("https://browser.engineering").unwrap());
                        }
                        ChromeAction::SelectTab(index) => {
                            self.current_tab = Some(self.tabs[index].clone());
                        }
                        ChromeAction::GoBack => {
                            if let Some(tab) = &self.current_tab {
                                Tab::send_message(tab.clone(), TabMessage::GoBack);
                            }
                           
                        }
                    }
                }
            }  else {
                self.focus = Some("content".parse().unwrap());
                self.chrome.borrow_mut().blur();
                if let Some(tab) = &self.current_tab {
                    Tab::send_message(tab.clone(), TabMessage::Click { position: pos - Vec2::new(0.0, self.chrome.borrow().bottom()) });
                }
            }
        }

        if ui.input(|i| i.viewport().close_requested()){
            self.measure.lock().unwrap().finish();
        }

        let chrome = self.chrome.clone();
        let current_tab = self.current_tab.clone();
        let focus = self.focus.clone();
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    chrome.borrow_mut().keypress(text);
                    if focus == Some("content".parse().unwrap()) {
                        if let Some(tab) = &current_tab {
                            Tab::send_message(tab.clone(), TabMessage::KeyPress { text: text.clone() });
                        }
                    }

                }
            }
        });

        self.raster(ui);
    }




}