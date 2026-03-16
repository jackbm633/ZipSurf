use std::cell::{RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;
use eframe::emath::{Pos2, Rect};
use eframe::epaint::{Color32, Stroke, StrokeKind};
use egui::{Context, Modifiers, Painter, Vec2};
use crate::chrome::{Chrome, ChromeAction};
use crate::layout::HEIGHT;
use crate::tab::{DrawCommand, Tab};
use crate::url::Url;

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
    pub(crate) tabs: Vec<Rc<RefCell<Tab>>>,
    current_tab: Rc<RefCell<Tab>>,
    chrome: Rc<RefCell<Chrome>>,
    focus: Option<String>
}

impl Browser {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Rc<RefCell<Self>> {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Self::setup_custom_fonts(&cc.egui_ctx);

        let tab = Rc::new(RefCell::new(Tab::new(&cc.egui_ctx, 0.0)));
        let browser = Rc::new(RefCell::new(
            Browser { tabs: vec![tab.clone()], current_tab: tab.clone(),
            chrome: Rc::new(RefCell::new(Chrome::new(Weak::new(), &cc.egui_ctx))),
                focus: None,
            }));

        // 2. Now update Chrome with a real weak pointer to the browser
        let chrome = Rc::new(RefCell::new(Chrome::new(Rc::downgrade(&browser), &cc.egui_ctx)));


        browser.borrow_mut().chrome = chrome.clone();
        browser


    }

    pub fn new_tab(&mut self, cc: &Context, url: Url)
    {
        let tab = Rc::new(RefCell::new(Tab::new(cc, HEIGHT - self.chrome.borrow().bottom())));
        tab.borrow_mut().load(url);
        self.tabs.push(tab.clone());
        self.current_tab = tab.clone();
    }

    pub fn load_first_tab(&mut self, url: Url) {
        self.tabs[0].borrow_mut().load(url);
        self.tabs[0].borrow_mut().tab_height = HEIGHT - self.chrome.borrow().bottom();
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
                    0.0,
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
                    vec![line.from, line.to],
                    Stroke::new(line.thickness, line.color)
                );
            }
        }
    }
}

impl eframe::App for Browser {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.chrome.borrow_mut().init(ctx);

        {
            let mut tab = self.current_tab.borrow_mut();
            tab.update_layout(ctx);
        }
        self.chrome.borrow_mut().draw(ctx, &*self.tabs, &self.current_tab);

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.current_tab.borrow_mut().scroll_down();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.chrome.borrow_mut().on_enter(self.current_tab.borrow_mut());
        }


        if ctx.input(|i| i.pointer.primary_clicked()) {
            let pos = ctx.input(|i| i.pointer.interact_pos()).unwrap();

            if pos.y < self.chrome.borrow().bottom() {
                self.focus = None;
                // 1. Get the action and drop the chrome borrow immediately
                let action = self.chrome.borrow_mut().click(ctx, pos, self.tabs.len());
                
                // 2. Now handle the action. 'self' is no longer borrowed!
                if let Some(action) = action {
                    match action {
                        ChromeAction::NewTab => {
                            self.new_tab(ctx, Url::new("https://browser.engineering").unwrap());
                        }
                        ChromeAction::SelectTab(index) => {
                            self.current_tab = self.tabs[index].clone();
                        }
                        ChromeAction::GoBack => {
                            self.current_tab.borrow_mut().go_back();
                        }
                    }
                }
            }  else {
                self.focus = Some("content".parse().unwrap());
                self.chrome.borrow_mut().blur();
                let mut tab = self.current_tab.borrow_mut();
                tab.click((pos - Vec2::new(0.0, self.chrome.borrow().bottom())));
            }
        }

        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    self.chrome.borrow_mut().keypress(text);
                }
            }
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::WHITE))
            .show(ctx, |ui| {
                let painter = ui.painter();
                let tab = self.current_tab.borrow();
                let scroll_y = tab.scroll_y;
                let chrome = self.chrome.borrow();



                for cmd in &*tab.draw_commands {
                    if (cmd.top() < scroll_y + HEIGHT) || (cmd.bottom() > scroll_y) {
                        Self::draw_on_screen(painter, scroll_y - self.chrome.borrow().bottom(), &cmd);
                    }
                }

                for cmd in &*chrome.draw_commands
                {
                    Self::draw_on_screen(painter, 0.0, &cmd);
                }
            });
    }
}