use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::layout::{Layout, HEIGHT};
use crate::node::{Element, Text, HtmlNodeType, HtmlNode};
use crate::url::Url;
use eframe::egui;
use egui::{Color32, Galley, Pos2};
use std::sync::Arc;
use crate::html_parser::HtmlParser;

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
    texts: Vec<DrawText>,
    /// The current vertical scroll offset in points.
    scroll_y: f32,
    /// Handle to the egui context for font layout and UI state.
    context: egui::Context,
    /// The raw, sanitized text content extracted from the source HTML.
    body: String,
    nodes: Option<Rc<RefCell<HtmlNode>>>,
    document: Option<Layout>
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
            texts: Vec::new(),
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
                self.document = Some(Layout::new(self.nodes.clone().unwrap().clone(),
                                                 self.context.clone()));
            }
            Err(e) => {
                eprintln!("Error loading URL: {}", e);
            }
        }
    }

    /// Parses the input string into a sequence of tokens, distinguishing between text and tags.
    ///
    /// # Arguments
    /// * `text` - A string slice that represents the content to be tokenized.
    ///
    /// # Returns
    /// * A `Vec<Token>` containing the extracted tokens. Each token can be either a `Text`
    ///   (for plain text) or a `Tag` (for content enclosed in angle brackets `< >`).
    ///
    /// # Behavior
    /// * The function iterates through the characters of the input string.
    /// * When it encounters a `<`, it interprets the subsequent characters as part of a tag
    ///   until a `>` is found.
    ///   - Any text before `<` is treated as `Text` and added to the output tokens.
    ///   - The characters between `<` and `>` are treated as a `Tag`.
    /// * Text outside of `<` and `>` is treated as `Text`.
    /// * If the end of the input string is reached while not inside a tag, any remaining
    ///   content in the buffer is added as a `Text` token.
    ///
    /// # Example
    /// ```rust
    /// let input = "Hello <tag>world</tag>";
    /// let tokens = lex(input);
    /// assert_eq!(tokens, vec![
    ///     Token::Text(Text { text: "Hello ".to_string() }),
    ///     Token::Tag(Tag { tag: "tag".to_string() }),
    ///     Token::Text(Text { text: "world".to_string() }),
    ///     Token::Tag(Tag { tag: "/tag".to_string() }),
    /// ]);
    /// ```
    ///
    /// # Notes
    /// * This function assumes balanced usage of `<` and `>` in the input string.
    /// * Any text outside `< >` is treated as plain text without further processing.
    pub fn lex(text: &str) -> Vec<HtmlNodeType> {
        let mut output: Vec<HtmlNodeType> = Vec::new();
        let mut buffer = String::new();
        let mut in_tag = false;
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            match c {
                '<' => {
                    in_tag = true;
                    if !buffer.is_empty() {output.push(
                        HtmlNodeType::Text(Text {text: buffer.clone()}))
                    }
                    buffer.clear();
                }
                '>' => {
                    in_tag = false;
                    output.push(HtmlNodeType::Element(Element {tag: buffer.clone(), attributes: HashMap::new()}));
                    buffer.clear();
                },
                _ => {
                    buffer.push(c);
                }
            }
        }
        if !in_tag && !buffer.is_empty() {
            output.push(HtmlNodeType::Text(Text {text: buffer.clone()}))
        }
        output
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
        if self.texts.is_empty() {
            match self.document.as_mut() {
                None => { panic!("Browser document not initialized.") },
                Some(ref mut doc) => {
                    doc.layout();
                    self.texts = doc.texts.clone();
                }
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.scroll_y += SCROLL_STEP;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            for text in &self.texts {
                // Simple culling: don't draw text that is off-screen
                if (text.y > self.scroll_y + HEIGHT) || (text.y + text.galley.size().y < self.scroll_y) {
                    continue;
                }

                painter.galley(
                    Pos2::new(text.x, text.y - self.scroll_y),
                    text.galley.clone(),
                    Color32::BLACK,
                );
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
#[derive(Clone)]
pub(crate) struct DrawText {
    /// Absolute horizontal position in points.
    pub(crate) x: f32,
    /// Absolute vertical position in points.
    pub(crate) y: f32,
    /// Galley for drawing the text.
    pub(crate) galley: Arc<Galley>
}