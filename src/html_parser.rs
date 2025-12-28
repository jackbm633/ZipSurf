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
}

impl HtmlParser {
    fn parse(&self) {
        let mut output: Vec<HtmlNode> = Vec::new();
        let mut buffer = String::new();
        let mut in_tag = false;
        let mut chars = self.body.chars();
        while let Some(c) = chars.next() {
            match c {
                '<' => {
                    in_tag = true;
                    if !buffer.is_empty() {output.push(
                        HtmlNode::new(HtmlNodeType::Text(Text {text: buffer.clone()})))
                    }
                    buffer.clear();
                }
                '>' => {
                    in_tag = false;
                    output.push(HtmlNode::new(HtmlNodeType::Element(Element {tag: buffer.clone()})));
                    buffer.clear();
                },
                _ => {
                    buffer.push(c);
                }
            }
        }
        if !in_tag && !buffer.is_empty() {
            output.push(HtmlNode::new(HtmlNodeType::Text(Text {text: buffer.clone()})))
        }

    }
}