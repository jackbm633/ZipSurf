use crate::css_parser::CssParser;
use crate::html_parser::HtmlParser;
use crate::node::{HtmlNode, HtmlNodeType};
use crate::tab::Tab;
use lazy_static::lazy_static;
use rquickjs::Runtime;
use rquickjs::{Context, Function};
use std::cell::RefCell;
use std::rc::Rc;

lazy_static! {
    static ref RUNTIME_JS: String = include_str!("../assets/runtime.js").to_string();
}
pub struct JsContext {
    context: Context,
    nodes: Rc<RefCell<Vec<Rc<RefCell<HtmlNode>>>>>
}

impl JsContext {
    pub(crate) fn dispatch_event(&self, event_type: &str, node: Rc<RefCell<HtmlNode>>) {
        let index = {
            let mut nodes = self.nodes.borrow_mut();
            nodes.iter().position(|n| Rc::ptr_eq(n, &node)).unwrap_or_else(|| {
                nodes.push(node);
                nodes.len() - 1
            })
        };
        self.context.with(|ctx| {
            let res: Result<(), _>=  ctx.eval(format!("new Node({}).dispatchEvent('{}')", index, event_type).as_str());
            match res {
                Ok(_) => {}
                Err(e) => {
                    if let rquickjs::Error::Exception = e {
                        let exception = ctx.catch();
                        println!("JS Exception in dispatch_event: {:?}", exception);
                    } else {
                        println!("Failed to dispatch event: {e}");
                    }
                }
            }
        });
    }
}

impl JsContext {
    pub fn new(tab: Rc<RefCell<Tab>>) -> Self {
        let runtime = Runtime::new().expect("Failed to create JS runtime");
        // Full context includes standard library features (JSON, etc.)
        let context = Context::full(&runtime).expect("Failed to create JS context");

        let tab_clone = tab.clone();
        let nodes = Rc::new(RefCell::new(Vec::new()));
        let nodes_clone = nodes.clone();

        context.with(|ctx| {
            let tab_for_query = tab_clone.clone();
            let nodes_for_query = nodes_clone.clone();

            let log = |str: String| {println!("{}", str)};
            let query_selector_all = move |selector_str: String| {
                let selector = CssParser::new(selector_str.as_str()).selector().unwrap();
                let nodes_ref = tab_for_query.borrow().nodes.clone();

                if let Some(root) = nodes_ref {
                    let mut elements = vec![];
                    HtmlNode::tree_to_vec(root, &mut elements);

                    let mut matched_indices = vec![];
                    for nd in elements {
                        if selector.matches(nd.clone()) {
                            let index = {
                                let mut registry = nodes_for_query.borrow_mut();
                                let idx = registry.len();
                                registry.push(nd);
                                idx
                            };
                            matched_indices.push(index);
                        }
                    }

                    return matched_indices;
                }
                vec![]
            };

            let get_attribute = move |handle: usize, attrib: String| -> String {
                let nodes = nodes_clone.borrow();
                let element_rc = nodes.get(handle).unwrap();
                let element = element_rc.borrow();
                let attr = match &element.node_type {
                    HtmlNodeType::Element(e) => {
                        e.attributes.get(&attrib).unwrap_or(&"".to_string()).clone()
                    }
                    HtmlNodeType::Text(_) => {"".into()}
                };
                return attr;

            };
            let nodes_for_inner_html = nodes.clone();

            let inner_html_set = move |handle: usize, html: String| {
                let mut parser = HtmlParser {
                    body: format!("<html><body>{}</body></html>",html.clone()),
                    unfinished: vec![]
                };
                let parsed = parser.parse();
                let new_node_parent = &parsed.borrow().children.first().unwrap().clone();
                let new_node = &new_node_parent.borrow().children;
                let nodes = nodes_for_inner_html.borrow();
                let element_rc = nodes.get(handle).unwrap();
                element_rc.borrow_mut().children = new_node.clone();
                for child in &element_rc.borrow_mut().children
                {
                    child.borrow_mut().parent = Some(element_rc.clone());
                }
            };
            ctx.globals().set("rustGetAttribute", Function::new(ctx.clone(), get_attribute).unwrap()).unwrap();
            ctx.globals().set("rustLog", Function::new(ctx.clone(), log).unwrap()).unwrap();
            ctx.globals().set("rustInnerHtml", Function::new(ctx.clone(), inner_html_set).unwrap()).unwrap();
            ctx.globals().set("rustQuerySelectorAll", Function::new(ctx.clone(), query_selector_all).unwrap()).unwrap();

            let res: Result<(), _>= ctx.eval(RUNTIME_JS.as_str());
            match res {
                Ok(_) => {}
                Err(e) => {
                    if let rquickjs::Error::Exception = e {
                        let exception = ctx.catch();
                        println!("JS Exception in dispatch_event: {:?}", exception);
                    } else {
                        println!("Failed to dispatch event: {e}");
                    }
                }
            }
        });

        Self { context, nodes }
    }


    pub fn run<T>(&self, script_name: &str, code: &str)
    where
            for<'js> T: rquickjs::FromJs<'js> + 'static
    {
        // We use the 'for<'js>' syntax to handle lifetimes generically
        self.context.with(|ctx| {
            // We explicitly type the result as () to tell rquickjs
            // we don't intend to pull any JS values out of this call.
            let result: Result<T, _> = ctx.eval::<T, _>(code);
            match result {
                Ok(_) => {}
                Err(e) => {println!("Script {script_name} failed to run: {e}")}
            }
        });
    }
}