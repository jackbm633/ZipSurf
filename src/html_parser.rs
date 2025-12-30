use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::node::{Element, HtmlNode, HtmlNodeType, Text};

/// ```
/// A constant array that defines a list of HTML void elements (self-closing tags).
///
/// Void elements are HTML elements that do not have closing tags and cannot contain any child elements.
/// These elements are self-contained and are used for embedding resources or standalone content.
///
/// The array consists of the following elements:
/// - `"area"`: Defines a clickable area within an image map.
/// - `"base"`: Specifies the base URL for all relative URLs in the document.
/// - `"br"`: Inserts a line break.
/// - `"col"`: Specifies column properties for `<colgroup>` elements in tables.
/// - `"embed"`: Embeds external content, such as multimedia or applications.
/// - `"hr"`: Creates a horizontal rule or thematic break.
/// - `"img"`: Embeds an image into the document.
/// - `"input"`: Represents an input control for user interaction.
/// - `"link"`: Specifies a relationship between the document and an external resource, typically used to link stylesheets.
/// - `"meta"`: Provides metadata about the document, such as character encoding or viewport settings.
/// - `"param"`: Defines parameters for plugins or embedded objects.
/// - `"source"`: Specifies multiple media resources for `<audio>` or `<video>` elements.
/// - `"track"`: Provides text tracks for `<video>` or `<audio>` elements, such as subtitles or captions.
/// - `"wbr"`: Suggests a line break opportunity.
///
/// # Usage
/// This constant is useful when processing or validating HTML content, ensuring proper handling of self-closing tags.
///
/// # Example
/// ```
/// if VOID_TAGS.contains(&"img") {
///     println!("'img' is a void HTML element!");
/// }
/// ```
/// ```
const VOID_TAGS: [&str; 14] = ["area", "base", "br", "col", "embed", "hr", "img", "input", "link",
    "meta", "param", "source", "track", "wbr"];


/// ```rust
/// A simple HTML parser structure designed to hold and manipulate the body content of an HTML document.
///
/// # Fields
///
/// * `body` - A `String` representing the inner HTML content or the body of an HTML document.
///
/// # Examples
///
/// ```rust
/// let parser = HtmlParser {
///     body: "<div>Hello, World!</div>".to_string(),
/// };
///
/// println!("{}", parser.body); // Outputs: <div>Hello, World!</div>
/// ```
/// struct
pub struct HtmlParser {
    pub(crate) body: String,
    pub(crate) unfinished: Vec<Rc<RefCell<HtmlNode>>>
}

impl HtmlParser {
    /// ```rust
    /// Parses the `body` field of the struct and converts it into a collection of `HtmlNode` objects.
    ///
    /// This method processes an HTML-like string stored in the `body` field of the struct,
    /// and identifies both text and element nodes. The parsing logic detects whether the
    /// current context is inside an HTML tag or outside (plain text) by processing characters
    /// one by one.
    ///
    /// Different cases are handled during parsing:
    /// - Characters between `<` and `>` are treated as part of an element (tag).
    /// - Characters outside `<` and `>` are treated as text.
    /// - If the `buffer` stores any text or tag information before a new state begins,
    ///   it is pushed to the `output` as a new `HtmlNode`.
    ///
    /// # Node Types
    /// - `HtmlNodeType::Text`: Created when plain text (outside of tags) is processed.
    /// - `HtmlNodeType::Element`: Created when an element/tag is processed.
    ///
    /// # Fields Involved
    /// - `buffer`: Temporarily holds the text or tag content being processed.
    /// - `in_tag`: A flag determining whether the current position is inside a tag.
    /// - `output`: Stores the parsed `HtmlNode` objects that represent the structured
    ///   content of the `body`.
    ///
    /// # Notes
    /// - If the `body` ends with plain text that is not followed by a tag, the method ensures
    ///   the remaining text in the `buffer` is pushed to the `output` as a `HtmlNodeType::Text`.
    /// - The behavior assumes the input is well-formed and does not explicitly handle errors
    ///   like mismatched or incomplete tags.
    ///
    /// # Example (Pseudo Code)
    /// Given an input like:
    /// ```html
    /// <div>Hello World</div>
    /// ```
    /// - The method creates an `HtmlNode` for the opening `<div>`.
    /// - It creates a `Text` node for the content `Hello World`.
    /// - It creates an `HtmlNode` for the closing `</div>`.
    ///
    /// No explicit return value is defined in the current implementation.
    /// ```
    pub(crate) fn parse(&mut self) -> Rc<RefCell<HtmlNode>> {
        let mut buffer = String::new();
        let mut in_tag = false;
        let mut chars: Vec<_> = self.body.chars().collect();
        let mut iter = chars.iter();
        while let Some(c) = iter.next() {
            match c {
                '<' => {
                    in_tag = true;
                    if !buffer.is_empty() {
                        self.add_text(&buffer);
                    }
                    buffer.clear();
                }
                '>' => {
                    in_tag = false;
                    self.add_tag(&buffer);
                    buffer.clear();
                },
                _ => {
                    buffer.push(*c);
                }
            }
        }
        if !in_tag && !buffer.is_empty() {
            self.add_text(&buffer);
        }
        self.finish()
    }

