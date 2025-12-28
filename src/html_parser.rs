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
struct HtmlParser {
    body: String,
    unfinished: Vec<HtmlNode>
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
    fn parse(&self) {
        let mut output: Vec<HtmlNode> = Vec::new();
        let mut buffer = String::new();
        let mut in_tag = false;
        let mut chars = self.body.chars();
        while let Some(c) = chars.next() {
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
                    buffer.push(c);
                }
            }
        }
        if !in_tag && !buffer.is_empty() {
            self.add_text(&buffer);
        }

    }

    fn add_text(&self, text: &String)  {
        todo!()
    }

    fn add_tag(&self, p0: &String) {
        todo!()
    }
}

