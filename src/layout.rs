use std::cell::RefCell;
use std::rc::Rc;
use crate::browser::DrawText;
use crate::node::{HtmlNode, HtmlNodeType};
use eframe::epaint::{Color32, FontFamily, FontId};
use egui::{Context};
use std::sync::Arc;

pub const HSTEP: f32 = 13.0;
pub const VSTEP: f32 = 17.0;

pub const WIDTH: f32 = 800.0;
pub const HEIGHT: f32 = 600.0;

/// Represents a node in the layout tree, which corresponds to an element in the
/// HTML document and stores layout-related information for rendering.
///
/// # Fields
///
/// * `node` - A reference-counted, mutable reference to the corresponding HTML node
///            in the DOM. This provides access to the attributes, tag name, and other
///            relevant properties of the HTML element.
///
/// * `parent` - An `Option` containing a reference-counted, mutable reference to the
///              parent layout node, if it exists. This links the node to its parent
///              in the tree structure.
///
/// * `children` - A vector of reference-counted, mutable references to the child
///                layout nodes. This represents the hierarchical relationships between
///                nodes, allowing traversal of the layout tree.
///
/// * `previous` - An `Option` containing a reference-counted, mutable reference to the
///                previous sibling layout node, if it exists. This facilitates sibling
///                traversal within the tree.
///
/// * `content` - Specifies the type of content this layout node represents. This could be
///               information about text, inline elements, or blocks, as represented by
///               the `LayoutNodeType` enum.
///
/// * `display_list` - A reference-counted, mutable reference to a vector of `DrawText`
///                    objects, which form the display list for rendering. This contains
///                    drawing instructions (e.g., positioning and styles) for the graphical
///                    representation of the node.
pub struct LayoutNode {
    node: Rc<RefCell<HtmlNode>>,
    parent: Option<Rc<RefCell<LayoutNode>>>,
    children: Vec<Rc<RefCell<LayoutNode>>>,
    previous: Option<Rc<RefCell<LayoutNode>>>,
    content: LayoutNodeType,
    pub(crate) display_list: Rc<RefCell<Vec<DrawText>>>
}

impl LayoutNode {
    /// Creates a new layout node representing a document.
    ///
    /// # Parameters
    /// - `node`: A reference-counted, mutable container (`Rc<RefCell<HtmlNode>>`) that represents
    ///   the associated HTML node for this layout node.
    ///
    /// # Returns
    /// - A reference-counted, mutable container (`Rc<RefCell<LayoutNode>>`) encapsulating the newly
    ///   created layout node.
    ///
    /// # Description
    /// This function initializes and returns a new `LayoutNode` instance configured as a document
    /// node. The `LayoutNode` is created with the following properties:
    /// - `node`: The provided HTML node reference.
    /// - `parent`: Set to `None`, indicating no parent node.
    /// - `children`: An empty vector, as no child nodes are present initially.
    /// - `previous`: Set to `None`, indicating no sibling node.
    /// - `content`: Set to `LayoutNodeType::Document`, specifying its type as a document node.
    /// - `display_list`: An empty vector wrapped in a reference-counted and mutable container
    ///   (`Rc<RefCell>`), intended for storing renderable items.
    ///
    /// This is a utility method useful for layout tree creation in browsers or UI systems.
    pub fn new_document(node: Rc<RefCell<HtmlNode>>) -> Rc<RefCell<LayoutNode>> {
        Rc::new(RefCell::new(Self {
            node,
            parent: None,
            children: Vec::new(),
            previous: None,
            content: LayoutNodeType::Document,
            display_list: Rc::new(RefCell::new(Vec::new()))
        }))
    }

