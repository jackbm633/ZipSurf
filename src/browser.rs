use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::layout::{LayoutNode, HEIGHT, VSTEP};
use crate::node::{HtmlNodeType, HtmlNode};
use crate::url::Url;
use eframe::egui;
use egui::{Color32, Galley, Pos2, Rect, Stroke};
use std::sync::Arc;
use eframe::epaint::StrokeKind;
use crate::css_parser::CssParser;
use crate::html_parser::HtmlParser;
use crate::selector::Selector;

/// A constant that holds the default stylesheet for a browser application.
///
/// # Description
/// The `DEFAULT_STYLE_SHEET` constant is a string literal that contains the
/// contents of the "browser.css" stylesheet file located in the `../assets` directory.
/// It is embedded into the binary at compile time using the `include_str!` macro.
///
/// # Usage
/// This constant can be used as the initial or fallback stylesheet for a
/// browser-like application to ensure a consistent default appearance.
///
/// # Example
/// ```rust
/// fn main() {
///     println!("Default stylesheet: {}", DEFAULT_STYLE_SHEET);
/// }
/// ```
///
/// # File Location
/// The "browser.css" file must exist in the specified relative path (`../assets`) at compile time.
/// If the file is not found, the compilation will fail.
///
/// # Notes
/// - Ensure that the "browser.css" file contains valid CSS content.
/// - Changes to the referenced file will require recompilation to reflect updates in the binary.
const DEFAULT_STYLE_SHEET: &str = include_str!("../assets/browser.css");

/// The primary state controller for the web browser engine.
///
/// This struct manages the lifecycle of web content from initial URL fetching
/// through HTML sanitization and final 2D layout. It maintains a persistent
/// reference to the `egui::Context` to perform font metric calculations and
/// handles the application's scroll state.
pub struct Browser {
    /// A collection of `Token` objects.
    ///
    /// This vector stores instances of the `Token` type, which represent
    /// individual elements or symbols in a parsed input (e.g., programming
    /// language keywords, operators, or other lexemes). The `tokens` vector
    /// can be used in scenarios such as tokenizing source code, analyzing
    /// input streams, or implementing interpreters/compilers.
    ///
    /// # Example
    /// ```
    /// let tokens: Vec<Token> = Vec::new();
    /// // Add tokens to the vector as needed
    /// // tokens.push(Token::new(...));
    /// ```
    ///
    /// # Usage
    /// - Maintain the sequential order of tokens for parsing tasks.
    /// - Perform operations like iteration, filtering, or mapping on the list of tokens.
    tokens: Vec<HtmlNodeType>,
    draw_commands: Vec<DrawCommand>,
    /// The current vertical scroll offset in points.
    scroll_y: f32,
    /// Handle to the egui context for font layout and UI state.
    context: egui::Context,
    /// The raw, sanitized text content extracted from the source HTML.
    body: String,
    nodes: Option<Rc<RefCell<HtmlNode>>>,
    document: Option<Rc<RefCell<LayoutNode>>>
}



const SCROLL_STEP: f32 = 100.0;

impl Default for Browser {
    /// Returns a `Browser` instance with empty buffers and default scroll position.
    ///
    /// Note: The `context` is initialized with a default handle which should be
    /// overwritten during `new()` to ensure it points to the active UI context.
    fn default() -> Self {
        Browser {
            tokens: Vec::new(),
            draw_commands: Vec::new(),
            scroll_y: 0.0,
            context: egui::Context::default(),
            body: String::new(),
            nodes: None,
            document: None
        }
    }
}

impl Browser {
    /// Initializes a new browser instance and configures the UI environment.
    ///
    /// Sets the global visual theme to light mode to ensure text contrast and
    /// registers custom fonts required for international character support.
    ///
    /// # Arguments
    /// * `cc` - Integration context providing access to the egui render state.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Self::setup_custom_fonts(&cc.egui_ctx);

