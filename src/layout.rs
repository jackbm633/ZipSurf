
use std::cell::RefCell;
use std::rc::Rc;
use crate::browser::DrawText;
use crate::node::{HtmlNode, HtmlNodeType};
use eframe::epaint::{Color32, FontFamily, FontId};
use egui::{Context, Vec2};
use std::sync::Arc;
use crate::layout::LayoutMode::{Block, Inline};

pub const HSTEP: f32 = 13.0;
pub const VSTEP: f32 = 17.0;

pub const WIDTH: f32 = 800.0;
pub const HEIGHT: f32 = 600.0;

const BLOCK_ELEMENTS: [&str; 37] =  [
    "html", "body", "article", "section", "nav", "aside",
    "h1", "h2", "h3", "h4", "h5", "h6", "hgroup", "header",
    "footer", "address", "p", "hr", "pre", "blockquote",
    "ol", "ul", "menu", "li", "dl", "dt", "dd", "figure",
    "figcaption", "main", "div", "table", "form", "fieldset",
    "legend", "details", "summary"
];

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
    position: Option<Vec2>,
    size: Option<Vec2>,
    pub(crate) display_list: Rc<RefCell<Vec<DrawText>>>
}

impl std::fmt::Debug for LayoutNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutNode")
            .field("content", &self.content)
            .field("children", &self.children)
            .finish()
    }
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
            display_list: Rc::new(RefCell::new(Vec::new())),
            position: None,
            size: None
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
                    cursor_y: 0.0,
                    cursor_x: 0.0,
                    line: Vec::new(),
                    font_id: FontId::default(),
                    space_width: 0.0,
                    display_list: vec![],
                }
            ),
            previous,
            display_list: Rc::new(RefCell::new(Vec::new())),
            position: None,
            size: None
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
        // 1. Identify what type of node we are dealing with.
        // We use a scoped block or a temporary variable to ensure the borrow is dropped immediately.
        let is_block = matches!(node.borrow().content, LayoutNodeType::Block(_));
        let is_doc = matches!(node.borrow().content, LayoutNodeType::Document);

        if is_block {
            // Prepare data needed for the block logic
            let inner_node_ptr = node.borrow().node.clone();

            let mode = Self::layout_mode(inner_node_ptr.clone());
            let mut y: f32 = 0.0;
            if node.borrow().previous.is_some() {
                y = node.borrow().previous.clone().unwrap().borrow().position.unwrap().y
                    + node.borrow().previous.clone().unwrap().borrow().size.unwrap().y;
            } else {
                y = node.borrow().parent.clone().unwrap().borrow().position.unwrap().y;
            }
            let x = node.borrow().parent.clone().unwrap().borrow().position.unwrap().x;
            let width = node.borrow().parent.clone().unwrap().borrow().size.unwrap().x;
            let pos = Some(Vec2::new(x, y));
            let size = Some(Vec2::new(width, 0.0));
            {
                let mut node_borrow = node.borrow_mut();
                node_borrow.position = pos;
                node_borrow.size = size;
            }
            match mode {
                Block => {

                    let mut previous: Option<Rc<RefCell<LayoutNode>>> = None;
                    for child in inner_node_ptr.borrow().children.clone(){
                        let next = Self::new_block(child.clone(),
                                                   Some(node.clone()),
                                                   previous.clone(),
                                                   context.clone());
                        node.borrow_mut().children.push(next.clone());
                        previous = Some(next);
                    }
                }
                Inline => {
                    // Re-borrow mutably only for the block work
                    let mut node_borrow = node.borrow_mut();

                    // Destructure to split borrows, allowing simultaneous access to fields
                    let LayoutNode {
                        ref mut content,
                        ref mut position,
                        ref mut size,
                        ref mut display_list,
                        ..
                    } = *node_borrow;

                    if let LayoutNodeType::Block(block_layout) = content {
                        let mut composer = BlockComposer {
                            layout: block_layout,
                            outer_position: position,
                            outer_size: size
                        };

                        composer.update_font();
                        composer.recurse(inner_node_ptr);
                        composer.flush_line();

                        *display_list = Rc::new(RefCell::new(composer.layout.display_list.clone()));
                        node_borrow.size = Some(Vec2::new(size.unwrap().x, block_layout.cursor_y));
                    }
                }
            }

            for layout in node.borrow().children.iter() {
                Self::layout(layout.clone(), context.clone());
            }

            match mode {
                Inline => {

                }
                Block => {
                    let mut node_borrow = node.borrow_mut();

                    // 1. Calculate height based on children
                    let total_height: f32 = node_borrow.children
                        .iter()
                        .map(|c| c.borrow().size.unwrap().y)
                        .sum();

                    node_borrow.size = Some(Vec2::new(size.unwrap().x, total_height));
                }
            }


            // Borrow is dropped here when node_borrow goes out of scope
        } else if is_doc {
            let inner_node_ptr = node.borrow().node.clone();

            node.borrow_mut().position = Some(Vec2::new(HSTEP, VSTEP));
            node.borrow_mut().size = Some(Vec2::new(WIDTH - 2.0*HSTEP, 0.0));
            // Create the child
            let child = Self::new_block(inner_node_ptr, Some(node.clone()), None, context.clone());

            // Update children list and drop borrow immediately
            node.borrow_mut().children.push(child.clone());

            // Perform recursive layout (node is currently unborrowed)
            Self::layout(child.clone(), context);
            let child_size = child.borrow().size.unwrap();

            node.borrow_mut().size = Some(Vec2::new(WIDTH - 2.0 * HSTEP, child_size.y));
        }
    }

    /// Generates a list of `DrawText` objects representing the content to be
    /// painted or drawn. The function behavior depends on the type of layout node.
    ///
    /// # Returns
    /// - A `Vec<DrawText>` containing the rendered content for the layout node.
    ///
    /// ## Behavior:
    /// - If the layout node is a `LayoutNodeType::Document`, an empty vector is returned
    ///   as no specific content needs to be drawn for a document node.
    /// - If the layout node is a `LayoutNodeType::Block`, the function clones and
    ///   returns the `display_list` associated with the block.
    ///
    /// # Example
    /// ```rust
    /// let layout_node = LayoutNodeType::Block(block_instance);
    /// let draw_content = layout_node.paint();
    /// // `draw_content` now contains the `display_list` of the block.
    /// ```
    ///
    /// # Note
    /// This function assumes that the layout node and its associated structures
    /// (e.g., `display_list` in a block) are properly initialized and accessible.
    pub fn paint(&self) -> Vec<DrawText>{
        match &self.content {
            LayoutNodeType::Document => {vec![]},
            LayoutNodeType::Block(blk) => {
                blk.display_list.clone()
            }
        }
    }

    /// Recursively traverses the layout tree and populates the display list with draw commands.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference-counted and internally mutable pointer to the root `LayoutNode` from which the painting begins.
    /// * `display_list` - A mutable reference to a vector that collects the `DrawText` commands for rendering.
    ///
    /// # Details
    ///
    /// This function processes the current `node` by appending its paint commands to
    /// the `display_list`. It then iterates over each child of the current `node` and
    /// recursively calls itself (`paint_tree`) to continue the painting process down
    /// the tree.
    ///
    /// ## Key points:
    /// - Painting begins with the provided `node` and traverses all of its children.
    /// - `node.borrow()` is used to safely access the node's data while managing
    ///   internal mutability using `RefCell`.
    /// - The `children` of a node are cloned and processed recursively to avoid
    ///   borrow checker conflicts when iterating over them.
    ///
    /// # Usage
    ///
    /// This function is generally used to produce a display list from a hierarchical
    /// layout structure, enabling rendering of complex graphics or UI elements.
    ///
    /// ```
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let root_node = Rc::new(RefCell::new(LayoutNode::new()));
    /// let mut display_list: Vec<DrawText> = Vec::new();
    /// paint_tree(root_node, &mut display_list);
    /// ```
    pub fn paint_tree(node: Rc<RefCell<LayoutNode>>, mut display_list: &mut Vec<DrawText>)  {
        display_list.append(&mut node.borrow().paint());

        for child in node.borrow().children.clone()
        {
            Self::paint_tree(child.clone(), &mut display_list);
        }
    }

    /// ```rust
    /// Determines the layout mode for a given HTML node by analyzing its type and children.
    ///
    /// # Arguments
    /// * `layout_node` - A reference-counted, mutable `HtmlNode` object wrapped in `RefCell`.
    ///
    /// # Returns
    /// * `LayoutMode` - Either `Block` or `Inline`, based on the analysis of the node type and its children.
    ///
    /// # Behavior
    /// - For nodes of type `HtmlNodeType::Element`:
    ///     - If any child is an element and its tag is a block element (`BLOCK_ELEMENTS`), the function returns `Block`.
    ///     - If the node has children but none of them satisfy the above condition, it returns `Inline`.
    ///     - If the node has no children, it defaults to `Block`.
    /// - For nodes of type `HtmlNodeType::Text`, the function returns `Inline`.
    ///
    /// # Notes
    /// - `BLOCK_ELEMENTS` is assumed to be a predefined collection of block-level HTML tags.
    /// - This function uses interior mutability with `RefCell` to enable borrowing and modification of the `HtmlNode`.
    ///
    /// # Examples
    /// ```
    /// let html_node = Rc::new(RefCell::new(HtmlNode::new(HtmlNodeType::Element(Element::new("div")))));
    /// let mode = layout_mode(html_node.clone());
    /// assert_eq!(mode, LayoutMode::Block);
    /// ```
    ///
    /// ```
    /// let text_node = Rc::new(RefCell::new(HtmlNode::new(HtmlNodeType::Text("Hello".to_string()))));
    /// let mode = layout_mode(text_node.clone());
    /// assert_eq!(mode, LayoutMode::Inline);
    /// ```
    /// ```
    fn layout_mode(layout_node: Rc<RefCell<HtmlNode>>) -> LayoutMode {
        match layout_node.borrow().node_type {
            HtmlNodeType::Element(_) => {
                if layout_node.borrow().children.iter().any(|c| {
                    match &c.borrow().node_type {
                        HtmlNodeType::Element(ele) => {
                            BLOCK_ELEMENTS.contains(&&*ele.tag)
                        },
                        _ => false
                    }
                }) { Block}
                else if (!layout_node.borrow().children.is_empty()) {
                     Inline
                } else {
                    Block
                }
            }
            HtmlNodeType::Text(_) => {Inline}
        }
    }
}

