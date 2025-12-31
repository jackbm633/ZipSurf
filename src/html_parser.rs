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

/// A constant array of HTML tag names commonly used within the `<head>` section of a document.
///
/// # Tags Included:
/// - `"title"`: Specifies the title of the document, shown in the browser's title bar or tab.
/// - `"meta"`: Provides metadata about the document, such as character set, viewport settings, or descriptions.
/// - `"link"`: Defines relationships to external resources, like stylesheets or icons.
/// - `"style"`: Contains internal CSS styles to be applied to the document.
/// - `"script"`: Embeds or references JavaScript code for dynamic behavior.
/// - `"noscript"`: Specifies alternate content for users with JavaScript disabled.
/// - `"base"`: Defines a base URL for relative URLs in the document.
///
/// # Usage:
/// This array can be used for validating or generating `<head>` elements in HTML parsing, rendering,
/// or manipulation tasks.
///
/// # Example:
/// ```rust
/// for tag in HEAD_TAGS.iter() {
///     println!("Supported head tag: {}", tag);
/// }
/// ```
const HEAD_TAGS: [&str; 7] = ["title", "meta", "link", "style", "script", "noscript", "base"];
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
        let chars: Vec<_> = self.body.chars().collect();
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
        self.implicit_tags(None);
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

    /// Adds a tag to the HTML document structure being built.
    ///
    /// This function processes a given tag by analyzing its type and
    /// appropriately modifies the internal state of the structure. It handles
    /// standard, void, and closing tags while maintaining a stack of
    /// unfinished nodes.
    ///
    /// # Parameters
    /// - `tag`: A string slice representing the HTML tag to be processed.
    ///
    /// # Behavior
    /// 1. Parses the tag using the `get_attributes` function to extract its attributes
    ///    and determine its type.
    /// 2. If the tag starts with `!` (e.g., an HTML comment or doctype), no further processing is done, and the function returns early.
    /// 3. If the tag starts with `/` (a closing tag):
    ///     - If there is only one node in the `unfinished` stack, the function returns.
    ///     - If a parent node exists in the `unfinished` stack, the current node is popped
    ///       and added as a child of its parent.
    ///     - If no parent node is found, the function panics with an error message.
    /// 4. If the tag belongs to the `VOID_TAGS` set (e.g., `<img>`, `<br>`), it does the following:
    ///     - Ensures there is a current parent node in the `unfinished` stack.
    ///     - Creates a new `HtmlNode` for the void tag and appends it as a child to the current parent.
    ///     - Panics if no parent node is found.
    /// 5. For standard (non-void, non-closing) tags:
    ///     - Determines the parent node from the `unfinished` stack, if one exists.
    ///     - Creates a new `HtmlNode` for the tag and pushes it onto the `unfinished` stack.
    ///
    /// # Errors
    /// - Panics if a closing tag is processed without an appropriate parent node in the `unfinished` stack.
    /// - Panics if a void tag is processed without a parent node in the `unfinished` stack.
    ///
    /// # Notes
    /// - The function utilizes the `implicit_tags` method with the current tag for specific behavior
    ///   customization.
    /// - The `VOID_TAGS` set is used as a reference for identifying self-closing tags.
    ///
    /// # Example
    /// ```
    /// let mut parser = HtmlParser::new();
    /// parser.add_tag("<div>");
    /// parser.add_tag("<img src='image.png'>");
    /// parser.add_tag("</div>");
    /// ```
    ///
    /// The above example constructs an HTML structure equivalent to:
    /// ```html
    /// <div>
    ///     <img src="image.png" />
    /// </div>
    fn add_tag(&mut self, tag: &str) {
        let element = get_attributes(tag);

        if element.tag.starts_with('!') {
            return;
        }

        self.implicit_tags(Some(&*element.tag));

        if element.tag.starts_with('/') {
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
        } else if VOID_TAGS.contains(&element.tag.as_str()) {
             match self.unfinished.last_mut() {
                None => {
                    panic!("No parent node found for void tag");
                }
                Some(parent) => {
                    let node = HtmlNode::new(
                        HtmlNodeType::Element(element), Some(Rc::clone(&parent)));
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
                HtmlNodeType::Element(element), parent);
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
        if self.unfinished.is_empty() {
            self.implicit_tags(None);
        }
        while self.unfinished.len() > 1 {
            let node = self.unfinished.pop().unwrap();
            let parent = self.unfinished.last_mut().unwrap();
            parent.borrow_mut().children.push(Rc::clone(&node));
        }
        self.unfinished.pop().unwrap()
    }

    /// Handles the insertion of implicit tags when parsing an HTML document structure.
    /// This function ensures that certain structural elements, such as `<html>`, `<head>`,
    /// and `<body>`, are added where necessary to conform to standard HTML rules.
    ///
    /// # Arguments
    ///
    /// * `tag` - An optional reference to a string slice representing the currently
    ///   processed tag. If `tag` is `None`, this function determines what adjustments
    ///   to make based on the state of the `unfinished` stack alone.
    ///
    /// # Behavior
    ///
    /// This function implements the following logic:
    ///
    /// 1. Iterates over the `unfinished` stack (which represents the hierarchy of open
    ///    tags being parsed) to extract the tag names of open elements, skipping text
    ///    nodes.
    /// 2. Constructs a list of open tags (`open_tags`) to determine the current parsing context.
    /// 3. Based on the HTML parsing context and the incoming tag:
    ///    - If no tags are open and the incoming tag is not "html", it adds a `<html>` tag.
    ///    - If only the `<html>` tag is open and the incoming tag requires either a `<head>`
    ///      or `<body>` context (depending on the tag), it adds the appropriate tag.
    ///    - If both `<html>` and `<head>` tags are open and the tag should not appear
    ///      within `<head>`, it closes the `<head>` tag by adding a `</head>` tag.
    /// 4. Repeats the adjustment logic in a loop until no further implicit tags need to be added.
    ///
    /// # Notes
    ///
    /// * This function relies on the presence of `unfinished`, a stack-like structure
    ///   of open HTML nodes being processed, and `HEAD_TAGS`, a predefined set of tags
    ///   that are valid within the `<head>` element.
    /// * Invokes `self.add_tag(tag)` to programmatically add the necessary implicit tags
    ///   to maintain the document's structure.
    ///
    /// # Example
    ///
    /// Assume `HEAD_TAGS = ["title", "meta", "link", "style"]`.
    /// When parsing the following snippet:
    ///
    /// ```html
    /// <title>My Page</title>
    /// <p>Hello World!</p>
    /// ```
    ///
    /// The function will implicitly add `<html>`, `<head>`, and `<body>` as follows:
    ///
    /// ```html
    /// <html>
    ///   <head>
    ///     <title>My Page</title>
    ///   </head>
    ///   <body>
    ///     <p>Hello World!</p>
    ///   </body>
    /// </html>
    /// ```
    ///
    /// This mechanism ensures the document's structure adheres to HTML standards.
    fn implicit_tags(&mut self, tag: Option<&str>) {
        loop {
            let open_tags = self.unfinished.iter().map(|n| {
                match &n.borrow().node_type {
                    HtmlNodeType::Element(ele) => {
                        Some(ele.tag.clone())
                    }
                    HtmlNodeType::Text(_) => {None}
                }
            }).filter(|t| t.is_some())
                .map(|t| t.unwrap())
                .collect::<Vec<_>>();

            if open_tags.is_empty() && tag.is_some() && tag.unwrap() != "html" {
                self.add_tag("html");
            } else if open_tags == ["html"] && tag.is_some() && match tag.unwrap() {"head" | "body" | "/html" => false, _ => true} {
                if HEAD_TAGS.contains(&tag.unwrap()) {
                    self.add_tag("head");
                } else {
                    self.add_tag("body");
                }
            } else if open_tags == ["html", "head"] && tag.is_some() && !HEAD_TAGS.contains(&tag.unwrap()) && tag.unwrap() != "/head" {
                self.add_tag("/head");
            } else {
                break;
            }
        }
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

