use std::sync::Arc;
use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2};
use crate::url::Url;

/// The primary state controller for the web browser engine.
///
/// This struct manages the lifecycle of web content from initial URL fetching
/// through HTML sanitization and final 2D layout. It maintains a persistent 
/// reference to the `egui::Context` to perform font metric calculations and
/// handles the application's scroll state.
pub struct Browser {
    /// A collection of positioned text elements ready for rendering.
    texts: Vec<Text>,
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
                self.body = Browser::lex(&body);
            }
            Err(e) => {
                eprintln!("Error loading URL: {}", e);
            }
        }
    }

    /// ```rust
    /// Handles the layout logic for arranging text in a constrained horizontal
    /// space, dynamically wrapping text to the next line if necessary.
    ///
    /// The function involves the following steps:
    /// - Initializes the starting cursor position (`cursor_x` and `cursor_y`)
    ///   with horizontal and vertical step constants (`HSTEP` and `VSTEP`).
    /// - Retrieves the text to arrange (`self.body`).
    /// - Utilizes egui's font engine to measure the dimensions of text and space
    ///   characters, ensuring accurate placement.
    /// - Iteratively breaks the input `text` into whitespace-separated words and
    ///   measures each word's width.
    /// - Adds words to a line until they exceed the available width (`WIDTH`),
    ///   then wraps the text to the next line by adjusting the cursor's vertical
    ///   position (`cursor_y`).
    /// - Pushes each word and its layout position into the `self.texts` collection.
    ///
    /// ### Key Components:
    /// - **Text Measurement**: Uses egui's font layout engine to measure individual
    ///   words and space widths.
    /// - **Line Wrapping**: Ensures words do not exceed the predefined width
    ///   (`WIDTH`), aligning neatly within bounds while maintaining proper spacing.
    /// - **Dynamic Cursor Adjustment**: Updates `cursor_x` and `cursor_y` to
    ///   account for word placement and line wrapping.
    ///
    /// ### Parameters:
    /// This method operates on the following struct fields:
    /// - `self.body` - The text content to layout.
    /// - `self.context` - Used for accessing font-related operations in egui.
    /// - `self.texts` - A mutable list to store the laid-out text segments. Each
    ///   segment includes its content and coordinates.
    ///
    /// ### Output:
    /// - No direct return value. Mutates `self.texts` to store the layout metadata.
    ///
    /// ### Example:
    /// ```rust
    /// // Assuming `layout` is called within a struct that properly initializes
    /// // `self.body`, `self.context`, and `self.texts`, no additional setup is required.
    /// self.layout();
    /// ```
    /// ```
    fn layout(&mut self) {
        let mut cursor_x = HSTEP;
        let mut cursor_y = VSTEP;
        let text = &self.body;
        let font_id = FontId::proportional(13.0);

        let space_galley = self.context.fonts_mut(|f|
            f.layout_no_wrap(" ".to_string(), font_id.clone(), Color32::BLACK));
        let space_width = space_galley.size().x;
        for c in text.split_whitespace() {

            // Access egui's font engine to measure word dimensions
            let galley = self.context.fonts_mut(|f| 
                f.layout_no_wrap(c.to_string(), font_id.clone(), Color32::BLACK));

            let text_width = galley.size().x;

            if cursor_x + text_width > WIDTH - HSTEP {
                cursor_y += self.context.fonts_mut(|f|
                    f.row_height(&font_id)) * 1.25;
                cursor_x = HSTEP;
            }
            self.texts.push(Text {
                content: c.to_string(),
                x: cursor_x,
                y: cursor_y,
            });


            cursor_x += text_width + space_width;

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
    pub fn lex(text: &str) -> String {
        let mut output = String::new();
        let mut in_tag = false;
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => output.push(c),
                _ => {}
            }
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
struct Text {
    /// The string content of the word or character.
    content: String,
    /// Absolute horizontal position in points.
    x: f32,
    /// Absolute vertical position in points.
    y: f32
}