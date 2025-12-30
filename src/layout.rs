use std::cell::RefCell;
use std::rc::Rc;
use crate::browser::DrawText;
use crate::node::{HtmlNode, HtmlNodeType};
use eframe::epaint::{Color32, FontFamily, FontId};
use egui::{Context, TextBuffer};
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
    line: Vec<DrawText>,
    font_id: FontId,
    space_width: f32,

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
    pub fn new(node: Rc<RefCell<HtmlNode>>, context: Context) -> Self {

        let mut layout = Self {
            texts: Vec::new(),
            font_family: "sans".to_string(),
            font_weight: "".to_string(),
            font_style: "".to_string(),
            font_size: 16.0,
            context: context.clone(),
            cursor_y: VSTEP,
            cursor_x: HSTEP,
            line: Vec::new(),
            font_id: FontId::default(),
            space_width: 0.0

        };

        layout.update_font();
        layout.recurse(node);

        layout.flush_line();


        layout
    }

    /// Handles opening HTML-like tags and adjusts the corresponding text formatting properties
    /// or behavior of the object accordingly.
    ///
    /// This function is responsible for interpreting specific tags and modifying the object's
    /// state. It supports a small subset of tags, which adjust font styles, font weights,
    /// font sizes, or introduce line breaks.
    ///
    /// # Parameters
    /// - `tag`: A `String` representing the tag (such as "i", "b", "big", "small", "br") to be processed.
    ///
    /// # Behavior
    /// - `"i"`: Sets the `font_style` property to `"italic"`.
    /// - `"b"`: Sets the `font_weight` property to `"bold"`.
    /// - `"big"`: Increases the `font_size` property by `16.0/3.0`.
    /// - `"small"`: Decreases the `font_size` property by `8.0/3.0`.
    /// - `"br"`: Calls the `flush_line` method to perform a line break.
    /// - Default case (`_`): If the tag does not match any of the above, no action is performed.
    ///
    ///
    /// Note: Tags that are not explicitly handled in the match block are ignored.
    fn open_tag(&mut self, tag: String)
    {
        match tag.as_str() {
            "i" => {
                self.font_style = "italic".into();
                self.update_font()
            },
            "b" => {
                self.font_weight = "bold".into();
                self.update_font()

            },
            "big" => {
                self.font_size += 16.0/3.0;
                self.update_font()
            },
            "small" => {
                self.font_size -= 8.0/3.0;
                self.update_font()
            },
            "br" => {
                self.flush_line();
            }
            _ => {}
        }
    }

    /// Closes a given HTML-like tag and updates the rendering state accordingly.
    ///
    /// This function modifies the internal properties of the object to reflect the
    /// closing of a specified tag. Depending on the tag provided, it performs actions
    /// such as resetting font styles, adjusting font size, or moving the cursor.
    ///
    /// # Parameters
    /// - `tag` (`String`): The name of the tag being closed. Supported tags are:
    ///   - `"i"`: Resets the italic font style.
    ///   - `"b"`: Resets the bold font weight.
    ///   - `"big"`: Decreases the font size by `16.0 / 3.0`.
    ///   - `"small"`: Increases the font size by `8.0 / 3.0`.
    ///   - `"p"`: Flushes the current line, moves the cursor down by a predefined vertical step (`VSTEP`).
    ///
    /// Any unsupported or unrecognized tags will be ignored.
    ///
    /// # Behavior
    /// - For `"i"` and `"b"`, the corresponding font properties (`font_style` and
    ///   `font_weight`) will be reset to an empty string.
    /// - For font size changes (`"big"` and `"small"`), proper adjustments are applied
    ///   to the `font_size` property.
    /// - For the `"p"` tag, the current line is flushed, and the vertical position of
    ///   the cursor (`cursor_y`) is incremented by the value of `VSTEP`.
    fn close_tag(&mut self, tag: String)
    {
        match tag.as_str() {
            "i" => {
                self.font_style = "".into();
                self.update_font()
            },
            "b" => {
                self.font_weight = "".into();
                self.update_font()
            },
            "big" => {
                self.font_size -= 16.0/3.0;
                self.update_font()

            },
            "small" => {
                self.font_size += 8.0/3.0;
                self.update_font()
            },
            "p" => {
                self.flush_line();
                self.cursor_y += VSTEP;
            }
            _ => {}
        }
    }

    /// Adds a word to the current line of text being processed in the rendering context.
    /// If the word does not fit within the remaining horizontal space on the line, the current
    /// line is flushed and the word is added to a new line.
    ///
    /// # Parameters
    /// - `word`: A reference to the string slice representing the word to be added.
    ///
    /// # Behavior
    /// - Uses egui's font engine to measure the width of the word in pixels based on the
    ///   provided font size and color.
    /// - Checks if adding the word would exceed the maximum allowed line width (`WIDTH - HSTEP`).
    ///   - If it would exceed, the current line is flushed (`self.flush_line()`), and the word
    ///     is added to a new line.
    /// - Adds the word's visual representation and position (using `DrawText`) to the current line.
    /// - Updates the horizontal cursor position (`self.cursor_x`) to account for the added word and
    ///   the required space between words.
    ///
    /// # Notes
    /// - `self.context.fonts_mut` is used to access and measure the font metrics for the word.
    /// - `self.line` is updated with a new `DrawText` object containing the word's rendered
    ///   galley and its position on the current line.
    /// - Assumes that `self.flush_line()` properly handles the process of completing the current
    ///   line and prepares for new content.
    /// - Relies on the constants `WIDTH` and `HSTEP` for layout constraints.
    ///
    /// # Example
    /// ```
    /// let mut renderer = Renderer::new();
    /// renderer.word("Example");
    fn word(&mut self, word: &str) {
        // Access egui's font engine to measure word dimensions
        let galley = self.context.fonts_mut(|f|
            f.layout_no_wrap(word.to_string(), self.font_id.clone(), Color32::BLACK));

        let text_width = galley.size().x;


        if self.cursor_x + text_width > WIDTH - HSTEP {
            self.flush_line();
        }

        self.line.push(DrawText {
                x: self.cursor_x,
                y: 0.0,
                galley
            });
            self.cursor_x += text_width + self.space_width;




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

    /// Recursively processes an HTML tree represented by `HtmlNode`.
    ///
    /// # Arguments
    ///
    /// * `tree` - A reference-counted, mutable `HtmlNode` wrapped in a `RefCell`.
    ///
    /// The function determines the type of the node (`Element` or `Text`) and takes the
    /// appropriate actions based on the node type. For `Element` nodes:
    /// - The function opens a tag for the element using `self.open_tag`.
    /// - Recursively processes each of its child nodes.
    /// - Closes the tag for the element using `self.close_tag`.
    ///
    /// For `Text` nodes:
    /// - Splits the text into whitespace-separated words.
    /// - Processes each word individually using `self.word`.
    ///
    /// Internally, the node's properties are borrowed immutably to enable safe access.
    /// The `Action` enum is used as an intermediate abstraction to encapsulate what needs
    /// to be done for a particular node type.
    ///
    /// # Node Types
    ///
    /// - `HtmlNodeType::Element`: Represents an element node with a tag name and children.
    /// - `HtmlNodeType::Text`: Represents a text node containing text content.
    ///
    /// # Example
    ///
    /// ```
    /// let root_node = Rc::new(RefCell::new(HtmlNode::new_element("div")));
    /// let child_text = Rc::new(RefCell::new(HtmlNode::new_text("Hello, World!")));
    /// root_node.borrow_mut().children.push(child_text.clone());
    ///
    /// let mut processor = HtmlTreeProcessor::new();
    /// processor.recurse(root_node);
    /// ```
    ///
    /// The above example demonstrates creating an `HtmlNode` tree with a `div` element
    /// containing a text node. The `recurse` function processes this tree recursively.
    fn recurse(&mut self, tree: Rc<RefCell<HtmlNode>>) {
        enum Action {
            ProcessElement { tag: String, children: Vec<Rc<RefCell<HtmlNode>>> },
            ProcessText(String),
        }

        let action = {
            let borrowed = tree.borrow();
            match &borrowed.node_type {
                HtmlNodeType::Element(ele) => Action::ProcessElement {
                    tag: ele.tag.clone().into(),
                    children: borrowed.children.clone(),
                },
                HtmlNodeType::Text(txt) => Action::ProcessText(txt.text.clone()),
            }
        };

        match action {
            Action::ProcessElement { tag, children } => {
                self.open_tag(tag.clone());
                for child in children {
                    self.recurse(child);
                }
                self.close_tag(tag);
            }
            Action::ProcessText(text) => {
                for word in text.split_whitespace() {
                    self.word(word);
                }
            }
        }
    }

    /// Updates the font configuration for the current context.
    ///
    /// This method constructs a unique font identifier string by concatenating the
    /// font family, font weight, and font style. It then sets the `font_id` property
    /// using the specified font size and the newly constructed font identifier.
    ///
    /// Additionally, this method calculates the width of a single space character
    /// (`space_width`) using the updated font settings, which may be used for
    /// layout calculations or spacing adjustments.
    ///
    /// # Fields Updated
    /// - `font_id`: Represents the new font descriptor based on the updated parameters.
    /// - `space_width`: Stores the width of a single space character (`" "`) for the selected font.
    ///
    /// # Process
    /// 1. Constructs the `font_name` as a combination of:
    ///    - `font_family`
    ///    - `font_weight`
    ///    - `font_style`
    /// 2. Updates the `font_id` with the specified font size and the constructed font identifier.
    /// 3. Computes the width of a space character (`space_width`) by laying out a single
    ///    space using the updated `font_id`.
    ///
    /// # Requirements
    /// - This method assumes `self.context` provides access to the font system
    ///   via its `fonts_mut` method.
    /// - The `FontId` and `FontFamily::Name` types must be available in the scope.
    ///
    /// # Example
    /// ```rust
    /// // Before calling update_font, ensure the font properties like
    /// // font_family, font_weight, font_style, and font_size are properly set.
    /// obj.update_font();
    ///
    /// // After calling update_font, the `font_id` and `space_width` are updated.
    /// println!("Font ID updated to: {:?}", obj.font_id);
    /// println!("Space width calculated as: {:?}", obj.space_width);
    /// ```
    fn update_font(&mut self) {
        let font_name = format!("{}{}{}", self.font_family, self.font_weight, self.font_style);
        self.font_id = FontId::new(self.font_size, FontFamily::Name(Arc::from(font_name.clone())));
        let space_galley = self.context.fonts_mut(|f|
            f.layout_no_wrap(" ".to_string(), self.font_id.clone(), Color32::BLACK));
        self.space_width = space_galley.size().x;
    }
}