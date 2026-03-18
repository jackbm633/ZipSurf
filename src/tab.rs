//! ```
//! Applies CSS styling rules to an HTML node and its descendants.
//!
//! This function traverses the structure of an HTML document, starting from the given node.
//! It matches CSS selectors with elements, applies the corresponding styles, and resolves
//! inline styles if defined in the `style` attribute of elements. The styles are stored in
//! the `style` attribute of each `HtmlNode` object.
//!
//! # Parameters
//! - `node`: An optional reference-counted pointer (`Rc`) to a mutable `HtmlNode`, which is
//!           the starting point of the HTML tree where the styles will be applied. If `None`,
//!           the function panics because the browser document is not properly initialized.
//! - `rules`: A reference to a vector of tuples where each tuple consists of:
//!   - A `Selector` object representing a CSS selector.
//!   - A `HashMap` containing CSS property-value key-pairs to apply.
//!
//! # Behavior
//! - The function uses the cascade principle to resolve conflicting styles based on selector specificity.
//! - Inline styles (from the HTML element's `style` attribute) take the highest precedence.
//! - Inherited properties (defined in the `INHERITED_PROPERTIES` constant) are passed down to descendant elements.
//!
//! # Panics
//! - The function panics if `node` is `None` when styling is attempted.
//!
//! # Example
//! ```rust
//! // Example usage:
//! let html_tree = HtmlNode::new_element("div".to_string());
//! let rules = vec![
//!     (Selector::new(".test".to_string()), HashMap::from([
//!         ("color".to_string(), "red".to_string())
//!     ]))
//! ];
//! Browser::style(Some(Rc::new(RefCell::new(html_tree))), &rules);
//! ```
//! fn
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use crate::layout::{LayoutNode, HEIGHT, VSTEP};
use crate::node::{HtmlNodeType, HtmlNode, Element};
use crate::url::Url;
use eframe::egui;
use egui::{Color32, Context, Galley, Pos2, Rect, Stroke, Vec2};
use std::sync::Arc;
use eframe::epaint::StrokeKind;
use lazy_static::lazy_static;
use crate::css_parser::CssParser;
use crate::html_parser::HtmlParser;
use crate::selector::Selector;


lazy_static! {
    static ref DEFAULT_STYLE_SHEET: Vec<(Selector, HashMap<String, String>)> = CssParser::new(include_str!("../assets/browser.css")).parse().unwrap();

    static ref INHERITED_PROPERTIES: HashMap<&'static str, &'static str> = HashMap::from([
        ("color", "black"),
        ("font-size", "16px"),
        ("font-weight", "normal"),
        ("font-style", "normal"),
    ]);
}


