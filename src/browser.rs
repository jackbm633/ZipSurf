use std::sync::Arc;
use eframe::egui;
use egui::{Align2, Color32, FontFamily, FontId, Pos2};
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

    /// Lays out tokens into lines while respecting formatting tags and viewport constraints.
    ///
    /// The `layout` function iterates over the tokens in `self.tokens`, handling how each token
    /// (plain text or formatting tag) affects the placement of text on a virtual canvas. It ensures
    /// that text respects the viewport dimensions, wraps correctly to a new line when necessary,
    /// and applies formatting (e.g., bold and italic styles) based on tags.
    ///
    /// #### Behavior:
    /// - Each word is measured using a font engine to determine its dimensions.
    /// - If the current word doesn't fit in the remaining horizontal space, the cursor moves to
    ///   the next line.
    /// - Words are placed with a consistent horizontal and vertical spacing defined by constants
    ///   such as `HSTEP` and `VSTEP`.
    /// - Formatting tags (`<i>`, `<b>`, etc.) dynamically adjust the style of the text until they
    ///   are closed (e.g., `</i>`, `</b>`).
    ///
    /// #### Token Handling:
    /// 1. **Text Tokens**: Split the text into words, measure word dimensions, and place them
    ///    onto the canvas with appropriate spacing.
    /// 2. **Tag Tokens**: Update the font style (e.g., italic or bold) based on the tag.
    ///
    /// #### Variables:
    /// - `cursor_x` and `cursor_y`:
    ///   Track the current position of the text cursor for word placement.
    /// - `HSTEP` and `VSTEP`:
    ///   Define the horizontal and vertical starting offsets for layout.
    /// - `WIDTH`:
    ///   The maximum horizontal width allowed for text placement before wrapping to a new line.
    /// - `font_family`, `font_weight`, `font_style`:
    ///   Represent the current font settings, dynamically updated as formatting tags are processed.
    ///
    /// #### Font Handling:
    /// - The font to be used for each piece of text is determined dynamically by concatenating
    ///   the `font_family`, `font_weight`, and `font_style`.
    /// - Fonts are accessed and measured using an underlying font renderer (`self.context.fonts_mut`).
    ///
    /// #### Example Usage:
    /// ```rust
    /// self.layout();
    /// ```
    ///
    /// This function works as part of a larger system that involves rendering and formatting
    /// text on a UI, such as within a text editor or a rich text renderer.
    ///
    /// #### Assumptions:
    /// - The function assumes pre-defined constants (`HSTEP`, `VSTEP`, `WIDTH`) and an associated
    ///   data structure managing tokens (`self.tokens` and `self.texts`).
    /// - The `FontId` and `DrawText` structs, along with the `Color32::BLACK` color constant, are
    ///   assumed to be available in the current context.
    ///
    /// #### Limitations:
    /// - Supports only a fixed set of formatting tags (`<i>` for italic, `<b>` for bold).
    /// - Assumes `WIDTH` accurately reflects the horizontal constraint of the layout area.
    ///
    /// This function is part of a text layout module and provides necessary functionality for
    /// rendering properly formatted and positioned text.
    fn layout(&mut self) {
        let mut cursor_x = HSTEP;
        let mut cursor_y = VSTEP;
        let tokens = &self.tokens;

        let font_family = "sans";
        let mut font_weight = "";
        let mut font_style = "";


        for c in tokens {
            let font_name = format!("{}{}{}", font_family, font_weight, font_style);
            let font_id = FontId::new(13.0, FontFamily::Name(Arc::from(font_name.clone())));

            let space_galley = self.context.fonts_mut(|f|
                f.layout_no_wrap(" ".to_string(), font_id.clone(), Color32::BLACK));
            let space_width = space_galley.size().x;


            match c {
                Token::Tag(tag) => {
                    match tag.tag.as_str() {
                        "i" => {
                            font_style = "italic"
                        },
                        "/i" => {
                            font_style = ""
                        },
                        "b" => {
                            font_weight = "bold"
                        },
                        "/b" => {
                            font_weight = ""
                        },
                        _ => {}
                    }
                }
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
                            font_name: font_name.to_string(),
                        });

                        cursor_x += text_width + space_width;
                    }

                }
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
    /// ```rust
    /// Updates the application's state and renders the user interface.
    ///
    /// # Parameters
    /// - `ctx`: A reference to the [`egui::Context`] object, which provides
    ///   context for drawing and handling input events in the `egui` UI framework.
    /// - `_frame`: A mutable reference to the [`eframe::Frame`] object, which can be used
    ///   to modify the frame window or interact with the host application (unused in this case).
    ///
    /// # Behavior
    /// - If `self.texts` is empty, it calls the `layout` method to initialize content.
    /// - Handles user input for scrolling by checking if the down arrow key
    ///   (`egui::Key::ArrowDown`) has been pressed. If so, it increments the `self.scroll_y`
    ///   value by a constant `SCROLL_STEP`.
    /// - Utilizes the [`egui::CentralPanel`] to render its contents:
    ///     - Loops over the `self.texts` collection and draws text elements that are within
    ///       the visible screen area.
    ///     - Skips rendering text elements that are outside the visible area using simple
    ///       culling logic.
    ///     - Uses the [`egui::Painter`] to draw text, aligning the content to the top-left corner
    ///       of its position, with a specified font and color.
    ///
    /// # Notes
    /// - The `scroll_y` value determines the vertical scroll position for the text layout.
    /// - The text culling logic ensures better performance by avoiding rendering of off-screen
    ///   elements.
    /// - Each text element is drawn using the specified font and size (via `FontId`) and color
    ///   (`Color32::BLACK`).
    ///
    /// # Constants Used
    /// - `SCROLL_STEP`: The magnitude of the vertical scroll increment when the down arrow key is pressed.
    /// - `HEIGHT`: Defines the height of the visible area for culling.
    /// - `VSTEP`: The vertical spacing between text elements.
    ///
    /// # Assumptions
    /// - The `self.texts` collection contains elements with `x`, `y`, `content`, and `font_name`
    ///   fields.
    /// - Font size is fixed at `13.0` units for rendering, and `Color32::BLACK` is used for text color.
    ///
    /// # Example Usage
    /// This method is intended to be part of an interactive application that utilizes
    /// `egui` and `eframe` frameworks for rendering and UI interaction.
    /// ```
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
                    FontId::new(13.0, FontFamily::Name(text.font_name.clone().into())),
                    Color32::BLACK
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
struct DrawText {
    /// The string content of the word or character.
    content: String,
    /// Absolute horizontal position in points.
    x: f32,
    /// Absolute vertical position in points.
    y: f32,
    /// Name of the font family used for rendering.
    font_name: String,
}