        Self {
            context: cc.egui_ctx.clone(),
            ..Default::default()
        }
    }

    /// Fetches a web page, strips HTML tags, and stores the raw content.
    ///
    /// This triggers a blocking network request. Upon success, the response
    /// body is passed through a lexer to remove markup before being cached
    /// in `self.body`.
    ///
    /// # Errors
    /// Network failures or request timeouts are logged to `stderr`.
    pub fn load(&mut self, url: Url) {
        match url.request() {
            Ok(body) => {
                let mut parser = HtmlParser {
                    body: body.clone(),
                    unfinished: vec![]
                };

                self.nodes =  Some(parser.parse());
                Self::style(Some(self.nodes.clone().unwrap()), vec![]);
                self.document = Some(LayoutNode::new_document(self.nodes.clone().unwrap()));
            }
            Err(e) => {
                eprintln!("Error loading URL: {}", e);
            }
        }
    }

    /// Applies CSS styling rules to an HTML node and its descendants.
    ///
    /// This function traverses the structure of an HTML document, starting from the given node.
    /// It matches CSS selectors with elements, applies the corresponding styles, and resolves
    /// inline styles if defined in the `style` attribute of elements. The styles are stored in
    /// the `style` attribute of each `HtmlNode` object.
    ///
    /// # Parameters
    /// - `node`: An optional reference-counted pointer (`Rc`) to a mutable `HtmlNode`, which is
    ///           the starting point of the HTML tree where the styles will be applied. If `None`,
    ///           the function panics because the browser document is not initialized.
    /// - `rules`: A vector of CSS rules, where each rule is represented as a tuple containing:
    ///   - `Selector`: A CSS selector that determines which elements the rule applies to.
    ///   - `HashMap<String, String>`: A map of CSS property names and their associated values.
    ///
    /// # Behavior
    /// 1. Checks if the `node` is `None`. If it is `None`, the function panics with the message:
    ///    "Browser document not initialized."
    /// 2. If the `node` is present:
    ///    - If the node is an element (`HtmlNodeType::Element`):
    ///      - Iterates over the provided `rules`.
    ///      - Matches the `Selector` against the current `node`. If the selector doesn't match,
    ///        the rule is skipped.
    ///      - For matching selectors, updates the node's `style` by adding CSS properties and
    ///        their values from the rule to the `style` attribute of the node.
    ///      - Resolves and parses any inline styles defined in the `style` attribute of the
    ///        element, applying them to the node's `style`.
    ///    - If the node is a text node (`HtmlNodeType::Text`), it skips processing as styles
    ///      are not applicable to text nodes.
    /// 3. Recursively applies styles to all child nodes of the current node.
    ///
    /// # Panics
    /// - If the `node` is `None`, indicating an uninitialized document.
    /// - If the inline `style` attribute of an element fails to parse using the `CssParser`,
    ///   the function will panic without a specific error message.
    ///
    /// # Examples
    /// ```
    /// // Example usage:
    /// let root_node = Some(Rc::new(RefCell::new(HtmlNode::new(HtmlNodeType::Element(
    ///     ElementData::new("div".to_string(), HashMap::new())
    /// )))));
    /// let rules = vec![
    ///     (Selector::new("div".to_string()), HashMap::from([("color".to_string(), "red".to_string())]))
    /// ];
    /// style(root_node, rules);
    /// ```
    ///
    /// In this example, the `div` element will have its `color` property set to "red".
    ///
    /// # Notes
    /// - This function assumes that `Selector::matches` is defined and correctly matches selectors
    ///   to elements.
    /// - The expected behavior of `CssParser::new` and its `body` method is to parse inline styles and
    ///   resolve them into a `HashMap<String, String>`. Any deviations from this expected behavior may
    ///   cause a panic.
    fn style(node: Option<Rc<RefCell<HtmlNode>>>, rules: Vec<(Selector, HashMap<String, String>)>) {
        match node {
            None => {panic!("Browser document not initialized.")}
            Some(nd) => {
                let mut borrow_node = nd.borrow_mut();
                match &borrow_node.node_type {
                    HtmlNodeType::Element(el) => {
                        for (selector, rules) in rules {
                            if !selector.matches(nd.clone()) {
                                continue
                            }
                            for (property, value) in rules.iter() {
                                nd.borrow_mut().style.insert(property.to_string(), value.to_string());
                            }
                        }

                        if el.attributes.contains_key("style") {
                            let mut parser = CssParser::new(el.attributes["style"].as_str());
                            let pairs = parser.body();
                            match pairs {
                                Ok(pairs_map) => {
                                    for (key, value) in pairs_map {
                                        borrow_node.style.insert(
                                            key.to_string(), value.to_string());
                                    }
                                }
                                Err(_) => {panic!()}
                            }
                        }
                    }
                    HtmlNodeType::Text(_) => {}
                }

                for child in borrow_node.children.clone() {
                    Self::style(Some(child), vec![])
                }
            }
        }
    }
    

    /// Configures and sets up custom fonts for the `egui` UI context.
    ///
    /// This function overrides the default font definitions and adds multiple custom fonts
    /// to the `egui` context. The fonts are loaded from static assets and associated with
    /// specific names for usage within the application. Additionally, custom font families
    /// are configured to define fallbacks when rendering text.
    ///
    /// # Arguments
    ///
    /// * `ctx` - A mutable reference to the `egui::Context`, which manages the UI state and
    ///           allows setting custom fonts.
    ///
    /// # Details
    ///
    /// 1. **Custom Font Data Insertion:**
    ///    - Fonts such as "DroidSansFallbackFull.ttf", "Roboto-Regular.ttf",
    ///      "Roboto-Italic.ttf", "Roboto-BoldItalic.ttf", and "Roboto-Bold.ttf"
    ///      are loaded via the `include_bytes!` macro and inserted into the font
    ///      definitions using unique names like "droid-sans-fallback", "sans", "sansitalic",
    ///      "sansbold", and "sansbolditalic".
    ///
    /// 2. **Font Family Assignments:**
    ///    - The loaded fonts are organized into custom font families (`sans`, `sansitalic`,
    ///      `sansbold`, and `sansbolditalic`) for specialized usage.
    ///    - Each family holds a list of corresponding font names for rendering text.
    ///
    /// 3. **Fallback Configuration:**
    ///    - The `droid-sans-fallback` font is appended as a fallback font to each of the
    ///      custom font families, ensuring proper glyph rendering if the primary font lacks
    ///      support for certain characters.
    ///
    /// 4. **Apply the Fonts:**
    ///    - The custom font definitions are applied to the provided `egui::Context`
    ///      using the `set_fonts` method.
    ///
    /// # Example Usage
    ///
    /// ```rust
    /// use egui::Context;
    ///
    /// fn main() {
    ///     let ctx = egui::Context::default();
    ///     setup_custom_fonts(&ctx);
    ///     // Now the UI will render using the custom fonts defined in this function
    /// }
    /// ```
    ///
    /// # Dependencies
    /// Requires font assets to be present in the `../assets/` directory relative to the
    /// source file, as specified in the `include_bytes!` macro.
    ///
    /// # Notes
    /// This function should be called during the initialization or setup phase of the
    /// application to ensure the custom fonts are used throughout the UI.
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
        if self.draw_commands.is_empty() {
            match self.document.as_mut() {
                None => { panic!("Browser document not initialized.") },
                Some(doc) => {
                    LayoutNode::layout(doc.clone(), ctx.clone());
                    self.draw_commands = vec![];
                    LayoutNode::paint_tree(doc.clone(), &mut self.draw_commands);

                }
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)){
            match &self.document {
                None => { panic!("Browser document not initialized.") },
                Some(doc) => {
                    let max_y = doc.borrow().size.unwrap().y + 2.0*VSTEP - HEIGHT;
                    self.scroll_y = (self.scroll_y + SCROLL_STEP).min(max_y);
                }
            }
            self.scroll_y += SCROLL_STEP;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            for text in &self.draw_commands {
                if (text.top() < self.scroll_y + HEIGHT) || (text.bottom() > self.scroll_y) {
                    match text {
                        DrawCommand::DrawText(text) => {
                            painter.galley(
                                Pos2::new(text.x, text.y - self.scroll_y),
                                text.galley.clone(),
                                Color32::BLACK,
                            );
                        }
                        DrawCommand::DrawRect(rect) => {
                            painter.rect(Rect::from_two_pos(
                                (rect.top_left - Pos2::new(0.0, self.scroll_y)).to_pos2(),
                                (rect.bottom_right - Pos2::new(0.0, self.scroll_y)).to_pos2()),
                                         0, rect.color,
                                         Stroke::new(0.0, Color32::BLACK),
                                         StrokeKind::Middle);
                        }
                    }
                }

                // Simple culling: don't draw text that is off-screen

            }
        });
    }
}