/// Represents a browser tab or a parsing/processing context.
///
/// The `Tab` struct encapsulates information about the current state of a browser tab
/// or a similar interactive environment. This includes visual components, stateful properties,
/// and references to layout and document structures.
///
/// # Fields
///
/// ## `draw_commands`
/// A collection of `DrawCommand` objects.
///
/// This vector stores instances of the `DrawCommand` type, which represent the visual rendering
/// commands required to display the contents of a tab. These commands are typically generated
/// during the rendering phase of a browser or UI application.
///
/// ### Example
/// ```
/// let draw_commands: Vec<DrawCommand> = Vec::new();
/// // Add draw commands to the vector as required
/// // draw_commands.push(DrawCommand::new(...));
/// ```
///
/// ### Usage
/// - Used in rendering pipelines for drawing UI elements.
/// - Can be serialized or processed to generate graphical outputs.
///
/// ## `scroll_y`
/// The current vertical scroll offset in points.
///
/// Represents how far the user has scrolled vertically within the content displayed by the tab.
/// It can be used to track user interactions or adjust rendering logic for visible content.
///
/// ### Example
/// ```
/// let scroll_y: f32 = 120.5;  // Scrolled 120.5 points downward
/// ```
///
/// ## `nodes`
/// An optional reference to an `HtmlNode` structure.
///
/// This field holds an `Option` that references a tree-like structure (`HtmlNode`) representing
/// the parsed DOM (Document Object Model) for an HTML document. The use of `Rc` and `RefCell`
/// provides shared ownership and interior mutability, allowing the DOM to be updated or accessed
/// between different parts of the system.
///
/// ### Example
/// ```
/// if let Some(ref html_nodes) = tab.nodes {
///     // Perform operations on the HtmlNode tree
/// }
/// ```
///
/// ## `document`
/// An optional reference to a `LayoutNode` structure.
///
/// Similar to `nodes`, this field holds a reference to the layout tree, which is generated
/// after the DOM is parsed and is used for rendering and visual arrangement of elements.
/// This is commonly seen in rendering engines for browsers, where the layout tree is used
/// to position and size visual elements.
///
/// ### Example
/// ```
/// if let Some(ref layout_document) = tab.document {
///     // Perform layout manipulations or rendering logic
/// }
/// ```
///
/// ## `url`
/// An optional `Url` representing the current URL of the tab.
///
/// Used to track the address of the resource currently displayed in the tab. This could be
/// an HTTP/HTTPS URL, or an internal representation such as a `file://` path. The `url`
/// provides context for navigation, back/forward actions, or displaying the resource location.
///
/// ### Example
/// ```
/// if let Some(ref current_url) = tab.url {
///     println!("Current tab URL: {}", current_url);
/// }
/// ```
///
/// ## `context`
/// The rendering or processing `Context` associated with the tab.
///
/// Represents the broader context in which the tab operates, such as rendering environments
/// or local state. This could include configurations, key-value storage, or system-level
/// objects needed for operations.
///
/// ### Example
/// ```
/// let rendering_context = tab.context.clone();
/// rendering_context.perform_action();
/// ```
pub struct Tab {
    /// A collection of `Token` objects.
    ///
    /// This vector stores instances of the `Token` type, which represent
    /// individual elements or symbols in a parsed input (e.g., programming
    /// language keywords, operators, or other lexemes). The `tokens` vector
    /// can be used in scenarios such as tokenizing source code, analyzing
    /// input streams, or implementing interpreters/compilers.
    ///
    /// # Example
    /// ```
    /// let tokens: Vec<Token> = Vec::new();
    /// // Add tokens to the vector as needed
    /// // tokens.push(Token::new(...));
    /// ```
    ///
    /// # Usage
    /// - Maintain the sequential order of tokens for parsing tasks.
    /// - Perform operations like iteration, filtering, or mapping on the list of tokens.
    pub(crate) draw_commands: Vec<DrawCommand>,
    /// The current vertical scroll offset in points.
    pub(crate) scroll_y: f32,
    nodes: Option<Rc<RefCell<HtmlNode>>>,
    document: Option<Rc<RefCell<LayoutNode>>>,
    pub(crate) url: Option<Url>,
    pub(crate) tab_height: f32,
    history: Vec<Url>,
    rules: Vec<(Selector, HashMap<String, String>)>,
    focus: Option<Rc<RefCell<HtmlNode>>>,
}



const SCROLL_STEP: f32 = 100.0;

impl Default for Tab {
    /// Returns a `Browser` instance with empty buffers and default scroll position.
    ///
    /// Note: The `context` is initialized with a default handle which should be
    /// overwritten during `new()` to ensure it points to the active UI context.
    fn default() -> Self {
        Tab {
            draw_commands: Vec::new(),
            scroll_y: 0.0,
            nodes: None,
            document: None,
            url: None,
            tab_height: 0.0,
            history: vec![],
            rules: vec![],
            focus: None,
        }
    }
}

impl Tab {
    /// Initializes a new browser instance and configures the UI environment.
    ///
    /// Sets the global visual theme to light mode to ensure text contrast and
    /// registers custom fonts required for international character support.
    ///
    /// # Arguments
    /// * `cc` - Integration context providing access to the egui render state.
    pub fn new(cc: &Context, height: f32) -> Self {
        cc.set_visuals(egui::Visuals::light());

        Self {
            tab_height: height,
            ..Default::default()
        }
    }


    pub fn update_layout(&mut self, ctx: &egui::Context) {
        if self.draw_commands.is_empty() {
            match self.document.as_mut() {
                None => { panic!("Browser document not initialized.") },
                Some(doc) => {
                    LayoutNode::layout(doc.clone(), ctx.clone());
                    self.draw_commands = vec![];
                    LayoutNode::paint_tree(doc.clone(), &mut self.draw_commands, Vec2::ZERO);

                }
            }
        }
    }

