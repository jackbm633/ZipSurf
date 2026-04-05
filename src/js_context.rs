use std::cell::RefCell;
use std::rc::Rc;
use lazy_static::lazy_static;
use rquickjs::{Context, Function};
use rquickjs::Runtime;
use crate::css_parser::CssParser;
use crate::node::{HtmlNode, HtmlNodeType};
use crate::tab::Tab;

lazy_static! {
    static ref RUNTIME_JS: String = include_str!("../assets/runtime.js").to_string();
}
pub struct JsContext {
    runtime: Runtime,
    context: Context,
    tab: Rc<RefCell<Tab>>,
    nodes: Rc<RefCell<Vec<Rc<RefCell<HtmlNode>>>>>
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
                            let mut registry = nodes_for_query.borrow_mut();
                            matched_indices.push(registry.len());
                            registry.push(nd);
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
            ctx.globals().set("rustGetAttribute", Function::new(ctx.clone(), get_attribute).unwrap()).unwrap();
            ctx.globals().set("rustLog", Function::new(ctx.clone(), log).unwrap()).unwrap();

            ctx.globals().set("rustQuerySelectorAll", Function::new(ctx.clone(), query_selector_all).unwrap()).unwrap();

            let _: () = ctx.eval(RUNTIME_JS.as_str()).expect("JS Execution failed");
        });

        Self { runtime, context, tab, nodes }
    }

    /// Generic evaluation with error handling
    /// The 'static bound on T ensures the data doesn't hold references
    /// back to the JS stack that's about to be destroyed.
    pub fn eval<T>(&self, code: &str) -> T
    where
            for<'js> T: rquickjs::FromJs<'js> + 'static
    {
        self.context.with(|ctx| {
            ctx.eval::<T, _>(code).expect("JS Evaluation failed")
        })
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