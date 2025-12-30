use std::cell::RefCell;
use std::rc::Rc;
use crate::node::{Element, HtmlNode, HtmlNodeType, Text};

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

    /// ```rust
    /// Adds a new tag to the current HTML tree structure.
    ///
    /// This method processes an HTML tag and updates the tree structure accordingly.
    /// If the tag starts with a `/` (indicating a closing tag), it pops the most recently
    /// added node from the `unfinished` stack and appends it as a child of the parent node.
    /// If the tag does not start with `/`, it creates a new `HtmlNode` with the specified
    /// tag and adds it to the `unfinished` stack.
    ///
    /// # Parameters
    /// - `tag`: A reference to a `String` representing the tag to be added. If it's a closing
    ///   tag (starts with '/'), it signifies the end of the current child node and completes its
    ///   association with the parent node.
    ///
    /// # Behavior
    /// - If a closing tag (`tag.starts_with('/')`) is encountered:
    ///   - The most recently added node is removed (popped) from the `unfinished` stack.
    ///   - The node is then treated as a completed child and added to its parent node's children list.
    ///   - If the stack is empty (unexpected closing tag), the method returns early without any modifications.
    /// - If an opening tag is encountered:
    ///   - A new `HtmlNode` is created with the given tag.
    ///   - The parent of this new node is set to the node currently at the top of the `unfinished` stack.
    ///   - The new node is wrapped in a `Rc<RefCell<_>>` and pushed onto the `unfinished` stack.
    ///
    /// # Panics
    /// - If this method is invoked when the `unfinished` stack is empty (and a non-closing tag is processed),
    ///   it will panic while attempting to access the last element of the stack.
    ///
    /// # Example
    /// ```rust
    /// let mut tree_builder = HtmlTreeBuilder::new();
    /// tree_builder.add_tag(&"html".to_string());
    /// tree_builder.add_tag(&"head".to_string());
    /// tree_builder.add_tag(&"/head".to_string());
    /// tree_builder.add_tag(&"body".to_string());
    /// tree_builder.add_tag(&"/body".to_string());
    /// tree_builder.add_tag(&"/html".to_string());
    /// ```
    /// In this example:
    /// - Opening tags `html`, `head`, and `body` are added as nested nodes to form a tree structure.
    /// - Closing tags `/head`, `/body`, and `/html` signal the completion of their corresponding nodes,
    ///   associating them with their parent nodes.
    /// ```
    fn add_tag(&mut self, tag: &String) {
        if tag.starts_with('!') {
            return;
        }
        if tag.starts_with('/') {
            if self.unfinished.len() == 1 {return;}
            match self.unfinished.pop() {
                None => {
                    panic!("No parent node found for closing tag");
                }
                Some(node) => {
                    let parent = self.unfinished.last_mut().unwrap();
                    parent.borrow_mut().children.push(Rc::clone(&node));
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
                HtmlNodeType::Element(Element{tag: tag.clone()}), parent);
            self.unfinished.push(Rc::new(RefCell::new(node)));
        }
    }

    fn finish(&mut self) -> Rc<RefCell<HtmlNode>> {
        while self.unfinished.len() > 1 {
            let node = self.unfinished.pop().unwrap();
            let parent = self.unfinished.last_mut().unwrap();
            parent.borrow_mut().children.push(Rc::clone(&node));
        }
        self.unfinished.pop().unwrap()
    }
}