    pub fn scroll_down(&mut self) {
        match &self.document {
            None => { panic!("Browser document not initialized.") },
            Some(doc) => {
                let max_y = f32::max(0.0, doc.borrow().size.unwrap_or(Vec2::ZERO).y + 2.0*VSTEP - self.tab_height);
                self.scroll_y = (self.scroll_y + crate::tab::SCROLL_STEP).min(max_y);
            }
        }
        self.scroll_y += crate::tab::SCROLL_STEP;
    }

    pub(crate) fn click(&mut self, position: Pos2) {
        if self.focus.is_some() {
            self.focus.clone().unwrap().borrow_mut().is_focused = false;
        }
        let mut new_pos = position.clone();
        new_pos.y += self.scroll_y;

        let mut vec: Vec<Rc<RefCell<LayoutNode>>> = vec![];
        let objs = LayoutNode::tree_to_vec(self.document.clone().unwrap(), &mut vec).iter().filter(
            |l|
                Rect::from_two_pos(l.borrow().position.unwrap().to_pos2(), (l.borrow().position.unwrap()
                    + l.borrow().size.unwrap()).to_pos2()).contains(new_pos)
        ).collect::<Vec<&Rc<RefCell<LayoutNode>>>>();
        if objs.len() == 0 {
            return;
        }

        let mut element = objs.last().map(|&e| e.borrow().node.clone());
        let mut should_render = false;
        let mut url_to_load = None;

        while let Some(current_element) = element {
            let mut action_to_take = None; // Track what to do after borrow is released

            {
                let mut node = current_element.borrow_mut();
                match node.node_type {
                    HtmlNodeType::Element(ref mut ele) => {
                        if ele.tag == "a" && ele.attributes.contains_key("href") {
                            let url = self.url.as_ref().unwrap()
                                .resolve(ele.attributes.get("href").unwrap().clone().as_mut_str()).unwrap();
                            url_to_load = Some(url);
                            break;
                        } else if ele.tag == "input" {
                            ele.attributes.insert("value".to_string(), "".to_string());
                            node.is_focused = true;
                            action_to_take = Some("input");
                        } else if ele.tag == "button" {
                            action_to_take = Some("button");
                        }
                    }
                    HtmlNodeType::Text(_) => {}
                }
            }

            // Handle actions after the 'node' borrow is dropped
            match action_to_take {
                Some("input") => {
                    self.focus = Some(current_element.clone());
                    self.render();
                    break;
                }
                Some("button") => {
                    let mut elt = Some(current_element.clone());
                    while let Some(elt_rc) = elt {
                        let is_form_with_action = {
                            let elt_borrow = elt_rc.borrow();
                            if let HtmlNodeType::Element(elem) = &elt_borrow.node_type {
                                elem.tag == "form" && elem.attributes.contains_key("action")
                            } else {
                                false
                            }
                        };

                        if is_form_with_action {
                            return self.submit_form(elt_rc);
                        }
                        elt = elt_rc.borrow().parent.clone();
                    }
                }
                _ => {}
            }

            if url_to_load.is_some() { break; }
            element = current_element.borrow().parent.clone();
        }

        if let Some(url) = url_to_load {
            self.load(url);
        } else if should_render {
            self.render();
        }
    }