/// A structure that represents text rendering properties, including its content,
/// position, and font details.
///
/// # Fields
/// - `content`:
///   The textual content to be rendered, such as a word or character.
/// - `x`:
///   The absolute horizontal position of the text in points.
/// - `y`:
///   The absolute vertical position of the text in points.
/// - `font_name`:
///   The name of the font family to be used for rendering the text.
#[derive(Clone, Debug)]
pub(crate) struct DrawText {
    /// Absolute horizontal position in points.
    pub(crate) x: f32,
    /// Absolute vertical position in points.
    pub(crate) y: f32,
    /// Galley for drawing the text.
    pub(crate) galley: Arc<Galley>
}

pub(crate) struct DrawRect {
    /// Represents the top-left position of a rectangle or a bounding box.
    ///
    /// # Fields
    /// - `top_left`: A `Pos2` struct that defines the coordinates of the
    /// top-left corner. It typically contains `x` and `y` values, denoting
    /// the horizontal and vertical positions in 2D space, respectively.
    ///
    /// # Example
    /// ```
    /// let rect_top_left = Pos2 { x: 0.0, y: 0.0 };
    /// let my_rectangle = Rectangle { top_left: rect_top_left };
    /// ```
    pub top_left: Pos2,
    /// Represents the bottom-right corner of a rectangle or bounding box.
    ///
    /// `bottom_right` is a field of type `Pos2` that defines the position
    /// of the bottom-right corner in a 2D coordinate space. This is
    /// typically used to describe geometric boundaries or the dimensions
    /// of a rectangular area.
    ///
    /// # Example
    /// ```rust
    /// let rect_bottom_right = Pos2 { x: 10.0, y: 5.0 };
    /// println!("Bottom-right corner is at: ({}, {})", rect_bottom_right.x, rect_bottom_right.y);
    /// ```
    ///
    /// # Fields
    /// - `bottom_right`: A `Pos2` struct containing `x` and `y` coordinates.
    ///
    /// # See also
    /// - `Pos2` for understanding the structure of the coordinate type.
    pub bottom_right: Pos2,
    /// A public field representing the color of the object.
    ///
    /// This field uses the `Color32` type, which encapsulates a 32-bit color value,
    /// typically including red, green, blue, and alpha (transparency) components.
    ///
    /// # Example
    /// ```rust
    /// use some_module::Color32;
    ///
    /// struct Object {
    ///     pub color: Color32,
    /// }
    ///
    /// let my_color = Color32::from_rgba_unmultiplied(255, 0, 0, 255); // Red color
    /// let object = Object { color: my_color };
    /// println!("The color is {:?}", object.color);
    /// ```
    ///
    /// # Usage
    /// This field can be read or modified directly in structs where it is declared as `pub`.
    pub color: Color32
}

