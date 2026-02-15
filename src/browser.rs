use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use crate::tab::{Tab};
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
    tabs: Vec<Rc<RefCell<Tab>>>,
    current_tab: Rc<RefCell<Tab>>
}

impl Browser {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Self::setup_custom_fonts(&cc.egui_ctx);

        let tab = Rc::new(RefCell::new(Tab::new(cc)));

        Browser { tabs: vec![tab.clone()], current_tab: tab.clone()}
    }

    pub fn load_first_tab(&mut self, url: Url) {
        self.tabs[0].borrow_mut().load(url);
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
}

impl eframe::App for Browser {
    /// Updates the application state and renders the user interface.
    ///
    /// This function is invoked during every frame update loop. It handles user input, updates internal
    /// state data, and renders text elements onto the screen.
    ///
    /// # Parameters
    /// - `ctx`: A reference to the `egui::Context` object, which provides access to the current user interface context.
    /// - `_frame`: A mutable reference to the `eframe::Frame` object, representing the application frame (not currently used).
    ///
    /// # Description
    /// - If `self.texts` is empty, it attempts to initialize it from `self.document`. The document is laid out, and the `texts`
    ///   field of the document is cloned into `self.texts`.
    /// - Responds to user input:
    ///   - If the user presses the "ArrowDown" key, the `self.scroll_y` value increases by the predefined `SCROLL_STEP`,
    ///     effectively scrolling the view down.
    /// - Renders text elements:
    ///   - The function uses `egui::CentralPanel` to define the main rendering area.
    ///   - It iterates over all `self.texts` and checks if each text element is in the visible scroll region.
    ///   - If the text element is within the visible region, it is drawn using the `painter.galley()` method at the appropriate
    ///     position and with a black color (`Color32::BLACK`).
    ///
    /// # Panics
    /// - If `self.document` is not initialized (`None`) when it is required to fetch text elements, the function will panic
    ///   with the message: "Browser document not initialized."
    ///
    /// # Notes
    /// - The `scroll_y` value helps manage vertical scrolling within the rendering area.
    /// - Text elements are culled (skipped) if they are outside the currently visible vertical region.
    ///
    /// # Example Usage
    /// ```rust
    /// app.update(ctx, frame);
    /// ```
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.current_tab.borrow_mut().draw(ctx, _frame);

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)){
           self.current_tab.borrow_mut().scroll_down();
        }

        if ctx.input(|i| i.pointer.primary_clicked())
        {
            let pos = ctx.input(|i| i.pointer.interact_pos()).unwrap();
            self.current_tab.borrow_mut().click(pos)
        }
    }
}