enum LayoutMode {
    Inline,
    Block
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

impl std::fmt::Debug for LayoutNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutNodeType::Block(_) => write!(f, "Block"),
            LayoutNodeType::Document => write!(f, "Document"),
        }
    }
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
#[derive(Debug)]
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

/// A struct that assists in composing and configuring block layouts.
///
/// The `BlockComposer` struct is designed to facilitate the layout configuration
/// of a block by holding mutable references to the layout properties and their
/// related attributes. It provides a mechanism for modifying these properties
/// in a cohesive manner.
///
/// # Type Parameters
/// - `'a`: A lifetime parameter that ties the lifetime of borrowed data to the block composer.
///
/// # Fields
/// - `layout`: A mutable reference to a `BlockLayout` object that defines the structure
///   or arrangement of the block.
/// - `outer_position`: A mutable reference to an `Option<Vec2>` that represents the
///   outer position of the block. This can be used to specify or modify its position in
///   a layout space.
/// - `outer_size`: A mutable reference to an `Option<Vec2>` that represents the outer
///   size of the block. This allows the overall dimensions of the block to be set or adjusted.
///
/// # Example
/// ```rust
/// // Example usage of the BlockComposer struct
/// let mut layout = BlockLayout::new();
/// let mut position = None;
/// let mut size = None;
///
/// let mut composer = BlockComposer {
///     layout: &mut layout,
///     outer_position: &mut position,
///     outer_size: &mut size,
/// };
///
/// // The composer can now be used to modify the layout, position, and size.
/// ```
struct BlockComposer<'a> {
    layout: &'a mut BlockLayout,
    outer_position: &'a mut Option<Vec2>,
    outer_size: &'a mut Option<Vec2>,
}

