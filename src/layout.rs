
use std::cell::RefCell;
use std::rc::Rc;
use crate::browser::{DrawCommand, DrawRect, DrawText};
use crate::node::{HtmlNode, HtmlNodeType};
use eframe::epaint::{Color32, FontFamily, FontId};
use egui::{Context, Galley, TextBuffer, Vec2};
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
    pub(crate) node: Rc<RefCell<HtmlNode>>,
    pub(crate) parent: Option<Rc<RefCell<LayoutNode>>>,
    children: Vec<Rc<RefCell<LayoutNode>>>,
    previous: Option<Rc<RefCell<LayoutNode>>>,
    pub(crate) content: LayoutNodeType,
    pub(crate) position: Option<Vec2>,
    pub(crate) size: Option<Vec2>,
    pub(crate) display_list: Rc<RefCell<Vec<DrawCommand>>>
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

    /// ```rust
    /// Creates a new `LayoutNode` representing a line, which is a child of the specified parent node.
    ///
    /// # Parameters
    /// - `parent`: A `Rc<RefCell<LayoutNode>>` representing the parent layout node.
    ///             This parent node is typically a block-level layout node.
    ///
    /// # Returns
    /// - A `Rc<RefCell<LayoutNode>>` representing the newly created line layout node.
    ///
    /// # Details
    /// This function initializes a new `LayoutNode` with the following properties:
    /// - Inherits the HTML node of the parent block (`parent.borrow().node.clone()`).
    /// - Sets its `parent` field to the provided parent node.
    /// - Initializes an empty list of `children`.
    /// - Sets the `previous` field to `None`, with the potential to add sibling line logic in the future.
    /// - Configures the `content` field using the `LineLayout` type, with `max_ascent` and `max_descent`
    ///   initialized to `0.0`.
    /// - Creates an empty `display_list` for rendering purposes.
    /// - Sets the `position` field to `Vec2::ZERO`, representing the line's relative position within the block.
    /// - Sets the `size` field to `Vec2::ZERO`, which can later be adjusted to represent correct layout dimensions.
    ///
    /// # Examples
    /// ```
    /// let parent = Rc::new(RefCell::new(LayoutNode::new_block(...)));
    /// let new_line_node = LayoutNode::new_line(parent);
    /// // The new_line_node is now ready to be added to the parent's list of children.
    /// ```
    /// ```
    pub fn new_line(node: Rc<RefCell<HtmlNode>>, parent: Rc<RefCell<LayoutNode>>) -> Rc<RefCell<LayoutNode>> {
        Rc::new(RefCell::new(Self {
            node, // Use the passed HTML node
            parent: Some(parent),
            children: Vec::new(),
            previous: None,
            content: LayoutNodeType::Line(LineLayout { max_ascent: 0.0, max_descent: 0.0 }),
            display_list: Rc::new(RefCell::new(Vec::new())),
            position: Some(Vec2::ZERO),
            size: Some(Vec2::ZERO),
        }))
    }

    pub(crate) fn tree_to_vec(tree: Rc<RefCell<LayoutNode>>, vec: &mut Vec<Rc<RefCell<LayoutNode>>>) -> &Vec<Rc<RefCell<LayoutNode>>> {
        vec.push(tree.clone());
        for child in tree.borrow().children.clone() {
            Self::tree_to_vec(child, vec);
        }

        return vec;

    }

    /// ```rust
    /// Creates a new `LayoutNode` instance to represent a textual element within a layout hierarchy.
    ///
    /// This function initializes the `LayoutNode` with the provided parent node, text layout,
    /// and size. The created node is wrapped in both `Rc` (to allow multiple ownership)
    /// and `RefCell` (to enable interior mutability).
    ///
    /// # Parameters
    ///
    /// - `parent`: A reference-counted and mutable reference (`Rc<RefCell<LayoutNode>>`)
    ///   to the parent `LayoutNode` of the newly created text node. It represents the
    ///   hierarchical relationship between nodes.
    /// - `text_layout`: A `TextLayout` instance that defines the visual and structural
    ///   representation of the text (e.g., font, style, alignment).
    /// - `size`: A `Vec2` struct specifying the dimensions of the text node (width and height).
    ///
    /// # Returns
    ///
    /// An `Rc<RefCell<LayoutNode>>` representing the newly created text node. This node
    /// is configured with the following properties:
    /// - References the `LayoutNode` of the parent node.
    /// - No children (an empty vector).
    /// - No previous sibling.
    /// - `content` set to `LayoutNodeType::Text` with the provided `text_layout`.
    /// - An empty display list, ready to be populated during rendering.
    /// - Starting position as `Vec2::ZERO`, relative to a line.
    /// - Specific size provided by the `size` parameter.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// // Assuming `LayoutNode`, `TextLayout`, and `Vec2` structs are defined:
    /// let parent_node = Rc::new(RefCell::new(LayoutNode::new_root()));
    /// let text_layout = TextLayout::new("Hello, world!", font, size);
    /// let text_node_size = Vec2::new(200.0, 50.0);
    ///
    /// let text_node = LayoutNode::new_text(parent_node.clone(), text_layout, text_node_size);
    ///
    /// // The `text_node` can now be used as part of the layout tree.
    /// ```
    ///
    /// Note: The caller is responsible for managing memory and ensuring
    /// that the parent node and text node are part of a consistent layout hierarchy.
    /// ```
    /// Changed: Now takes `node` explicitly
    pub fn new_text(node: Rc<RefCell<HtmlNode>>, parent: Rc<RefCell<LayoutNode>>, text_layout: TextLayout, size: Vec2) -> Rc<RefCell<LayoutNode>> {
        Rc::new(RefCell::new(Self {
            node, // Use the passed HTML node
            parent: Some(parent),
            children: Vec::new(),
            previous: None,
            content: LayoutNodeType::Text(text_layout),
            display_list: Rc::new(RefCell::new(Vec::new())),
            position: Some(Vec2::ZERO),
            size: Some(size),
        }))
    }

    /// Creates a new `LayoutNode` of type `Block` with the given parameters.
    ///
    /// # Parameters
    /// - `node`: An `Rc<RefCell<HtmlNode>>` representing the HTML node associated with this layout node.
    /// - `parent`: An optional `Rc<RefCell<LayoutNode>>` representing the parent layout node, or `None` if this is a root layout node.
    /// - `previous`: An optional `Rc<RefCell<LayoutNode>>` representing the previous sibling layout node, or `None` if there is no previous sibling.
    /// - `context`: A `Context` struct containing necessary information such as styling, configuration, or environment details for layout computations.
    ///
    /// # Returns
    /// Returns an `Rc<RefCell<LayoutNode>>` representing the newly created layout node.
    ///
    /// # LayoutNode Details
    /// - This function specifically initializes a node of type `Block`, making it suitable for block-level content.
    /// - The `BlockLayout` contains default font properties:
    ///   - `font_family`: Set to `"sans"`.
    ///   - `font_weight`, `font_style`: Set to empty strings.
    ///   - `font_size`: Set to `16.0`.
    /// - A default `FontId` is assigned, and `space_width` is initialized to `0.0`.
    /// - The `cursor_x` and `cursor_y` fields are initialized to `0.0`.
    /// - The `display_list` is initialized as an empty vector wrapped in an `Rc<RefCell<_>>`, allowing for dynamic updates to the graphical representation.
    ///
    /// # Usage
    /// This function is intended to construct layout nodes in a UI rendering engine or similar system, where each layout node can represent a piece of the rendered structure.
    ///
    /// ```rust
    /// let html_node = Rc::new(RefCell::new(HtmlNode::new("div")));
    /// let context = Context::default();
    /// let layout_node = LayoutNode::new_block(html_node, None, None, context);
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
                    font_id: FontId::default(),
                    space_width: 0.0,
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
                    let outer_node_ptr = node.clone();
                    // Re-borrow mutably only for the block work
                    let mut node_borrow = node.borrow_mut();

                    // Destructure to split borrows, allowing simultaneous access to fields
                    let LayoutNode {
                        ref mut content,
                        ref mut position,
                        ref mut size,
                        ref node,
                        ref mut children,
                        ..
                    } = *node_borrow;

                    if let LayoutNodeType::Block(block_layout) = content {

                        // 3. Create Composer with the split references
                        let mut composer = BlockComposer {
                            layout: block_layout,
                            children: children, // Pass the Vec<Rc> directly
                            parent_ptr: outer_node_ptr.clone(), // Pass the Rc<RefCell> (the original `node` variable) for parentage
                            block_html_node: node.clone(),      // Pass the HTML node Rc
                            outer_position: position,
                            outer_size: size,
                            current_line_nodes: Vec::new(),
                        };

                        composer.update_font();
                        composer.recurse(inner_node_ptr);
                        composer.flush_line();

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

    /// Generates a sequence of drawing commands (`Vec<DrawCommand>`) for rendering elements
    /// based on the layout node and its attributes, such as position, size, style, and type.
    ///
    /// # Returns
    /// A vector of `DrawCommand` objects representing the drawing instructions
    /// for the current layout node and its children (if applicable).
    ///
    /// # Behavior
    /// The function processes nodes of type `Document` and `Block` differently:
    ///
    /// ## Document Node
    /// * If the node's type is `LayoutNodeType::Document`, an empty vector is returned
    ///   as no specific rendering logic is associated with the document node itself.
    ///
    /// ## Block Node
    /// * If the node's type is `LayoutNodeType::Block`, the function generates drawing commands
    ///   based on the following:
    ///
    /// ### Background Color
    /// * Retrieves the `background-color` style from the node's associated styles.
    /// * If a valid color is found and has a non-zero alpha (i.e., not fully transparent),
    ///   a `DrawRect` command is created for rendering a rectangle that represents the background.
    /// * The rectangle's position and size are determined using the `position` and `size` properties
    ///   of the layout node.
    ///
    /// ### Text Contents
    /// * If the layout mode for the block is `Inline`, iterates over the `display_list` and generates
    ///   one or more `DrawText` commands for rendering text content.
    ///
    /// ### Layout Mode
    /// * Handles additional layout-specific behaviors, such as `Inline` or `Block`, although
    ///   the `Block` mode logic is currently not implemented in this function.
    ///
    /// ### Node Type
    /// * The function differentiates between `HtmlNodeType::Element` and `HtmlNodeType::Text`,
    ///   where the latter does not produce any specific drawing commands in this implementation.
    ///
    /// # Dependencies
    /// * The function depends on external modules such as `csscolorparser` for parsing CSS-compatible
    ///   color strings and converting them into `Color32`.
    /// * Drawing commands like `DrawRect` and `DrawText` are facilitated via the `DrawCommand` enum.
    ///
    /// # Parameters
    /// * `&self`: A reference to the current `LayoutNode` object that contains the type, position, size,
    ///   and other properties necessary for computing draw commands.
    ///
    /// # Notes
    /// * This function assumes that the `position` and `size` attributes of the layout node are
    ///   always present (`Some`). If they are `None`, it will likely result in a runtime panic when unwrapped.
    ///
    /// # Example Behavior
    /// For a block node with a background color of `#FF5733` and a valid position and size,
    /// the function will emit a `DrawRect` command to render a rectangle.
    /// If the block node contains inline text, it will emit `DrawText` commands for each text element.
    ///
    /// # Potential Improvements
    /// * Gracefully handle cases where position or size is `None` to avoid runtime panics.
    /// * Implement additional drawing logic for `Block` mode if required.
    /// * Add support for more styles and attributes (e.g., borders, shadows) in future enhancements.
    ///
    /// # Output Format
    /// The result is a list of drawing commands that downstream subsystems or renderers can consume
    /// to render the element onto a display or canvas.
    pub fn paint(&self, offset: Vec2) -> Vec<DrawCommand> {
        let mut cmds = Vec::<DrawCommand>::new();

        match &self.content {
            LayoutNodeType::Document => {},

            LayoutNodeType::Block(_blk) => {
                if let HtmlNodeType::Element(_ele) = &self.node.borrow().node_type {
                    let bgcolor = self.node.borrow().style.get("background-color")
                        .unwrap_or(&"transparent".to_string()).clone();

                    if let Ok(color_parse) = csscolorparser::parse(&bgcolor) {
                        if let Ok(color) = Color32::from_hex(&color_parse.to_css_hex()) {
                            if color.a() > 0 {
                                // Blocks are absolute, so we ignore 'offset' for the rect position
                                // and just use self.position.
                                let pos = self.position.unwrap_or(Vec2::ZERO);
                                let size = self.size.unwrap_or(Vec2::ZERO);

                                cmds.push(DrawCommand::DrawRect(DrawRect {
                                    top_left: pos.to_pos2(),
                                    bottom_right: (pos + size).to_pos2(),
                                    color,
                                }));
                            }
                        }
                    }
                }
            },

            LayoutNodeType::Line(_) => {
                // Lines are usually invisible containers, but if you wanted to draw
                // a border or debug box, you would use: offset + self.position
            },

            LayoutNodeType::Text(text_layout) => {
                // Text is relative. We must add the accumulated offset.
                // 'offset' passed here is (Block Abs Pos + Line Rel Pos).
                // We add self.position (Text Rel Pos) to get final screen coordinates.
                let final_pos =  self.position.unwrap_or(Vec2::ZERO);

                cmds.push(DrawCommand::DrawText(DrawText {
                    x: final_pos.x,
                    y: final_pos.y,
                    galley: text_layout.galley.clone(),
                }));
            }
        }
        cmds
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
    pub fn paint_tree(node: Rc<RefCell<LayoutNode>>, display_list: &mut Vec<DrawCommand>, accumulated_offset: Vec2) {
        // 1. Ask the node to paint itself at the given offset
        display_list.append(&mut node.borrow().paint(accumulated_offset));

        // 2. Calculate the offset for the children
        let node_borrow = node.borrow();

        // CRITICAL: Determine coordinate space
        // Blocks in your system currently store ABSOLUTE positions.
        // Lines and Text store RELATIVE positions.
        let child_offset = match node_borrow.content {
            // If this is a Block, its 'position' is absolute.
            // We reset the accumulator to this block's position for its children (Lines).
            LayoutNodeType::Block(_) => node_borrow.position.unwrap_or(Vec2::ZERO),

            // If this is a Line or Text, its position is relative.
            // We add its position to the incoming accumulator.
            _ => accumulated_offset + node_borrow.position.unwrap_or(Vec2::ZERO),
        };

        // 3. Recurse
        for child in &node_borrow.children {
            Self::paint_tree(child.clone(), display_list, child_offset);
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

/// ```rust
/// Represents the types of nodes in a layout tree.
///
/// This enum is used to differentiate between different types of layout nodes
/// that can exist in a document's layout structure. Each variant corresponds
/// to a specific type of layout node.
///
/// Variants:
///
/// - `Document`:
///   Represents the root of the layout tree, typically corresponding to the
///   entire document or container. This is the top-most level of a layout
///   structure.
///
/// - `Block(BlockLayout)`:
///   Represents a block-level container, such as a paragraph or a division.
///   It contains a `BlockLayout` that holds additional layout information
///   specific to block-level elements.
///
/// - `Line(LineLayout)`:
///   Represents a line-level layout element within a block. It contains a
///   `LineLayout` that encapsulates the attributes and structure of a
///   particular line in the layout.
///
/// - `Text(TextLayout)`:
///   Represents a text fragment or inline-level content within a line. It
///   contains a `TextLayout` that describes the layout details of the text,
///   such as its style, size, and content.
/// ```
#[derive(Debug)]
pub enum LayoutNodeType {
    Document,
    Block(BlockLayout),
    Line(LineLayout),
    Text(TextLayout),
}

/// ```rust
/// Represents the layout metrics for a line of text,
/// providing information about its vertical alignment.
///
/// # Fields
///
/// * `max_ascent` - The maximum ascent value of the line.
///   This represents the distance from the baseline to the highest point
///   of any glyph in the line. A larger value indicates a taller ascent.
///
/// * `max_descent` - The maximum descent value of the line.
///   This represents the distance from the baseline to the lowest point
///   of any glyph in the line. A larger value indicates a deeper descent.
///
/// # Debug
///
/// This struct derives the `Debug` trait, allowing it to be
/// formatted using the `{:?}` formatter for debugging purposes.
///
/// # Example
/// ```
/// let line_layout = LineLayout {
///     max_ascent: 12.5,
///     max_descent: 3.2,
/// };
///
/// println!("{:?}", line_layout);
/// ```
/// ```
#[derive(Debug)]
pub struct LineLayout {
    pub max_ascent: f32,
    pub max_descent: f32,
}


/// A structure representing a layout of text with associated styling and rendering information.
///
/// # Fields
///
/// * `galley` - An `Arc`-wrapped `Galley` instance that contains the text, its layout, and associated metadata.
///   The `Galley` object manages the text layout, including line-breaking logic and glyph positioning.
///
/// * `color` - A `Color32` value representing the color of the text. This determines how the text is rendered visually,
///   typically defined in RGBA format with 8 bits per channel.
///
/// # Derives
///
/// * `Debug` - Implements the Debug trait for easy formatting and debugging of the `TextLayout` structure.
///
/// # Example
/// ```
/// use std::sync::Arc;
/// use some_crate::{TextLayout, Galley, Color32};
///
/// let galley = Arc::new(Galley::new("Hello, world!"));
/// let text_color = Color32::from_rgba_unmultiplied(255, 255, 255, 255); // white
/// let text_layout = TextLayout {
///     galley,
///     color: text_color,
/// };
///
/// println!("{:?}", text_layout); // Prints the debug representation of the text layout
/// ```
#[derive(Debug)]
pub struct TextLayout {
    pub galley: Arc<Galley>,
    pub color: Color32,
}

/// Represents the layout properties of a text block.
///
/// The `BlockLayout` struct encapsulates various attributes related to the graphical representation
/// and rendering of a block of text, including font properties, cursor position, and additional
/// context information.
///
/// # Fields
///
/// * `font_family` - A `String` representing the font family used for the text (e.g., "Arial").
/// * `font_weight` - A `String` that specifies the weight of the font (e.g., "Bold", "Regular").
/// * `font_style` - A `String` defining the style of the font (e.g., "Italic", "Normal").
/// * `font_size` - A `f32` value that indicates the size of the font in points.
/// * `cursor_x` - A `f32` representing the X-coordinate of the cursor in the block's layout.
/// * `cursor_y` - A `f32` representing the Y-coordinate of the cursor in the block's layout.
/// * `context` - A `Context`, which holds additional information or settings relevant to the layout.
/// * `font_id` - A `FontId`, identifying the font used in the block layout.
/// * `space_width` - A `f32` value representing the width of a single space character, useful
///   for alignment and spacing calculations.
///
/// This struct is derived from the `Debug` trait, which allows for formatted output using the
/// `{:?}` formatter.
#[derive(Debug)]
pub struct BlockLayout {
    font_family: String,
    font_weight: String,
    font_style: String,
    font_size: f32,
    cursor_x: f32,
    cursor_y: f32,
    context: Context,
    font_id: FontId,
    space_width: f32,
}

/// ```rust
/// Represents a helper struct used to compose and organize blocks in a layout system.
///
/// The `BlockComposer` is primarily responsible for managing and manipulating the layout of
/// blocks and their respective child nodes within a rendering or UI layout engine. It maintains
/// references to the layout, block node, and temporary buffers required for efficiently
/// handling lines and child nodes.
///
/// # Type Parameters
/// - `'a`: A lifetime bound that ties certain references in the struct to their owners,
///         ensuring safety.
///
/// # Fields
/// * `layout`: A mutable reference to a `BlockLayout`.
///   This represents the layout context for the current block, which includes layout-specific
///   properties such as position and size.
///
/// * `block_node`: An `Rc<RefCell<LayoutNode>>` representing the block-level `LayoutNode`.
///   This node acts as the parent to which child nodes (e.g., lines of text) will be added.
///   The use of `Rc<RefCell<_>>` ensures shared ownership and interior mutability for the node.
///
/// * `outer_position`: A mutable reference to an optional `Vec2` representing the outer
///   position of the block. If set, this determines the block's absolute position in the layout
///   space.
///
/// * `outer_size`: A mutable reference to an optional `Vec2` representing the size of the block.
///   If set, this specifies the block's total dimensions, including padding or margins.
///
/// * `current_line_nodes`: A vector of `Rc<RefCell<LayoutNode>>` representing temporary storage
///   for the text nodes belonging to the current line. This buffer facilitates efficient handling
///   of nodes before they are finalized and incorporated into the layout structure.
/// ```
struct BlockComposer<'a> {
    layout: &'a mut BlockLayout,
    // CHANGED: We access the parent's children list directly via split borrow
    children: &'a mut Vec<Rc<RefCell<LayoutNode>>>,
    // We keep the parent struct strictly for reference/parent pointers, not for borrowing data
    parent_ptr: Rc<RefCell<LayoutNode>>,
    // We need the block's HTML node to create Line nodes
    block_html_node: Rc<RefCell<HtmlNode>>,

    outer_position: &'a mut Option<Vec2>,
    outer_size: &'a mut Option<Vec2>,
    current_line_nodes: Vec<Rc<RefCell<LayoutNode>>>,
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
    fn word(&mut self, word: &str, node: Rc<RefCell<HtmlNode>>) {
        let weight = node.borrow().style.get("font-weight").unwrap_or(&"normal".to_string()).clone();
        match weight.as_str() {
            "bold" => self.layout.font_weight = "bold".into(),
            _ => self.layout.font_weight = "".into(),
        }

        let style = node.borrow().style.get("font-style").unwrap_or(&"normal".to_string()).clone();
        match style.as_str() {
            "italic" => self.layout.font_style = "italic".into(),
            _ => self.layout.font_style = "".into(),
        };

        let size = node.borrow().style.get("font-size").unwrap_or(&"16px".to_string()).clone().replace("px", "").parse::<f32>().unwrap();
        self.layout.font_size = size;
        self.update_font();
        let color_parse = csscolorparser::parse(node.borrow().style.get("color").unwrap_or(&"black".to_string()).as_str());
        let color = Color32::from_hex(&*color_parse.unwrap().to_css_hex()).unwrap();


        let galley = self.layout.context.fonts_mut(|f|
            f.layout_no_wrap(word.to_string(), self.layout.font_id.clone(), color));
        let text_width = galley.size().x;
        let text_height = galley.size().y;
        // 3. Check for wrapping
        if self.layout.cursor_x + text_width > self.outer_size.unwrap().x {
            self.flush_line();
        }

        // 4. Create the TextLayout struct
        let text_content = TextLayout {
            galley,
            color,
        };

        // 5. Create the Text Node (Note: We don't have the Line parent yet,
        // so we use block_node temporarily or handle parentage when creating the Line)
        // For simplicity, we create the node now and will assign the correct parent in flush_line.
        let text_node = LayoutNode::new_text(
            node.clone(),
            self.parent_ptr.clone(),
            text_content,
            Vec2::new(text_width, text_height)
        );

        // Set the relative X position immediately
        text_node.borrow_mut().position = Some(Vec2::new(self.layout.cursor_x, 0.0));

        // 6. Add to pending buffer
        self.current_line_nodes.push(text_node);

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
        if self.current_line_nodes.is_empty() { return; }

        // 1. Calculate Metrics
        let mut max_ascent: f32 = 0.0;
        let mut max_descent: f32 = 0.0;

        for node in &self.current_line_nodes {
            if let LayoutNodeType::Text(txt) = &node.borrow().content {
                if let Some(row) = txt.galley.rows.first() {
                    let ascent = row.row.glyphs.first().map(|g| g.font_ascent).unwrap_or(0.0);
                    let height = row.row.glyphs.first().map(|g| g.font_height).unwrap_or(0.0);

                    if ascent > max_ascent { max_ascent = ascent; }

                    let descent = height - ascent;
                    if descent > max_descent { max_descent = descent; }
                }
            }
        }

        let line_height = (max_ascent + max_descent) * 1.25;
        let baseline_offset = max_ascent * 1.25;

        // 2. Get the Block's Absolute Position
        // We assume self.outer_position is already absolute (set in layout())
        let block_pos = self.outer_position.unwrap_or(Vec2::ZERO);

        // Calculate Line's Absolute Position
        // X = Block X
        // Y = Block Y + Current Cursor Y
        let line_abs_pos = Vec2::new(block_pos.x, block_pos.y + self.layout.cursor_y);

        // 3. Create Line Node with Absolute Position
        let line_node = LayoutNode::new_line(
            self.block_html_node.clone(),
            self.parent_ptr.clone()
        );

        line_node.borrow_mut().position = Some(line_abs_pos);
        line_node.borrow_mut().size = Some(Vec2::new(self.outer_size.unwrap().x, line_height));

        if let LayoutNodeType::Line(l) = &mut line_node.borrow_mut().content {
            l.max_ascent = max_ascent;
            l.max_descent = max_descent;
        }

        // 4. Update Children (Text Nodes) with Absolute Positions
        for text_node in &self.current_line_nodes {
            let mut txt = text_node.borrow_mut();

            // Parent pointer update
            txt.parent = Some(line_node.clone());

            // Get the relative X we stored temporarily in word()
            let relative_x = txt.position.unwrap().x;

            // Calculate Text Ascent for alignment
            let mut ascent = 0.0;
            if let LayoutNodeType::Text(t) = &txt.content {
                if let Some(row) = t.galley.rows.first() {
                    ascent = row.row.glyphs.first().map(|g| g.font_ascent).unwrap_or(0.0);
                }
            }

            // Calculate Final Absolute Position
            // X = Block X + Word Relative X
            // Y = Line Y + (Baseline Offset - Font Ascent)
            let abs_x = block_pos.x + relative_x;
            let abs_y = line_abs_pos.y + (baseline_offset - ascent);

            txt.position = Some(Vec2::new(abs_x, abs_y));
        }

        // Move children into the line
        line_node.borrow_mut().children = self.current_line_nodes.drain(..).collect();

        // Add Line to Block
        self.children.push(line_node);

        // Advance Cursor
        self.layout.cursor_x = 0.0;
        self.layout.cursor_y += line_height;
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
                    self.word(word, tree.clone());
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
