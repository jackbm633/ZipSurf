use crate::browser::DrawText;
use crate::node::Token;
use eframe::epaint::{Color32, FontFamily, FontId};
use egui::Context;
use std::sync::Arc;

pub const HSTEP: f32 = 13.0;
pub const VSTEP: f32 = 17.0;

pub const WIDTH: f32 = 800.0;
pub const HEIGHT: f32 = 600.0;
pub struct Layout {
    /// A collection of positioned text elements ready for rendering.
    pub(crate) texts: Vec<DrawText>,
    font_family: String,
    font_weight: String,
    font_style: String,
    font_size: f32,
    cursor_x: f32,
    cursor_y: f32,
    context: Context,
    line: Vec<DrawText>
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
            font_family: "sans".to_string(),
            font_weight: "".to_string(),
            font_style: "".to_string(),
            font_size: 16.0,
            context: context.clone(),
            cursor_y: VSTEP,
            cursor_x: HSTEP,
            line: Vec::new()
        };
        //

        for token in tokens {
            layout.token(token)
        }

        layout.flush_line();


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
        let font_id = FontId::new(self.font_size, FontFamily::Name(Arc::from(font_name.clone())));

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
                    "big" => self.font_size += 16.0/3.0,
                    "/big" => self.font_size -= 16.0/3.0,
                    "small" => self.font_size -= 8.0/3.0,
                    "/small" => self.font_size += 8.0/3.0,
                    "br" => {
                        self.flush_line();
                    }

                    "/p" => {
                        self.flush_line();
                        self.cursor_y += VSTEP;
                    }

                    _ => {}
                }
            }
            Token::Text(text) => {
                for word in text.text.split_whitespace() {
                    self.word(&font_id, space_width, word);
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
    fn word(&mut self, font_id: &FontId, space_width: f32, word: &str) {
        // Access egui's font engine to measure word dimensions
        let galley = self.context.fonts_mut(|f|
            f.layout_no_wrap(word.to_string(), font_id.clone(), Color32::BLACK));

        let text_width = galley.size().x;


        if self.cursor_x + text_width > WIDTH - HSTEP {
            self.flush_line();
        }

        self.line.push(DrawText {
                x: self.cursor_x,
                y: 0.0,
                galley
            });
            self.cursor_x += text_width + space_width;




    }

    /// Flushes the current line of text into the `texts` buffer, positions the elements vertically based
    /// on font metrics, and updates the cursor position for the next line of text.
    ///
    /// # Behavior
    /// - If the current line (`self.line`) is empty, this method returns early without taking action.
    /// - Computes the maximum ascent and descent values from the `galley` (text layout) of each item in the line.
    /// - Calculates the baseline position for the text layout based on the current cursor y-coordinate and
    ///   the maximum ascent value, with an added adjustment factor (1.25 times ascent).
    /// - Vertically positions each text item in the current line by adjusting their `y` coordinate relative
    ///   to the calculated baseline. Horizontally, the position remains unchanged.
    /// - Appends each adjusted text item into the `texts` buffer as a `DrawText` structure, preserving
    ///   `x`, `y`, and `galley`.
    /// - Clears the current line (`self.line`) after processing.
    /// - Updates the cursor position:
    ///   - Resets the `cursor_x` to a predefined horizontal step (`HSTEP`).
    ///   - Moves the `cursor_y` downward based on the baseline and 1.25 times the maximum descent value.
    ///
    /// # Assumptions
    /// - Each item in the line contains a `galley` object, with rows, rows containing rows of glyphs,
    ///   and each glyph having font metrics such as `font_ascent` and `font_height`.
    /// - At least one glyph is present in each row of every galley. Proper checks must ensure these conditions.
    ///
    /// # Dependencies
    /// - Uses the methods:
    ///   - `.is_empty()` to check if the line is empty.
    ///   - `.iter()` to iterate over the line and compute ascent and descent values.
    ///   - `.clone()` to duplicate `galley` objects for processing.
    ///   - `reduce(f32::max)` to identify the maximum ascent and descent values.
    /// - Presumes `HSTEP` is a global or class-defined constant that defines the horizontal step for the cursor.
    ///
    /// # Parameters
    /// This method modifies the following properties of the struct:
    /// - `self.line`: Cleared after processing.
    /// - `self.texts`: Appends `DrawText` structs for the processed line.
    /// - `self.cursor_x`: Reset to `HSTEP`.
    /// - `self.cursor_y`: Updated to a new vertical position based on the calculated baseline and descent.
    ///
    /// # Example
    /// ```rust
    /// let mut renderer = Renderer::new();
    /// renderer.line.push(TextItem::new("Hello, World!"));
    /// renderer.flush_line();
    /// assert!(renderer.line.is_empty());
    /// assert_eq!(renderer.cursor_x, HSTEP);
    /// assert!(renderer.cursor_y > 0.0);
    /// ```
    ///
    /// # Notes
    /// - The use of `.unwrap()` assumes valid and non-empty data for rows and glyphs in the `galley`.
    ///   Ensure robustness by adding necessary validation where appropriate.
    /// - Considerations for performance: Cloning objects and iterating multiple times over the line could
    ///   have performance implications for very large datasets. Optimize if necessary.
    fn flush_line(&mut self) {
        if (self.line.is_empty()){
            return;
        }

        let galleys = self.line.iter().map(|l| l.galley.clone());
        let max_ascent = galleys.clone().map(|g| g.rows.first().unwrap().row.glyphs.first().unwrap().font_ascent).into_iter()
            .reduce(f32::max)
            .unwrap_or(0.);
        let max_descent = galleys.map(|g| {
            let glyph = g.rows.first().unwrap().row.glyphs.first().unwrap();
            glyph.font_height - glyph.font_ascent
        }).into_iter()
            .reduce(f32::max)
            .unwrap_or(0.);

        let baseline = self.cursor_y + 1.25 * max_ascent;

        for text in &mut self.line {
            text.y = baseline - text.galley.rows.first().unwrap().row.glyphs.first().unwrap().font_ascent;
            self.texts.push(DrawText {
                x: text.x,
                y: text.y,
                galley: text.galley.clone()
            })
        }

        self.cursor_x = HSTEP;
        self.cursor_y = baseline + 1.25 * max_descent;
        self.line.clear();

    }
}