impl<'a> BlockComposer<'a> {
    /// Updates the text layout or formatting based on the provided HTML-like tag.
    ///
    /// # Parameters
    /// - `tag`: A `String` representing the tag name to process. Supported tags include:
    ///   - `"i"`: Applies an italic font style.
    ///   - `"b"`: Applies a bold font weight.
    ///   - `"big"`: Increases the font size by 16/3 units.
    ///   - `"small"`: Decreases the font size by 8/3 units.
    ///   - `"br"`: Flushes the current line by triggering a line break.
    ///
    fn open_tag(&mut self, tag: String) {
        match tag.as_str() {
            "i" => {
                self.layout.font_style = "italic".into();
                self.update_font()
            },
            "b" => {
                self.layout.font_weight = "bold".into();
                self.update_font()
            },
            "big" => {
                self.layout.font_size += 16.0/3.0;
                self.update_font()
            },
            "small" => {
                self.layout.font_size -= 8.0/3.0;
                self.update_font()
            },
            "br" => {
                self.flush_line();
            }
            _ => {}
        }
    }

    /// Handles the closing of an HTML-like tag by updating the layout state accordingly.
    ///
    /// This method is responsible for reversing changes made to the layout state when
    /// a corresponding opening tag is closed. It supports tags such as `<i>`, `<b>`,
    /// `<big>`, `<small>`, and `<p>`. Different actions are taken depending on the tag:
    /// - `<i>`: Resets the font style.
    /// - `<b>`: Resets the font weight.
    /// - `<big>`: Decreases the font size.
    /// - `<small>`: Increases the font size.
    /// - `<p>`: Flushes the current line and moves the cursor down by a vertical step.
    ///
    /// # Parameters
    /// - `tag`: A `String` representing the closing tag (e.g., "i", "b", "big", "small", "p").
    ///
    /// # Behavior
    /// - For `<i>` and `<b>`, the font style and weight are reset respectively.
    /// - For `<big>` and `<small>`, the font size is adjusted relative to a predefined value.
    /// - For `<p>`, the current line is finalized and the cursor is moved down to the next line.
    /// - If the tag does not match any of the handled cases, the method does nothing.
    ///
    /// # Side Effects
    /// - May modify the `font_style`, `font_weight`, and `font_size` properties of the `layout`.
    /// - Calls `self.update_font()` to apply font-related changes.
    /// - For the `<p>` tag, calls `self.flush_line()` and modifies `layout.cursor_y`.
    ///
    /// # Examples
    /// ```rust
    /// let mut renderer = Renderer::new();
    /// renderer.close_tag("i".into());  // Resets italic style.
    /// renderer.close_tag("p".into());  // Moves to a new paragraph.
    /// renderer.close_tag("unknown".into());  // No action taken.
    /// ```
    ///
    /// # Note
    /// - `VSTEP` is assumed to be a predefined constant controlling the vertical spacing between lines.
    fn close_tag(&mut self, tag: String) {
        match tag.as_str() {
            "i" => {
                self.layout.font_style = "".into();
                self.update_font()
            },
            "b" => {
                self.layout.font_weight = "".into();
                self.update_font()
            },
            "big" => {
                self.layout.font_size -= 16.0/3.0;
                self.update_font()
            },
            "small" => {
                self.layout.font_size += 8.0/3.0;
                self.update_font()
            },
            "p" => {
                self.flush_line();
                self.layout.cursor_y += VSTEP;
            }
            _ => {}
        }
    }

