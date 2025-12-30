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

/// ```rust
/// Represents a node in an HTML DOM tree structure.
///
/// The `HtmlNode` struct is used to model elements in an HTML document,
/// enabling the representation of hierarchical relationships between nodes,
/// such as parent-child and sibling relationships.
///
/// # Fields
///
/// * `node_type` - Specifies the type of the HTML node. This could represent
///   an element node, a text node, or other node types as defined by the `HtmlNodeType` enum.
///
/// * `children` - A vector of references to the child nodes of this `HtmlNode`.
///   Each child node is wrapped in an `Rc<RefCell<HtmlNode>>` to enable shared ownership
///   and interior mutability, allowing for dynamic updates to the DOM structure.
///
/// * `parent` - An optional reference to the parent of this `HtmlNode`. This field
///   is also wrapped in an `Rc<RefCell<HtmlNode>>` to facilitate shared
///   ownership and mutable access. If set to `None`, this node is the root of the tree.
///
/// # Usage
///
/// The `HtmlNode` struct can be used to model and manipulate a tree of HTML
/// elements programmatically. It is especially suitable for use cases such as
/// building custom HTML parsers, rendering engines, or other tools that operate
/// on structured HTML data.
///
/// # Example
/// ```rust
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let root = Rc::new(RefCell::new(HtmlNode {
///     node_type: HtmlNodeType::Element(String::from("div")),
///     children: vec![],
///     parent: None,
/// }));
///
/// let child = Rc::new(RefCell::new(HtmlNode {
///     node_type: HtmlNodeType::Element(String::from("span")),
///     children: vec![],
///     parent: Some(Rc::clone(&root)),
/// }));
///
/// root.borrow_mut().children.push(Rc::clone(&child));
/// ```
/// ```
pub struct HtmlNode {
    pub(crate) node_type: HtmlNodeType,
    pub(crate) children: Vec<Rc<RefCell<HtmlNode>>>,
    pub(crate) parent: Option<Rc<RefCell<HtmlNode>>>
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
    pub(crate) fn new(node_type: HtmlNodeType, parent: Option<Rc<RefCell<HtmlNode>>>) -> HtmlNode {
        HtmlNode {
            node_type,
            children: vec![],
            parent,
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