    fn add_text(&mut self, text: &String)  {
        if text.trim().is_empty() {return;}
        match self.unfinished.last_mut() {
            None => {
                panic!("No parent node found for text node");
            }
            Some(parent) => {
                let node = HtmlNode::new(HtmlNodeType::Text(Text{text: text.clone()}),
                                         Some(Rc::clone(&parent)));
                parent.borrow_mut().children.push(Rc::new(RefCell::new(node)));
            }
        }
    }

    /// Adds an HTML tag to the current structure being built.
    ///
    /// # Parameters
    /// - `tag`: A reference to the string representing the HTML tag to be added.
    ///
    /// # Behavior
    /// This method handles three main cases based on the provided tag:
    ///
    /// 1. **Ignorable Tags**:
    ///    - If the tag starts with a `!`, the method immediately returns without making modifications.
    ///      These tags are considered ignorable.
    ///
    /// 2. **Closing Tags**:
    ///    - If the tag starts with a `/`, it indicates a closing tag. The method attempts to:
    ///        - Ensure there is at least one unfinished node. If no parent node exists and `unfinished`
    ///          is empty or cannot safely close a tag, it will either return or panic.
    ///        - Pop the most recent unfinished node, fetch its parent (if present), and attach the
    ///          popped node as its child.
    ///
    /// 3. **Void Tags**:
    ///    - If the tag is present in `VOID_TAGS`, it creates a new node representing the void tag:
    ///        - The method checks for the latest parent node in the `unfinished` stack.
    ///        - If no parent exists, it panics. Otherwise, a new `HtmlNode` is created and added
    ///          to the parent's children.
    ///
    /// 4. **Other Tags**:
    ///    - For standard tags that neither start with `/` nor `!` and are not void tags:
    ///        - The method determines the parent (if any) of the new tag.
    ///        - A new `HtmlNode` is created with the given tag and added to the `unfinished` stack
    ///          for further processing of its children or closure.
    ///
    /// # Panics
    /// - When attempting to add a closing tag and no parent node exists in `unfinished`.
    /// - When attempting to add a void tag and no parent node exists in `unfinished`.
    ///
    fn add_tag(&mut self, tag: &String) {
        if tag.starts_with('!') {
            return;
        }
        if tag.starts_with('/') {
            if self.unfinished.len() == 1 { return; }
            match self.unfinished.pop() {
                None => {
                    panic!("No parent node found for closing tag");
                }
                Some(node) => {
                    let parent = self.unfinished.last_mut().unwrap();
                    parent.borrow_mut().children.push(Rc::clone(&node));
                }
            }
        } else if VOID_TAGS.contains(&tag.as_str()) {
             match self.unfinished.last_mut() {
                None => {
                    panic!("No parent node found for void tag");
                }
                Some(parent) => {
                    let node = HtmlNode::new(
                        HtmlNodeType::Element(get_attributes(tag)), Some(Rc::clone(&parent)));
                    parent.borrow_mut().children.push(Rc::new(RefCell::new(node)));
                }
            }
        } else {
            let parent = match self.unfinished.is_empty() {
                false => {
                    Some(self.unfinished.last().unwrap().clone())
                }
                true => {None}
            };
            let node = HtmlNode::new(
                HtmlNodeType::Element(get_attributes(tag)), parent);
            self.unfinished.push(Rc::new(RefCell::new(node)));
        }
    }