    /// Adds a word to the current layout, ensuring proper alignment and wrapping based on the available space.
    ///
    /// # Parameters
    /// - `word`: A string slice (`&str`) representing the word to be added to the layout.
    ///
    /// # Behavior
    /// 1. Measures the width of the `word` using the font settings stored in the layout context.
    /// 2. Checks if adding the `word` to the current line would exceed the available width (`outer_size.x`).
    ///    - If it would exceed, calls `flush_line()` to finalize the current line and move to the next one.
    /// 3. Creates a `DrawText` object for the `word`, initialized with:
    ///    - The current cursor position (`cursor_x`) for the x-coordinate.
    ///    - A y-coordinate of `0.0` (default alignment).
    ///    - The precomputed galley (text layout information).
    /// 4. Adds the `DrawText` object to the current layout line.
    /// 5. Advances the `cursor_x` position by the width of the `word` (`text_width`) plus spacing between words (`space_width`).
    ///
    /// # Notes
    /// - This function relies on the layout system's `flush_line` method to handle wrapping when a word does not fit within the available width.
    /// - The word's color is hardcoded as `Color32::BLACK`, which can be modified as needed for dynamic color support.
    ///
    /// # Example
    /// ```
    /// some_layout.word("example");
    /// ```
    fn word(&mut self, word: &str) {
        let galley = self.layout.context.fonts_mut(|f|
            f.layout_no_wrap(word.to_string(), self.layout.font_id.clone(), Color32::BLACK));

        let text_width = galley.size().x;

        if self.layout.cursor_x + text_width > self.outer_size.unwrap().x {
            self.flush_line();
        }

        self.layout.line.push(DrawText {
            x: self.layout.cursor_x,
            y: 0.0,
            galley
        });
        self.layout.cursor_x += text_width + self.layout.space_width;
    }