    /// Fetches a web page, strips HTML tags, and stores the raw content.
    ///
    /// This triggers a blocking network request. Upon success, the response
    /// body is passed through a lexer to remove markup before being cached
    /// in `self.body`.
    ///
    /// # Errors
    /// Network failures or request timeouts are logged to `stderr`.
    pub fn load(&mut self, url: Url) {
        self.url = Some(url.clone());
        self.draw_commands.clear();
        self.scroll_y = 0.0;
        match url.request() {
            Ok(body) => {
                let mut parser = HtmlParser {
                    body: body.clone(),
                    unfinished: vec![]
                };

                self.nodes =  Some(parser.parse());
                self.rules = DEFAULT_STYLE_SHEET.clone();

                let links =
                    HtmlNode::tree_to_vec(self.nodes.clone().unwrap(), &mut vec![])
                        .iter().filter_map(|p| match &p.borrow().node_type {
                        HtmlNodeType::Element(e) => {
                            if e.tag == "link" && e.attributes.contains_key("rel") && e.attributes.get("rel").unwrap() == "stylesheet"
                            && e.attributes.contains_key("href") {
                                return Some(e.attributes.get("href").unwrap().to_string())
                            }
                            None
                        }
                        HtmlNodeType::Text(_) => {None}
                    }).collect::<Vec<String>>();

                for link in links {
                    let style_url = url.resolve(link.clone().as_mut_str());
                    match style_url {
                        Ok(st) => {
                            let body = st.request();
                            match body {
                                Ok(bd) => {
                                    self.rules.append(&mut CssParser::new(&*bd).parse().unwrap_or(vec![]));
                                }
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                self.rules.sort_by(|a, b|
                    Self::cascade_priority(a).cmp(&Self::cascade_priority(b)));
                self.render();
            }
            Err(e) => {
                eprintln!("Error loading URL: {}", e);
            }
        }
        self.history.push(url);
    }

    fn render(&mut self) {
        Self::style(Some(self.nodes.clone().unwrap()), &self.rules);
        self.document = Some(LayoutNode::new_document(self.nodes.clone().unwrap()));
        self.draw_commands.clear()
    }

    pub fn go_back(&mut self) {
        if self.history.len() > 1
        {
            self.history.pop();
            self.load(self.history.last().unwrap().clone());
        }
    }

    fn cascade_priority(rule: &(Selector, HashMap<String, String>)) -> i32 {
        return rule.0.priority();
    }

    /// Applies CSS styling rules to an HTML node and its descendants.
    ///
    /// This function traverses the structure of an HTML document, starting from the given node.
    /// It matches CSS selectors with elements, applies the corresponding styles, and resolves
    /// inline styles if defined in the `style` attribute of elements. The styles are stored in
    /// the `style` attribute of each `HtmlNode` object.
    ///
    /// # Parameters
    /// - `node`: An optional reference-counted pointer (`Rc`) to a mutable `HtmlNode`, which is
    ///           the starting point of the HTML tree where the styles will be applied. If `None`,
    ///           the function panics because the browser document is not initialized.
    /// - `rules`: A vector of CSS rules, where each rule is represented as a tuple containing:
    ///   - `Selector`: A CSS selector that determines which elements the rule applies to.
    ///   - `HashMap<String, String>`: A map of CSS property names and their associated values.
    ///
    /// # Behavior
    /// 1. Checks if the `node` is `None`. If it is `None`, the function panics with the message:
    ///    "Browser document not initialized."
    /// 2. If the `node` is present:
    ///    - If the node is an element (`HtmlNodeType::Element`):
    ///      - Iterates over the provided `rules`.
    ///      - Matches the `Selector` against the current `node`. If the selector doesn't match,
    ///        the rule is skipped.
    ///      - For matching selectors, updates the node's `style` by adding CSS properties and
    ///        their values from the rule to the `style` attribute of the node.
    ///      - Resolves and parses any inline styles defined in the `style` attribute of the
    ///        element, applying them to the node's `style`.
    ///    - If the node is a text node (`HtmlNodeType::Text`), it skips processing as styles
    ///      are not applicable to text nodes.
    /// 3. Recursively applies styles to all child nodes of the current node.
    ///
    /// # Panics
    /// - If the `node` is `None`, indicating an uninitialized document.
    /// - If the inline `style` attribute of an element fails to parse using the `CssParser`,
    ///   the function will panic without a specific error message.
    ///
    /// # Examples
    /// ```
    /// // Example usage:
    /// let root_node = Some(Rc::new(RefCell::new(HtmlNode::new(HtmlNodeType::Element(
    ///     ElementData::new("div".to_string(), HashMap::new())
    /// )))));
    /// let rules = vec![
    ///     (Selector::new("div".to_string()), HashMap::from([("color".to_string(), "red".to_string())]))
    /// ];
    /// style(root_node, rules);
    /// ```
    ///
    /// In this example, the `div` element will have its `color` property set to "red".
    ///
    /// # Notes
    /// - This function assumes that `Selector::matches` is defined and correctly matches selectors
    ///   to elements.
    /// - The expected behavior of `CssParser::new` and its `body` method is to parse inline styles and
    ///   resolve them into a `HashMap<String, String>`. Any deviations from this expected behavior may
    ///   cause a panic.
    fn style(node: Option<Rc<RefCell<HtmlNode>>>, rules: &Vec<(Selector, HashMap<String, String>)>) {
        let nd = node.expect("Browser document not initialized.");

        let mut css_style_maps = Vec::new();
        let mut inherited_style_map = HashMap::<String, String>::new();
        for item in INHERITED_PROPERTIES.iter() {
            match nd.borrow().parent {
                None => {
                    inherited_style_map.insert(item.0.to_string(), item.1.to_string());
                }
                Some(ref pt) => {
                    inherited_style_map.insert(item.0.parse().unwrap(), pt.borrow().style.get(&item.0.to_string()).unwrap().to_string());

                }
            }
        }
        css_style_maps.push(inherited_style_map);

        for (selector, style_map) in rules {
            if selector.matches(nd.clone()) {
                css_style_maps.push(style_map.clone());
            }
        }


        
        // Encapsulate the mutation in a block to drop the borrow_mut() before recursion
        let children = {
            let mut node_ref = nd.borrow_mut();

            let inline_style_attr = if let HtmlNodeType::Element(el)
                = &node_ref.node_type {
                el.attributes.get("style").cloned()
            } else {
                None
            };

            for style_map in css_style_maps {
                for (property, value) in style_map {
                    node_ref.style.insert(property.clone(), value.clone());
                }
            }

            if let Some(style_str) = inline_style_attr {
                let mut parser = CssParser::new(&style_str);
                if let Ok(pairs_map) = parser.body() {
                    for (key, value) in pairs_map {
                        node_ref.style.insert(key, value);
                    }
                }
            }

            node_ref.children.clone()
        };


        if nd.borrow_mut().style.get("font-size").unwrap().ends_with("%") {
            let mut node_ref = nd.borrow_mut();
            let parent_font_size = match &node_ref.parent
            {
                None => {INHERITED_PROPERTIES.get("font-size").unwrap().to_string()}
                Some(pt) => {pt.borrow().style.get("font-size").unwrap().to_string()}
            };

            let node_pct = f32::from_str(&node_ref.style.get("font-size").unwrap().replace("%", "")).unwrap() / 100.0;
            let parent_x = parent_font_size.replace("px", "").parse::<f32>().unwrap();
            node_ref.style.insert("font-size".to_string(), format!("{}px", parent_x * node_pct));
        }

        // 3. Recursive phase - the borrow on 'nd' is now released
        for child in children {
            Self::style(Some(child), rules);
        }
    }

    pub fn keypress(&mut self, keypress: &String)
    {
        if self.focus.is_some() {
            match self.focus.clone().unwrap().borrow_mut().node_type {
                HtmlNodeType::Element(ref mut e) => {
                    e.attributes.get_mut("value").unwrap().push_str(keypress);
                }
                HtmlNodeType::Text(_) => {}
            }
            self.render();
        }
    }

    fn submit_form(&self, html_node: Rc<RefCell<HtmlNode>>) {
        let mut binding = vec![];
        let inputs: Vec<_> = HtmlNode::tree_to_vec(html_node, &mut binding)
            .iter()
            .filter(|n| {
                match &n.borrow().node_type {
                    HtmlNodeType::Element(e) => {
                        e.tag == "input" && e.attributes.contains_key("name")
                    }
                    HtmlNodeType::Text(_) => {false}
                }
            })
        .collect();

        let mut body: String = "".into();
        for input in inputs {
            match &input.borrow().node_type {
                HtmlNodeType::Element(e) => {
                    let name = &e.attributes["name"];
                    let value = e.attributes.get("value").unwrap_or(&"".to_string()).clone();
                    body.push_str( &format!("&{}={}", name, value));
                }
                HtmlNodeType::Text(_) => {}
            }
        }
        body.remove(0);
    }
}





/// A structure that represents text rendering properties, including its content,
/// position, and font details.
///
/// # Fields
/// - `content`:
///   The textual content to be rendered, such as a word or character.
/// - `x`:
///   The absolute horizontal position of the text in points.
/// - `y`:
///   The absolute vertical position of the text in points.
/// - `font_name`:
///   The name of the font family to be used for rendering the text.
#[derive(Clone, Debug)]
pub(crate) struct DrawText {
    /// Absolute horizontal position in points.
    pub(crate) x: f32,
    /// Absolute vertical position in points.
    pub(crate) y: f32,
    /// Galley for drawing the text.
    pub(crate) galley: Arc<Galley>
}

pub(crate) struct DrawRect {
    pub rect: Rect,
    /// A public field representing the color of the object.
    ///
    /// This field uses the `Color32` type, which encapsulates a 32-bit color value,
    /// typically including red, green, blue, and alpha (transparency) components.
    ///
    /// # Example
    /// ```rust
    /// use some_module::Color32;
    ///
    /// struct Object {
    ///     pub color: Color32,
    /// }
    ///
    /// let my_color = Color32::from_rgba_unmultiplied(255, 0, 0, 255); // Red color
    /// let object = Object { color: my_color };
    /// println!("The color is {:?}", object.color);
    /// ```
    ///
    /// # Usage
    /// This field can be read or modified directly in structs where it is declared as `pub`.
    pub color: Color32
}

pub(crate) struct DrawOutline{
    pub(crate) rect: Rect,
    pub(crate) color: Color32,
    pub(crate) thickness: f32
}

pub struct DrawLine {
    pub(crate) from: Pos2,
    pub(crate) to: Pos2,
    pub(crate) color: Color32,
    pub(crate) thickness: f32
}
pub enum DrawCommand {
    DrawText(DrawText),
    DrawRect(DrawRect),
    DrawOutline(DrawOutline),
    DrawLine(DrawLine),
}

impl DrawCommand {
    /// Calculates the bottom y-coordinate of a `DrawCommand`.
    ///
    /// This method determines the vertical bottom position based on the type of
    /// `DrawCommand`:
    ///
    /// - If the `DrawCommand` is `DrawText`, the bottom is computed as the sum of
    ///   the `y` coordinate of the text and the height of its galley (text layout).
    /// - If the `DrawCommand` is `DrawRect`, the bottom is the `y` coordinate of
    ///   the rectangle's bottom-right corner.
    ///
    /// # Returns
    ///
    /// A `f32` value representing the bottom y-coordinate of the `DrawCommand`.
    ///
    /// # Examples
    /// ```
    /// let text_command = DrawCommand::DrawText(Text {
    ///     y: 10.0,
    ///     galley: Galley { rect: Rect { height: 20.0 } },
    /// });
    /// assert_eq!(text_command.bottom(), 30.0);
    ///
    /// let rect_command = DrawCommand::DrawRect(Rect {
    ///     bottom_right: Point { y: 25.0 },
    /// });
    /// assert_eq!(rect_command.bottom(), 25.0);
    /// ```
    ///
    /// # Panics
    /// This function does not explicitly panic under normal circumstances, provided
    /// valid `DrawCommand` variants are used.
    pub(crate) fn bottom(&self) -> f32 {
        match self {
            DrawCommand::DrawText(txt) => {
                txt.y + txt.galley.rect.height()
            }
            DrawCommand::DrawRect(rct) => {
                rct.rect.bottom()
            },
            DrawCommand::DrawOutline(out) => out.rect.top(),
            DrawCommand::DrawLine(line) => f32::min(line.from.y, line.to.y)
        }
    }

    /// ```rust
    /// Returns the `y` coordinate of the top position for the current `DrawCommand`.
    ///
    /// This method determines the top position based on the specific variant
    /// of the `DrawCommand` enum:
    ///
    /// - If the `DrawCommand` is a `DrawText`, it returns the `y` coordinate of the text's position.
    /// - If the `DrawCommand` is a `DrawRect`, it returns the `y` coordinate of the rectangle's top-left corner.
    ///
    /// # Returns
    /// A `f32` representing the `y` coordinate of the top position.
    ///
    /// # Example
    /// ```rust
    /// let text_command = DrawCommand::DrawText(Text { y: 10.0 });
    /// assert_eq!(text_command.top(), 10.0);
    ///
    /// let rect_command = DrawCommand::DrawRect(Rect { top_left: Point { x: 5.0, y: 20.0 } });
    /// assert_eq!(rect_command.top(), 20.0);
    /// ```
    /// ```
    pub(crate) fn top(&self) -> f32 {
        match self {
            DrawCommand::DrawText(txt) => {
                txt.y
            }
            DrawCommand::DrawRect(rct) => {
                rct.rect.top()
            },
            DrawCommand::DrawOutline(out) => out.rect.top(),
            DrawCommand::DrawLine(line) => f32::max(line.from.y, line.to.y)
        }
    }
}