pub enum DrawCommand {
    DrawText(DrawText),
    DrawRect(DrawRect)
}

impl DrawCommand {
    /// Calculates the bottom y-coordinate of a `DrawCommand`.
    ///
    /// This method determines the vertical bottom position based on the type of
    /// `DrawCommand`:
    ///
    /// - If the `DrawCommand` is `DrawText`, the bottom is computed as the sum of
    ///   the `y` coordinate of the text and the height of its galley (text layout).
    /// - If the `DrawCommand` is `DrawRect`, the bottom is the `y` coordinate of
    ///   the rectangle's bottom-right corner.
    ///
    /// # Returns
    ///
    /// A `f32` value representing the bottom y-coordinate of the `DrawCommand`.
    ///
    /// # Examples
    /// ```
    /// let text_command = DrawCommand::DrawText(Text {
    ///     y: 10.0,
    ///     galley: Galley { rect: Rect { height: 20.0 } },
    /// });
    /// assert_eq!(text_command.bottom(), 30.0);
    ///
    /// let rect_command = DrawCommand::DrawRect(Rect {
    ///     bottom_right: Point { y: 25.0 },
    /// });
    /// assert_eq!(rect_command.bottom(), 25.0);
    /// ```
    ///
    /// # Panics
    /// This function does not explicitly panic under normal circumstances, provided
    /// valid `DrawCommand` variants are used.
    fn bottom(&self) -> f32 {
        match self {
            DrawCommand::DrawText(txt) => {
                txt.y + txt.galley.rect.height()
            }
            DrawCommand::DrawRect(rct) => {
                rct.bottom_right.y
            }
        }
    }

    /// ```rust
    /// Returns the `y` coordinate of the top position for the current `DrawCommand`.
    ///
    /// This method determines the top position based on the specific variant
    /// of the `DrawCommand` enum:
    ///
    /// - If the `DrawCommand` is a `DrawText`, it returns the `y` coordinate of the text's position.
    /// - If the `DrawCommand` is a `DrawRect`, it returns the `y` coordinate of the rectangle's top-left corner.
    ///
    /// # Returns
    /// A `f32` representing the `y` coordinate of the top position.
    ///
    /// # Example
    /// ```rust
    /// let text_command = DrawCommand::DrawText(Text { y: 10.0 });
    /// assert_eq!(text_command.top(), 10.0);
    ///
    /// let rect_command = DrawCommand::DrawRect(Rect { top_left: Point { x: 5.0, y: 20.0 } });
    /// assert_eq!(rect_command.top(), 20.0);
    /// ```
    /// ```
    fn top(&self) -> f32 {
        match self {
            DrawCommand::DrawText(txt) => {
                txt.y
            }
            DrawCommand::DrawRect(rct) => {
                rct.top_left.y
            }
        }
    }
}