    /// ```rust
    /// Finalizes the construction of an HTML tree by collapsing all unfinished nodes
    /// into a single structured tree and returns the root node.
    ///
    /// This method works by iteratively popping nodes from the `unfinished` stack,
    /// attaching each node as a child to its parent (the last node in the stack),
    /// until only one root node remains. The root node is then returned.
    ///
    /// # Returns
    ///
    /// A reference-counted `Rc<RefCell<HtmlNode>>` representing the root of the completed HTML tree.
    ///
    /// # Panics
    ///
    /// This function will panic if the `unfinished` stack is empty when it attempts
    /// to `pop`. It assumes that there is at least one node in the `unfinished` stack
    /// when it is called.
    /// ```
    fn finish(&mut self) -> Rc<RefCell<HtmlNode>> {
        while self.unfinished.len() > 1 {
            let node = self.unfinished.pop().unwrap();
            let parent = self.unfinished.last_mut().unwrap();
            parent.borrow_mut().children.push(Rc::clone(&node));
        }
        self.unfinished.pop().unwrap()
    }


}

/// ```
/// Parses a string representation of an HTML element and extracts the tag name
/// along with its attributes into an `Element` struct.
///
/// # Arguments
///
/// * `text` - A string slice representing an HTML element-like string.
///   - The first word is treated as the tag name.
///   - Subsequent words are treated as attributes in either `key=value` pairs or
///     standalone keys.
///
/// # Returns
///
/// Returns an `Element` struct containing the tag name and a `HashMap`
/// of attribute keys and values. The tag name and attribute keys are
/// converted to lowercase for consistency. Quotation marks around attribute
/// values are trimmed (if present).
///
/// # Behavior
///
/// - Attributes in the form `key=value` will be parsed into corresponding
///   key-value pairs. Quotation marks around `value` are removed.
/// - Attributes without `=` will be added as keys with an empty string as the value.
/// - If `text` is empty or improperly formatted, it assumes the first word is the tag name
///   and remaining words are attributes.
///
/// # Example
///
/// ```
/// let text = "img src='image.jpg' alt=\"Picture\" readonly";
/// let element = get_attributes(text);
///
/// assert_eq!(element.tag, "img");
/// assert_eq!(element.attributes.get("src").unwrap(), "image.jpg");
/// assert_eq!(element.attributes.get("alt").unwrap(), "Picture");
/// assert_eq!(element.attributes.get("readonly").unwrap(), "");
/// ```
///
/// # Notes
///
/// - The function assumes the input will follow an HTML-like structure.
/// - Improperly formatted input may result in incorrect parsing outcomes.
///
/// # Dependencies
///
/// This function requires the `std::collections::HashMap` crate for
/// attribute storage.
///
/// # Errors
///
/// No explicit error handling is implemented; malformed input may lead
/// to unexpected behavior, such as panics or invalid attribute values.
///
/// ```
/// ```
fn get_attributes(text: &str) -> Element {
    let parts: Vec<&str> = text.split_whitespace().collect();
    let tag = parts[0].to_lowercase();
    let mut attributes = HashMap::<String, String>::new();
    for attrpair in parts[1..].iter() {
        if attrpair.contains('=') {
            let kv: Vec<&str> = attrpair.split('=').collect();
            let key = kv[0].to_string();
            let mut value = kv[1].to_string();
            if value.len() > 2 && (value.starts_with('\'') || value.starts_with('"')) {
                value = value.trim_matches(|c| c == '\'' || c == '"').parse().unwrap();
            }
            attributes.insert(key.to_lowercase(), value);
        } else {
            attributes.insert(attrpair.to_string().to_lowercase(), "".to_string());
        }
    }

    Element{tag: tag.into(),
        attributes}
}

