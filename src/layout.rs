use std::sync::Arc;
use eframe::epaint::{Color32, FontFamily, FontId};
use egui::Context;
use crate::browser::DrawText;
use crate::node::Token;

pub const HSTEP: f32 = 13.0;
pub const VSTEP: f32 = 17.0;

pub const WIDTH: f32 = 800.0;
pub const HEIGHT: f32 = 600.0;
pub struct Layout {
    /// A collection of positioned text elements ready for rendering.
    pub(crate) texts: Vec<DrawText>,
    cursor_x: f32,
    cursor_y: f32,
    font_family: String,
    font_weight: String,
    font_style: String,
    context: egui::Context,
}

impl Layout {
    /// Creates a new instance of the `Self` struct by initializing its properties
    /// and processing a vector of `Token` elements.
    ///
    /// # Parameters
    /// - `tokens`: A vector of `Token` objects that will be processed and
    ///   incorporated into the layout.
    /// - `context`: A `Context` object that provides additional information or
    ///   settings required for layout initialization.
    ///
    /// # Returns
    /// A new instance of `Self` with its properties initialized and tokens processed.
    ///
    /// # Behavior
    /// - Initializes the layout with default values:
    ///   - An empty `texts` vector.
    ///   - Default cursor positions `cursor_x` and `cursor_y` set to `HSTEP` and `VSTEP` respectively.
    ///   - Default font properties: `font_family` as `"sans"`, and empty strings
    ///     for `font_weight` and `font_style`.
    ///   - Clones the provided `context` to assign it to the layout context.
    /// - Iterates through each `Token` in the provided `tokens` vector and processes it
    ///   using the `token` method.
    ///
    pub fn new(tokens: &Vec<Token>, context: Context) -> Self {

        let mut layout = Self {
            texts: Vec::new(),
            cursor_x: HSTEP,
            cursor_y: VSTEP,
            font_family: "sans".to_string(),
            font_weight: "".to_string(),
            font_style: "".to_string(),
            context,
        };

        for token in tokens {
            layout.token(token)
        }

        layout
    }

/// Processes a `Token` and updates the formatting or renders text.
    ///
    /// This function processes tokens of type `Token` and updates the current
    /// text style or renders textual content based on the token type. The style
    /// updates are determined by specific tag tokens, such as changing the font
    /// style to italic or bold. Text tokens are split into individual words
    /// and subsequently processed for rendering.
    ///
    /// # Parameters
    /// - `token`: An instance of `Token`, which represents either a formatting
    ///   tag or a textual content to be processed.
    ///
    /// # Behavior
    /// - If the token is a `Tag`, it modifies the current font style or weight
    ///   based on the tag's value. Supported tags are:
    ///     - `"i"`: Sets the font style to italic.
    ///     - `"/i"`: Resets the font style to normal.
    ///     - `"b"`: Sets the font weight to bold.
    ///     - `"/b"`: Resets the font weight to normal.
    ///   Unknown tags are ignored.
    /// - If the token is `Text`, it splits the text into words based on whitespace
    ///   and processes each word individually using the `word` method.
    ///
    /// # Font and Layout
    /// - Uses the combination of `self.font_family`, `self.font_weight`, and
    ///   `self.font_style` to construct a unified font name.
    /// - Creates a `FontId` instance with the specified font size (13.0) and
    ///   font family derived from the font name.
    /// - Calculates the width of a single space character using the current font
    ///   settings for proper spacing during word rendering.
    ///
    /// # Internal Methods
    /// - `self.word`: Called for each word in the `Text` token to process and
    ///   render the word with the current font and layout settings.
    ///
    /// # Example
    /// ```ignore
    /// let mut processor = YourProcessor { /* fields */ };
    /// processor.token(Token::Tag(Tag { tag: "b".to_string() })); // Switches to bold.
    /// processor.token(Token::Text(Text { text: "Hello World".to_string() })); // Renders "Hello" and "World".
    /// processor.token(Token::Tag(Tag { tag: "/b".to_string() })); // Switches back to normal weight.
    /// ```
    fn token(&mut self, token: &Token) {
        let font_name = format!("{}{}{}", self.font_family, self.font_weight, self.font_style);
        let font_id = FontId::new(13.0, FontFamily::Name(Arc::from(font_name.clone())));

        let space_galley = self.context.fonts_mut(|f|
            f.layout_no_wrap(" ".to_string(), font_id.clone(), Color32::BLACK));
        let space_width = space_galley.size().x;


        match token {
            Token::Tag(tag) => {
                match tag.tag.as_str() {
                    "i" => {
                        self.font_style = "italic".into()
                    },
                    "/i" => {
                        self.font_style = "".into()
                    },
                    "b" => {
                        self.font_weight = "bold".into()
                    },
                    "/b" => {
                        self.font_weight = "".into()
                    },
                    _ => {}
                }
            }
            Token::Text(text) => {
                for word in text.text.split_whitespace() {
                    self.word(font_name.clone(), &font_id, space_width, word);
                }

            }
        }

    }

    /// ```rust
    /// This function is responsible for rendering a word in the custom text rendering system.
    /// It lays out the word using the font engine, calculates word dimensions, and handles
    /// text wrapping if the word exceeds the available width.
    ///
    /// # Parameters
    /// - `font_name`: A `String` representing the name of the font to be used.
    /// - `font_id`: A reference to the `FontId` that specifies the font details (e.g., style and size).
    /// - `space_width`: A `f32` value representing the width of the space character to account for spacing after the word.
    /// - `word`: A `&str` that represents the word to be drawn.
    ///
    /// # Functionality
    /// 1. Uses the font engine from `egui` to measure the dimensions of the given word based on the provided `FontId`.
    /// 2. Checks whether the current cursor position (`cursor_x`) plus the word’s width exceeds
    ///    the available rendering area (`WIDTH - HSTEP`).
    ///    - If the word exceeds the available width, it moves the cursor to the next line by increasing
    ///      `cursor_y` and resetting `cursor_x` to the starting horizontal position (`HSTEP`).
    /// 3. Adds the word as a `DrawText` object to the `texts` list, including its content, position,
    ///    and font properties.
    /// 4. Updates `cursor_x` to include the word’s width and the provided `space_width`.
    ///
    /// # Remarks
    /// - The function is dependent on constants or variables such as `WIDTH` and `HSTEP` for layout constraints.
    /// - The `row_height` method of the font engine is used to determine the height of a line when moving to a new one.
    /// - The function assumes that `cursor_x` and `cursor_y` are updated globally for text positioning.
    /// ```
    fn word(&mut self, font_name: String, font_id: &FontId, space_width: f32, word: &str) {
        // Access egui's font engine to measure word dimensions
        let galley = self.context.fonts_mut(|f|
            f.layout_no_wrap(word.to_string(), font_id.clone(), Color32::BLACK));

        let text_width = galley.size().x;

        if self.cursor_x + text_width > WIDTH - HSTEP {
            self.cursor_y += self.context.fonts_mut(|f|
                f.row_height(&font_id)) * 1.25;
            self.cursor_x = HSTEP;
        }
        self.texts.push(DrawText {
            content: word.to_string(),
            x: self.cursor_x,
            y: self.cursor_y,
            font_name: font_name.to_string(),
        });

        self.cursor_x += text_width + space_width;
    }
}