    /// Flushes the current line of text from the internal layout structure to the display list
    /// for rendering. This method calculates the positioning of text elements within the line,
    /// adjusts their vertical alignment based on font ascent/descent values, and resets cursors
    /// after processing.
    ///
    /// ### Behavior:
    /// 1. If the current line is empty, the method returns immediately.
    /// 2. Calculates the maximum ascent and descent values for the line, ensuring proper vertical
    ///    alignment.
    /// 3. Determines the baseline Y-coordinate position based on the maximum ascent and current
    ///    cursor Y position.
    /// 4. Iterates through the line's text chunks to adjust their vertical position relative to
    ///    the calculated baseline and pushes their formatted representation to the display list.
    /// 5. Resets the cursor X position back to 0 and advances the cursor Y position to prepare for
    ///    the next line.
    /// 6. Clears the current line after processing.
    ///
    /// ### Panics:
    /// The method assumes non-empty `galley` rows and `glyphs` in the text layout structure.
    /// If these assumptions are violated, it could cause `unwrap` calls to panic.
    ///
    /// ### Example:
    /// ```ignore
    /// let mut renderer = Renderer::new();
    /// // Configure renderer's layout with some text.
    /// renderer.layout.line.push(...);
    /// renderer.flush_line();
    /// ```
    ///
    /// ### Notes:
    /// - The `1.25` multiplier applied to the ascent and descent margins provides additional vertical
    ///   spacing between lines for improved readability.
    /// - The method relies on an `outer_position` field for determining the outer coordinates of the
    ///   current layout block, which must be defined prior to calling `flush_line`.
    ///
    /// ### Fields Used:
    /// - `self.layout.line`: Stores the text elements in the current line.
    /// - `self.layout.cursor_x`: Tracks the X position of text addition.
    /// - `self.layout.cursor_y`: Tracks the Y position of text addition.
    /// - `self.layout.display_list`: A collection of `DrawText` objects representing text ready for
    ///   rendering.
    /// - `self.outer_position`: An optional field for the layout's outer positioning.
    fn flush_line(&mut self) {
        if self.layout.line.is_empty() {
            return;
        }

        let galleys = self.layout.line.iter().map(|l| l.galley.clone());
        let max_ascent = galleys.clone().map(|g| g.rows.first().unwrap()
            .row.glyphs.first().unwrap().font_ascent).into_iter()
            .reduce(f32::max)
            .unwrap_or(0.);
        let max_descent = galleys.map(|g| {
            let glyph = g.rows.first().unwrap().row.glyphs.first().unwrap();
            glyph.font_height - glyph.font_ascent
        }).into_iter()
            .reduce(f32::max)
            .unwrap_or(0.);

        let baseline = self.layout.cursor_y + 1.25 * max_ascent;

        for text in &mut self.layout.line {
            text.y = baseline - text.galley.rows.first().unwrap().row.glyphs.first().unwrap().font_ascent;
            self.layout.display_list.push(DrawText {
                x: text.x + self.outer_position.unwrap().x,
                y: text.y + self.outer_position.unwrap().y,
                galley: text.galley.clone()
            })
        }

        self.layout.cursor_x = 0.0;
        self.layout.cursor_y = baseline + 1.25 * max_descent;
        self.layout.line.clear();
    }

