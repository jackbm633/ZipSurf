use std::sync::Arc;

use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2};

use crate::url::Url;

/// The main application state for the Browser.
/// 
/// In a more complex app, this struct would hold data like URLs, 
/// history, or scroll positions.
pub struct Browser {
    texts: Vec<Text>,
    scroll_y: f32,
}

const HSTEP: f32 = 13.0;
const VSTEP: f32 = 17.0;
const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;
const SCROLL_STEP: f32 = 100.0;

impl Default for Browser {
    /// Provides the default state for the Browser.
    fn default() -> Self {
        Browser {
            texts: Vec::new(),
            scroll_y: 0.0
        }
    }
}

/// Provides browser functionality for loading and rendering web content.
///
/// The `Browser` struct handles URL loading, HTML parsing, and text rendering
/// with support for custom fonts and visual styling. It manages a collection
/// of text elements positioned for display in the egui UI framework.
///
/// # Features
/// - URL loading and HTML tag removal via lexing
/// - Custom font configuration with fallback support
/// - Light mode visuals for optimal visibility
/// - Text element management with positioning
impl Browser {
    /// Configures the initial context and returns a new instance of [`Browser`].
    ///
    /// # Arguments
    /// * `cc` - The creation context, used to set the visual theme and access the GPU.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Enforce light mode so black shapes are visible against the background.
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Self::setup_custom_fonts(&cc.egui_ctx);
        Self::default()
    }

    /// Fetches content from a URL and populates the internal text buffer.
    ///
    /// This method performs a network request, strips HTML tags from the response,
    /// and converts each character into a `Text` struct with a default screen position.
    ///
    /// # Arguments
    /// * `url` - A `Url` object (presumably a custom struct) that provides a `.request()` method.
    ///
    /// # Workflow
    /// 1. Calls `url.request()` to get the raw HTML/body.
    /// 2. Passes the result to `Browser::lex()` to remove tags.
    /// 3. Iterates through the sanitized characters.
    /// 4. Pushes each character into `self.texts` as a `Text` instance.
    ///
    /// # Errors
    /// If the network request fails, an error message is printed to `stderr` 
    /// via `eprintln!`, and no changes are made to `self.texts`.
    pub fn load(&mut self, url: Url) {
        match url.request() {
            Ok(body) => {
                self.layout(body);
            }
            Err(e) => {
                eprintln!("Error loading URL: {}", e);
            }
        }
    }

    /// Processes a string and calculates the 2D layout for each character.
    ///
    /// This function performs a simplified "reflow" logic:
    /// 1. It sanitizes the input using `Browser::lex`.
    /// 2. It iterates through characters, assigning each a coordinate based on 
    ///    horizontal (`HSTEP`) and vertical (`VSTEP`) increments.
    /// 3. It automatically handles line wrapping when the `cursor_x` exceeds the `WIDTH`.
    ///
    /// # Arguments
    /// * `body` - The raw string (likely HTML) to be laid out.
    ///
    /// # Layout Rules
    /// * **Initial Position:** Starts at `(HSTEP, VSTEP)`.
    /// * **Wrapping:** If the next character would exceed `WIDTH`, `cursor_x` resets 
    ///   to `HSTEP` and `cursor_y` increments by `VSTEP`.
    /// * **Spacing:** Every character is treated as having a fixed width of `HSTEP`.
    fn layout(&mut self, body: String) {
        let mut cursor_x = HSTEP;
        let mut cursor_y = VSTEP;
        let text = Browser::lex(body);
        for c in text.chars() {
            self.texts.push(Text {
                content: c.to_string(),
                x: cursor_x,
                y: cursor_y,
            });
            if cursor_x + HSTEP > WIDTH {
                cursor_x = HSTEP;
                cursor_y += VSTEP;
            } else {
                cursor_x += HSTEP;
            }
        }
    }
    /// Removes all HTML/XML-style tags from a given string.
    ///
    /// This function iterates through the input `text`, tracking whether the current
    /// character is inside a tag (between `<` and `>`). It returns a new `String` 
    /// containing only the text found outside of these markers.
    ///
    /// # Arguments
    /// * `text` - A `String` containing the raw text to be processed.
    ///
    /// # Returns
    /// A `String` with all `<...>` tags removed.
    ///
    /// # Example
    /// ```
    /// let input = String::from("<p>Hello, <b>world</b>!</p>");
    /// let result = lex(input);
    /// assert_eq!(result, "Hello, world!");
    /// ```
    pub fn lex(text: String) -> String {
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

    /// Configures custom fonts for the egui context.
    ///
    /// This function loads the "Droid Sans Fallback" font from the local assets, 
    /// registers it with the font manager, and adds it as a fallback for both 
    /// Proportional and Monospace font families. 
    ///
    /// # Behavior
    /// * **Static Inclusion:** The font file is embedded into the binary at compile time 
    ///   using `include_bytes!`.
    /// * **Priority:** The custom font is `.push()`-ed to the end of the font family vectors,
    ///   meaning it will be used only when characters are missing from the default fonts.
    ///
    /// # Arguments
    /// * `ctx` - A reference to the `egui::Context` where the fonts should be applied.
    fn setup_custom_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "droid-sans-fallback".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!("../assets/DroidSansFallbackFull.ttf"))),
        );

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
    /// The main update loop for the application UI.
    ///
    /// This function is called every frame by the `eframe` framework. It clears the
    /// central panel and uses a `Painter` to manually render each character stored
    /// in the `texts` vector at its specified coordinates.
    ///
    /// # Arguments
    /// * `ctx` - The egui context, used to handle input and layout.
    /// * `_frame` - The eframe frame, used for integration with the native window (unused).
    ///
    /// # Drawing Logic
    /// * **Painter:** Accesses the low-level 2D drawing API.
    /// * **Positioning:** Uses `text.x` and `text.y` to determine the screen location.
    /// * **Alignment:** Uses `Align2::LEFT_TOP`, meaning the (x, y) coordinate refers 
    ///   to the top-left corner of the character.
    /// * **Styling:** Renders text in a 16pt Proportional font in Black.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown))
        {
            self.scroll_y += SCROLL_STEP;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // The painter allows direct 2D drawing onto the UI layer.
            let painter = ui.painter();

            for text in &self.texts {
                if (text.y > self.scroll_y + HEIGHT) || (text.y + VSTEP < self.scroll_y) {
                    continue;
                }
                painter.text(
                    Pos2::new(text.x, text.y - self.scroll_y), 
                    Align2::CENTER_CENTER, 
                    &text.content, 
                    FontId::proportional(13.0), 
                    Color32::BLACK
                );
            }
           
        });
    }

    
}

struct Text {
    content: String,
    x: f32,
    y: f32
}