    /// Creates a new `LayoutNode` of type `Block` with default styling and layout attributes.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference-counted and mutable `HtmlNode` associated with this layout node.
    /// * `parent` - An `Option` encapsulating a reference-counted and mutable parent `LayoutNode`.
    ///              Specifies the parent layout node in the layout tree hierarchy, or `None` if it is the root.
    /// * `previous` - An `Option` encapsulating a reference-counted and mutable previous sibling `LayoutNode`.
    ///                Specifies the layout node that comes directly before this node in the layout tree structure,
    ///                or `None` if it is the first child.
    /// * `context` - A `Context` object that represents the environment or settings to be carried over for
    ///               this layout node (e.g., shared properties, rendering context).
    ///
    /// # Returns
    ///
    /// A reference-counted and mutable `LayoutNode` (`Rc<RefCell<LayoutNode>>`) with the following attributes:
    /// - Contains no children initially (`children` is empty).
    /// - Configures the node's content type as `LayoutNodeType::Block`, which holds styling information, such as:
    ///     * `font_family`: Defaults to `"sans"`.
    ///     * `font_weight`, `font_style`: Empty strings (default values).
    ///     * `font_size`: Defaults to `16.0`.
    ///     * `context`: Cloned from the given `context`.
    ///     * `cursor_y`, `cursor_x`: Default offsets (`VSTEP` and `HSTEP` respectively) for layout starting positions.
    ///     * `line`: Initialized empty (`Vec::new()`).
    ///     * `font_id`: Defaults to `FontId::default()`.
    ///     * `space_width`: Defaults to `0.0`.
    ///     * `display_list`: An empty display list (`vec![]`) for rendering content.
    /// - Takes a `parent` and `previous` node if provided.
    /// - Initializes its own `display_list` as an empty, reference-counted mutable vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// let html_node = Rc::new(RefCell::new(HtmlNode::new()));
    /// let parent_node = Rc::new(RefCell::new(LayoutNode::new()));
    /// let context = Context::new();
    ///
    /// let layout_node = LayoutNode::new_block(
    ///     html_node,
    ///     Some(parent_node),
    ///     None,
    ///     context
    /// );
    /// ```
    pub fn new_block(node: Rc<RefCell<HtmlNode>>,
                     parent: Option<Rc<RefCell<LayoutNode>>>,
                     previous: Option<Rc<RefCell<LayoutNode>>>,
                     context: Context) -> Rc<RefCell<LayoutNode>> {
        Rc::new(RefCell::new(Self {
            node,
            parent,
            children: vec![],
            content: LayoutNodeType::Block(
                BlockLayout{
                    font_family: "sans".to_string(),
                    font_weight: "".to_string(),
                    font_style: "".to_string(),
                    font_size: 16.0,
                    context: context.clone(),
                    cursor_y: VSTEP,
                    cursor_x: HSTEP,
                    line: Vec::new(),
                    font_id: FontId::default(),
                    space_width: 0.0,
                    display_list: vec![]
                }
            ),
            previous,
            display_list: Rc::new(RefCell::new(Vec::new()))
        }))
    }

    /// Performs layout processing on a `LayoutNode` based on its type (`Block` or `Document`)
    /// and updates its display list accordingly.
    ///
    /// # Parameters
    /// - `node`: A `Rc<RefCell<LayoutNode>>` representing the layout node to be processed.
    /// - `context`: A `Context` object providing relevant layout context or configuration.
    ///
    /// # Functionality
    /// 1. **Determine Node Type**:
    ///    - Checks whether the `LayoutNode` is of type `Block` or `Document` by pattern matching the `content` field.
    ///
    /// 2. **Handle Block Nodes**:
    ///    - Prepares `inner_node_ptr` by cloning the `node` reference.
    ///    - Retrieves a mutable borrow of the `node` to perform updates.
    ///    - Updates the block layout:
    ///      - Invokes `update_font()` on the block layout.
    ///      - Recursively performs layout logic using `recurse(inner_node_ptr)`.
    ///      - Flushes the block's line buffer using `flush_line()`.
    ///    - Updates the `display_list` property with a clone of the block's display list.
    ///    - Drops the mutable borrow once updates are complete.
    ///
    /// 3. **Handle Document Nodes**:
    ///    - Clones the `inner_node_ptr` for the document node.
    ///    - Creates a new `Block` node as a child using `Self::new_block()` with the current context.
    ///    - Updates the `children` of the current node with the newly created child.
    ///    - Recursively processes the child node by calling `Self::layout()` to layout its children.
    ///    - Merges the child's display list with the parent node's display list to ensure proper rendering.
    ///
    /// # Notes
    /// - Borrowing is carefully managed to ensure mutable and immutable borrows do not coexist,
    ///   using scoped blocks or temporary variables.
    /// - The `display_list` is synchronously updated between parent and child nodes to maintain a proper rendering hierarchy.
    ///
    /// # Dependencies
    /// - Requires `LayoutNode` to have the following properties:
    ///   - `content`: Enum of type `LayoutNodeType` (with variants like `Block` and `Document`).
    ///   - `node`: A reference or pointer to additional node data.
    ///   - `children`: A mutable vector to store child nodes.
    ///   - `display_list`: A reference-counted, mutable display list specific to the node.
    ///
    /// # Example
    /// ```rust
    /// let layout_node = Rc::new(RefCell::new(LayoutNode::new_document()));
    /// let context = Context::new();
    /// MyLayoutEngine::layout(layout_node, context);
    /// ```
    ///
    /// In the example above, a `Document` node is processed using the `layout` function,
    /// which creates child blocks, updates their layout, and synchronizes display lists.
    pub fn layout(node: Rc<RefCell<LayoutNode>>, context: Context) {
        println!("Layout node: {:?}", node.borrow().node);
        // 1. Identify what type of node we are dealing with.
        // We use a scoped block or a temporary variable to ensure the borrow is dropped immediately.
        let is_block = matches!(node.borrow().content, LayoutNodeType::Block(_));
        let is_doc = matches!(node.borrow().content, LayoutNodeType::Document);

        if is_block {
            // Prepare data needed for the block logic
            let inner_node_ptr = node.borrow().node.clone();

            // Re-borrow mutably only for the block work
            let mut node_borrow = node.borrow_mut();
            if let LayoutNodeType::Block(ref mut block_layout) = node_borrow.content {
                block_layout.update_font();
                block_layout.recurse(inner_node_ptr);
                block_layout.flush_line();
                node_borrow.display_list = Rc::new(RefCell::new(block_layout.display_list.clone()));
            }
            // Borrow is dropped here when node_borrow goes out of scope
        } else if is_doc {
            let inner_node_ptr = node.borrow().node.clone();

            // Create the child
            let child = Self::new_block(inner_node_ptr, Some(node.clone()), None, context.clone());

            // Update children list and drop borrow immediately
            node.borrow_mut().children.push(child.clone());

            // Perform recursive layout (node is currently unborrowed)
            Self::layout(child.clone(), context);

            // Sync display lists
            let child_dl = child.borrow_mut().display_list.clone();
            node.borrow_mut().display_list.borrow_mut().append(&mut *child_dl.borrow_mut());
        }
    }
}