    /// Recursively processes an HTML-like structure represented by a tree of `HtmlNode` objects.
    ///
    /// This function traverses an `HtmlNode` tree structure in depth-first order.
    /// Depending on the type of each node (element or text), it performs specific actions.
    ///
    /// - For elements: It opens the tag (`open_tag` method), processes its children recursively,
    ///   and then closes the tag (`close_tag` method).
    /// - For text: It splits the text content into words and processes each word using the `word` method.
    ///
    /// # Arguments
    ///
    /// * `tree` - A reference-counted, mutable smart pointer to an `HtmlNode`,
    ///   representing the current node in the tree to process.
    ///
    /// # Node Types
    ///
    /// Nodes can be one of the following types:
    ///
    /// - `HtmlNodeType::Element`: Represents an HTML element with a tag and potentially child nodes.
    /// - `HtmlNodeType::Text`: Represents a text node containing raw text.
    ///
    /// # Example Workflow
    ///
    /// 1. If the node is an `Element`, extract its tag name and children and invoke
    ///    `open_tag(tag)` followed by recursive traversal of its children. Finally,
    ///    invoke `close_tag(tag)`.
    /// 2. If the node is `Text`, split the text into individual words and process each
    ///    word using the `word(word)` method.
    ///
    /// # Implementation Notes
    ///
    /// - The `tree` is traversed using an `Rc<RefCell<T>>` to allow shared ownership
    ///   while keeping the ability to mutate the node.
    /// - Borrowing of the `tree` node is carefully managed to prevent borrow conflicts
    ///   when traversing the tree recursively and working on node data.
    ///
    /// # Dependencies
    ///
    /// This function assumes the presence of the following methods in its scope:
    ///
    /// * `open_tag(tag: String)`: Handles the processing of opening an HTML tag.
    /// * `close_tag(tag: String)`: Handles the processing of closing an HTML tag.
    /// * `word(word: &str)`: Handles the processing of individual words in text nodes.
    ///
    /// # Parameters of the Enum `Action`
    ///
    /// * `ProcessElement`: Represents an HTML element and contains:
    ///     - `tag`: The name of the element tag.
    ///     - `children`: A list of child nodes.
    /// * `ProcessText`: Represents a text node and contains:
    ///     - `text`: The text content of the node.
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

    /// ```rust
    /// Updates the font settings and recalculates the space width for the layout.
    ///
    /// This function performs the following tasks:
    /// 1. Constructs a new font name by combining the font family, weight, and style attributes.
    /// 2. Updates the `font_id` of the layout with the newly constructed font name and specified font size.
    /// 3. Calculates the width of a single space character (`space_width`) using the layout context and
    ///    the updated font settings.
    ///
    /// ### Fields Utilized
    /// - `self.layout.font_family`: The font family to be used.
    /// - `self.layout.font_weight`: The font weight to be
    fn update_font(&mut self) {
        let font_name = format!("{}{}{}", self.layout.font_family, self.layout.font_weight, self.layout.font_style);
        self.layout.font_id = FontId::new(self.layout.font_size, FontFamily::Name(Arc::from(font_name.clone())));
        let space_galley = self.layout.context.fonts_mut(|f|
            f.layout_no_wrap(" ".to_string(), self.layout.font_id.clone(), Color32::BLACK));
        self.layout.space_width = space_galley.size().x;
    }
}
