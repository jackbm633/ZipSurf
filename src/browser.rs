use std::sync::Arc;
use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2};
use crate::node::{Token, Text, Tag};
use crate::url::Url;

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
    tokens: Vec<Token>,
    /// A collection of positioned text elements ready for rendering.
    texts: Vec<DrawText>,
    /// The current vertical scroll offset in points.
    scroll_y: f32,
    /// Handle to the egui context for font layout and UI state.
    context: egui::Context,
    /// The raw, sanitized text content extracted from the source HTML.
    body: String,
}

const HSTEP: f32 = 13.0;
const VSTEP: f32 = 17.0;
const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;
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
                self.tokens = Browser::lex(&body);
            }
            Err(e) => {
                eprintln!("Error loading URL: {}", e);
            }
        }
    }

    /// Lays out text tokens in a two-dimensional space while adhering to certain layout constraints,
    /// such as maximum width and step definitions. This method is responsible for positioning words
    /// and handling line breaks to ensure proper alignment and spacing.
    ///
    /// # Description
    /// - The layout function iterates over a collection of tokens (`self.tokens`) and calculates the
    ///   x (`cursor_x`) and y (`cursor_y`) positions for each word in the provided text tokens.
    /// - If a word cannot fit within the remaining width of the current line (determined by `WIDTH - HSTEP`),
    ///   it moves the cursor to a new line with additional vertical spacing (calculated based on the font's row height).
    /// - Words are spaced horizontally with the width of a single blank space (`space_width`).
    /// - Handles only `Token::Text` tokens, ignoring `Token::Tag` entries.
    ///
    /// # Parameters
    /// - `cursor_x`: Tracks the horizontal position for text layout.
    /// - `cursor_y`: Tracks the vertical position for text layout.
    /// - `tokens`: Collection of tokens to be rendered (either `Token::Text` or `Token::Tag`).
    /// - `font_id`: Specifies the type and size of the font used for measuring and rendering text.
    /// - `space_width`: Precomputed width of a single space character for consistent spacing between words.
    ///
    /// # Output
    /// - Words' positions are stored in `self.texts` as `DrawText` instances, which include the content of the word and
    ///   its calculated `x` and `y` coordinates.
    ///
    /// # Constraints
    /// - Words are wrapped to a new line if the current `cursor_x` plus the word's width exceeds `WIDTH - HSTEP`.
    /// - Line spacing is increased by `1.25 * row_height` of the font whenever a word is wrapped to the next line.
    ///
    /// # Dependencies
    /// - Relies on `self.context.fonts_mut` for access to the font system, which measures text dimensions
    ///   and provides font-related utilities.
    /// - Requires `Token`, `DrawText`, and `FontId` structures and `Color32::BLACK` for layout customization.
    ///
    /// # Example Use Case
    /// Imagine an editor where a user inputs text. This method arranges the text into lines while respecting
    /// boundaries and spacing:
    /// - Words are wrapped appropriately to the next line when they exceed the allocated width.
    /// - Each word is positioned and stored in `self.texts` for rendering.
    ///
    /// # Notes
    /// - Ensure that the `WIDTH`, `HSTEP`, and `VSTEP` constants are appropriately defined in the enclosing scope.
    /// - The function currently ignores tokens of type `Token::Tag`. Future extensions could include support
    ///   for rendering or processing such tokens.
    fn layout(&mut self) {
        let mut cursor_x = HSTEP;
        let mut cursor_y = VSTEP;
        let tokens = &self.tokens;
        let font_id = FontId::proportional(13.0);

        let space_galley = self.context.fonts_mut(|f|
            f.layout_no_wrap(" ".to_string(), font_id.clone(), Color32::BLACK));
        let space_width = space_galley.size().x;
        for c in tokens {
            match c {
                Token::Tag(_) => {}
                Token::Text(text) => {
                    for word in text.text.split_whitespace() {
                        // Access egui's font engine to measure word dimensions
                        let galley = self.context.fonts_mut(|f|
                            f.layout_no_wrap(word.to_string(), font_id.clone(), Color32::BLACK));

                        let text_width = galley.size().x;

                        if cursor_x + text_width > WIDTH - HSTEP {
                            cursor_y += self.context.fonts_mut(|f|
                                f.row_height(&font_id)) * 1.25;
                            cursor_x = HSTEP;
                        }
                        self.texts.push(DrawText {
                            content: word.to_string(),
                            x: cursor_x,
                            y: cursor_y,
                        });


                        cursor_x += text_width + space_width;
                    }

                }
            }



        }
    }

    /// A stream-based lexer that extracts plain text from HTML/XML markup.
    ///
    /// Iterates through the input character by character, discarding any 
    /// content contained within `<` and `>` delimiters.
    ///
    /// # Example
    /// ```
    /// let html = "<div>Hello</div>";
    /// let plain = Browser::lex(html);
    /// assert_eq!(plain, "Hello");
    /// ```
    pub fn lex(text: &str) -> Vec<Token> {
        let mut output: Vec<Token> = Vec::new();
        let mut buffer = String::new();
        let mut in_tag = false;
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            match c {
                '<' => {
                    in_tag = true;
                    if !buffer.is_empty() {output.push(
                        Token::Text(Text {text: buffer.clone()}))
                    }
                    buffer.clear();
                }
                '>' => {
                    in_tag = false;
                    output.push(Token::Tag(Tag {tag: buffer.clone()}));
                    buffer.clear();
                },
                _ => {
                    buffer.push(c);
                }
            }
        }
        if !in_tag && !buffer.is_empty() {
            output.push(Token::Text(Text {text: buffer.clone()}))
        }
        output
    }

    /// Registers the Droid Sans Fallback font to the egui font manager.
    ///
    /// This ensures that CJK (Chinese, Japanese, Korean) and other non-Latin
    /// characters render correctly. The font is embedded in the binary via 
    /// `include_bytes!`.
    fn setup_custom_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "droid-sans-fallback".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!("../assets/DroidSansFallbackFull.ttf"))),
        );

        // Append to existing families to serve as a fallback
        fonts.families.get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push("droid-sans-fallback".to_owned());

        fonts.families.get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .push("droid-sans-fallback".to_owned());

        ctx.set_fonts(fonts);
    }
}

impl eframe::App for Browser {
    /// The main UI update and rendering loop.
    ///
    /// Handles:
    /// 1. **Lazy Layout**: Triggers `layout()` if the text buffer is empty.
    /// 2. **Input Handling**: Listens for `ArrowDown` to increment scroll.
    /// 3. **Culling & Drawing**: Iterates through `texts`, performing basic 
    ///    frustum culling (checking if text is within the visible height) 
    ///    before drawing to the `Painter`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.texts.is_empty() {
            self.layout();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.scroll_y += SCROLL_STEP;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            for text in &self.texts {
                // Simple culling: don't draw text that is off-screen
                if (text.y > self.scroll_y + HEIGHT) || (text.y + VSTEP < self.scroll_y) {
                    continue;
                }
                
                painter.text(
                    Pos2::new(text.x, text.y - self.scroll_y), 
                    Align2::LEFT_TOP, 
                    &text.content, 
                    FontId::proportional(13.0), 
                    Color32::BLACK
                );
            }
        });
    }
}

/// Represents a single unit of text positioned in 2D space.
struct DrawText {
    /// The string content of the word or character.
    content: String,
    /// Absolute horizontal position in points.
    x: f32,
    /// Absolute vertical position in points.
    y: f32
}