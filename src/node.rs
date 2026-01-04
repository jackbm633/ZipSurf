use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Represents a node in a document structure, which can either be a tag or plain text.
///
/// This enum is used to model the different types of elements that can exist in a document
/// or similar data structure.
///
/// # Variants
///
/// * `Tag(Tag)`
///   Represents an element with a tag, such as an HTML or XML element. The `Tag` type
///   encapsulates information related to the tag, such as the tag name, attributes, and children.
///
/// * `Text(Text)`
///   Represents a plain text element. The `Text` type contains the actual string data
///   for the text content.
#[derive(Clone, Debug)]
pub enum HtmlNodeType {
    Element(Element),
    Text(Text)
}

/// Represents a node in an HTML document tree structure.
///
/// Each `HtmlNode` object holds information about its type, hierarchical
/// relationships (parent and children), and inline style attributes.
///
/// # Fields
///
/// * `node_type` - The type of the HTML node, which determines its role
///   in the document (e.g., element, text node, comment, etc.).
///
/// * `children` - A vector of child nodes, stored as [`Rc`] wrapped in
///   [`RefCell`] for shared ownership and interior mutability.
///
/// * `parent` - An optional reference to the parent node, also stored
///   as an [`Rc`] wrapped in [`RefCell`]. If the node is the root, this
///   field will be `None`.
///
/// * `style` - A [`HashMap`] storing the inline CSS styles associated
///   with the node. The keys are CSS property names (as strings), and
///   the values are the corresponding property values.
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// let mut style = HashMap::new();
/// style.insert(String::from("color"), String::from("red"));
///
/// let root = Rc::new(RefCell::new(HtmlNode {
///     node_type: HtmlNodeType::Element(String::from("div")),
///     children: vec![],
///     parent: None,
///     style,
/// }));
/// ```
///
/// # Note
///
/// This structure supports shared ownership of nodes and allows mutable
/// access through the use of [`Rc`] and [`RefCell`]. Care should be taken
/// to avoid cyclic references when creating parent-child relationships.
pub struct HtmlNode {
    pub(crate) node_type: HtmlNodeType,
    pub(crate) children: Vec<Rc<RefCell<HtmlNode>>>,
    pub(crate) parent: Option<Rc<RefCell<HtmlNode>>>,
    pub(crate) style: HashMap<String, String>
}

impl std::fmt::Debug for HtmlNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HtmlNode")
            .field("node_type", &self.node_type)
            .field("children", &self.children)
            .finish()
    }
}

impl HtmlNode {
    /// Creates a new `HtmlNode` instance with the specified node type and optional parent.
    ///
    /// # Parameters
    /// - `node_type`: The type of the HTML node, defined by the `HtmlNodeType` enum.
    /// - `parent`: An optional reference-counted, mutable reference to the parent `HtmlNode`.
    ///    - Use `None` if the node has no parent (e.g., it is the root node).
    ///
    /// # Returns
    /// A new instance of `HtmlNode` with:
    /// - The specified `node_type`.
    /// - An empty list of children (`children`).
    /// - The provided parent node reference (`parent`).
    /// - An empty style map (`style`), which can later be used to store CSS-like properties.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    /// use std::collections::HashMap;
    ///
    /// let parent_node = Rc::new(RefCell::new(HtmlNode::new(HtmlNodeType::Div, None)));
    /// let child_node = HtmlNode::new(HtmlNodeType::Span, Some(parent_node.clone()));
    /// ```
    pub(crate) fn new(node_type: HtmlNodeType, parent: Option<Rc<RefCell<HtmlNode>>>) -> HtmlNode {
        HtmlNode {
            node_type,
            children: vec![],
            parent,
            style: HashMap::new()
        }
    }
}

/// A struct representing a tag.
///
/// The `Tag` struct is used to encapsulate a string-based tag.
/// It can be used to represent labels, keywords, or identifiers in various contexts.
///
/// # Fields
///
/// * `tag` - A `String` representing the value of the tag.
///
/// # Examples
///
/// ```
/// let my_tag = Tag {
///     tag: String::from("example"),
/// };
/// println!("{}", my_tag.tag); // Output: example
/// ```
/// struct
#[derive(Clone)]
#[derive(Debug)]
pub struct Element {
    pub(crate) tag: String,
    pub attributes: HashMap<String, String>,
}

/// ```rust
/// Struct `Text`
///
/// Represents a text entity containing a single field `text`.
///
/// # Fields
///
/// * `text` - A `String` that holds the content of the text.
///
/// # Example
///
/// ```
/// let my_text = Text {
///     text: String::from("Hello, world!"),
/// };
///
/// println!("{}", my_text.text); // Outputs: Hello, world!
/// ```
/// ```
#[derive(Clone)]
#[derive(Debug)]
pub struct Text {
    pub(crate) text: String
}