/// Represents the type of a layout node in a rendering or document processing system.
///
/// This enum is used to differentiate between various kinds of layout nodes:
///
/// - `Block`: Represents a block-level layout node, encapsulating its specific layout structure (`BlockLayout`).
/// - `Document`: Represents the root-level node, typically the entire document structure.
///
/// # Variants
///
/// * `Block(BlockLayout)`
///   - A block-level layout node, containing a `BlockLayout` structure that defines
///     specific properties and behaviors for this block layout.
/// * `Document`
///   - The root layout node, representing the overall document in the layout tree.
///
/// # Example
/// ```rust
/// use crate::LayoutNodeType;
/// use crate::BlockLayout; // assume BlockLayout is imported.
///
/// let block_layout = BlockLayout { /* properties */ };
/// let block_node = LayoutNodeType::Block(block_layout);
///
/// let document_node = LayoutNodeType::Document;
/// ```
///
/// This enum can be further extended or customized to accommodate additional layout types in the system.
enum LayoutNodeType {
    Block(BlockLayout),
    Document
}

/// Represents the layout and formatting attributes for text blocks.
///
/// The `BlockLayout` struct is utilized to manage and render text blocks
/// within a graphical user interface or a similar context. It provides
/// properties for font customization, cursor positioning, and the internal
/// data required for drawing the text.
///
/// # Fields
///
/// * `display_list` - A vector containing `DrawText` elements representing
///   the individual pieces of text to be drawn on the display.
///
/// * `font_family` - A string specifying the font family to be used for text
///   rendering (e.g., "Arial", "Times New Roman").
///
/// * `font_weight` - A string indicating the weight (thickness) of the font
///   (e.g., "normal", "bold").
///
/// * `font_style` - A string defining the style of the font (e.g., "normal",
///   "italic").
///
/// * `font_size` - A floating-point number representing the size of the font
///   in points.
///
/// * `cursor_x` - A floating-point value representing the current horizontal
///   position of the cursor in the layout.
///
/// * `cursor_y` - A floating-point value representing the current vertical
///   position of the cursor in the layout.
///
/// * `context` - A `Context` object containing the necessary information for
///   managing and rendering the text block within its environment.
///
/// * `line` - A vector of `DrawText` elements that represent the text in the
///   current line being processed or rendered.
///
/// * `font_id` - A `FontId` identifying the specific font being used from a
///   font management system.
///
/// * `space_width` - A floating-point value specifying the width of a space
///   character, which can vary depending on the font and its settings.
pub struct BlockLayout {
    pub(crate) display_list: Vec<DrawText>,
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

impl BlockLayout {

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
        if self.line.is_empty() {
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
            self.display_list.push(DrawText {
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