use crate::css_parser::CssParser;
use crate::html_parser::HtmlParser;
use crate::node::{HtmlNode, HtmlNodeType};
use crate::tab::Tab;
use crate::task::Task;
use lazy_static::lazy_static;
use rquickjs::Runtime;
use rquickjs::{Context, Function};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;

lazy_static! {
    static ref RUNTIME_JS: String = include_str!("../assets/runtime.js").to_string();
}
pub struct JsContext {
    pub(crate) context: Arc<RwLock<Context>>,
    nodes: Arc<RwLock<Vec<Arc<RwLock<HtmlNode>>>>>,
    pub(crate) discarded: Arc<RwLock<bool>>
}

impl JsContext {
    pub(crate) fn dispatch_event(&self, event_type: &str, node: Arc<RwLock<HtmlNode>>) {
        let index = {
            let mut nodes = self.nodes.write().unwrap();
            nodes.iter().position(|n| Arc::ptr_eq(n, &node)).unwrap_or_else(|| {
                nodes.push(node);
                nodes.len() - 1
            })
        };
        self.context.read().unwrap().with(|ctx| {
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
    pub fn new(tab: Arc<RwLock<Tab>>) -> Self {
        let runtime = Runtime::new().expect("Failed to create JS runtime");
        // Full context includes standard library features (JSON, etc.)
        let context = Arc::new(RwLock::new(Context::full(&runtime).expect("Failed to create JS context")));
        let discarded_pointer = Arc::new(RwLock::new(false));
        let discarded_pointer_clone = discarded_pointer.clone();
        let tab_clone = tab.clone();
        let nodes: Arc<RwLock<Vec<Arc<RwLock<HtmlNode>>>>> = Arc::new(RwLock::new(Vec::new()));
        let nodes_clone = nodes.clone();

        context.read().unwrap().with(|ctx| {
            let tab_for_query = tab_clone.clone();
            let nodes_for_query = nodes_clone.clone();

            let log = |str: String| {println!("{}", str)};
            let query_selector_all = move |selector_str: String| {
                let selector = CssParser::new(selector_str.as_str()).selector().unwrap();
                let nodes_ref = tab_for_query.read().unwrap().nodes.clone();

                if let Some(root) = nodes_ref {
                    let mut elements = vec![];
                    HtmlNode::tree_to_vec(root, &mut elements);

                    let mut matched_indices = vec![];
                    for nd in elements {
                        if selector.matches(nd.clone()) {
                            let index = {
                                let mut registry = nodes_for_query.write().unwrap();
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
                let nodes = nodes_clone.read().unwrap();
                let element_rc = nodes.get(handle).unwrap();
                let element = element_rc.read().unwrap();
                let attr = match &element.node_type {
                    HtmlNodeType::Element(e) => {
                        e.attributes.get(&attrib).unwrap_or(&"".to_string()).clone()
                    }
                    HtmlNodeType::Text(_) => {"".into()}
                };
                return attr;

            };
            let nodes_for_inner_html = nodes.clone();
            let ihs_tab = tab.clone();
            let inner_html_set = move |handle: usize, html: String| {
                let mut parser = HtmlParser {
                    body: format!("<html><body>{}</body></html>", html.clone()),
                    unfinished: vec![],
                };
                let parsed = parser.parse();
                let new_node_parent = &parsed.read().unwrap().children.first().unwrap().clone();
                let new_children = new_node_parent.read().unwrap().children.clone();

                let element_rc = {
                    let nodes = nodes_for_inner_html.read().unwrap();
                    nodes.get(handle).unwrap().clone()
                };

                {
                    let mut element = element_rc.write().unwrap();
                    element.children = new_children.clone();
                }

                    for child in new_children {
                        child.write().unwrap().parent = Some(element_rc.clone());
                    }

                    if let Ok(mut tab_borrow) = ihs_tab.try_write() {
                        tab_borrow.render();
                    } else {
                        eprintln!("RE-ENTRANT BORROW DETECTED!");
                        eprintln!("Tab is already borrowed. This call to inner_html_set is likely");
                        eprintln!("triggered from within a Tab method that already holds a borrow.");

                        // To see the current call stack:
                        let bt = std::backtrace::Backtrace::capture();
                        println!("{}", bt);
                        println!("Warning: Tab already borrowed, skipping immediate render.");
                    }
                };
            let xml_tab = tab.clone();
            let xml_http_request_send = move |ctx: rquickjs::Ctx, _method: String, mut url: String, body: String, is_async: bool, handle: usize| -> rquickjs::Result<String> {
                let full_url = xml_tab.read().unwrap().url.clone().unwrap().resolve(url.as_mut_str());
                if Tab::allowed_request(xml_tab.clone(), full_url.clone().unwrap()) == false {
                    return Err(ctx.throw(rquickjs::Value::from_string(rquickjs::String::from_str(ctx.clone(), "CORS request blocked").unwrap())));
                }
                let url_resolved = full_url.unwrap();
                let cookie_jar = xml_tab.read().unwrap().cookie_jar.clone();
                // Define standard logic to make request and create the task
                // (Without capturing context or tab!)
                let build_xhr_task = move |content: String| -> Task {
                    Task::new(move |tab: &Tab| {
                        // Safe: We are on the main thread here, and access the non-Send context locally
                        if let Some(ref js) = tab.js {
                            if *js.discarded.read().unwrap() { return; }
                            js.context.read().unwrap().with(|ctx| {
                                let js_dispatch = format!("__runXHROnload({}, {})", content, handle);
                                let _ = ctx.eval::<(), _>(js_dispatch.as_str());
                            });
                        }
                    })
                };
                if !is_async {
                    // Synchronous case: run immediately on main thread
                    if let Ok(response) = url_resolved.request(Some(body), cookie_jar) {
                        let mut task = build_xhr_task(response.content.clone());
                        task.run(&xml_tab.read().unwrap());
                        Ok(response.content)
                    } else {
                        Ok("".to_string())
                    }
                } else {
                    // Asynchronous case: get the thread-safe sender
                    let task_tx = xml_tab.read().unwrap().task_tx.clone().unwrap();
                    std::thread::spawn(move || {
                        if let Ok(response) = url_resolved.request(Some(body), cookie_jar) {
                            // Construct the task (capturing only the Send content string)
                            let task = build_xhr_task(response.content);
                            // Send it to the main thread's runner
                            let _ = task_tx.send(task);
                        }
                    });
                    Ok("".to_string())
                }
            };
            


            let timeout_arc = tab.clone();
            let context_for_timeout = context.clone();
            let set_timeout = move |_ctx: rquickjs::Ctx, code: String, timeout: u64| -> rquickjs::Result<()> {
                let context_for_task = context_for_timeout.clone();
                let pd = discarded_pointer.clone();
                let task = Task::new(move |tab| {
                                            if let Some(ref js) = tab.js {
                                                sleep(Duration::from_millis(timeout));
                    if pd.read().unwrap().clone() == true {
                        return;
                    }
                    let res: Result<(), _> = js.context.read().unwrap().with(|ctx| ctx.eval(code.as_str()));
                    if let Err(e) = res {
                        if let rquickjs::Error::Exception = e {
                            js.context.read().unwrap().with(|ctx| {
                                let exception = ctx.catch();
                                println!("JS Exception in setTimeout callback: {:?}", exception);
                            });
                        } else {
                            println!("Failed to run setTimeout callback: {e}");
                        }
                    }
                                            }

                    
                });
                timeout_arc.write().unwrap().task_runner.as_mut().unwrap().schedule_task(task);
                Ok(())
            };
            ctx.globals().set("rustGetAttribute", Function::new(ctx.clone(), get_attribute).unwrap()).unwrap();
            ctx.globals().set("rustLog", Function::new(ctx.clone(), log).unwrap()).unwrap();
            ctx.globals().set("rustInnerHtmlSet", Function::new(ctx.clone(), inner_html_set).unwrap()).unwrap();
            ctx.globals().set("rustQuerySelectorAll", Function::new(ctx.clone(), query_selector_all).unwrap()).unwrap();
            ctx.globals().set("rustXmlHttpRequestSend", Function::new(ctx.clone(), xml_http_request_send).unwrap()).unwrap();
            ctx.globals().set("rustSetTimeout", Function::new(ctx.clone(), set_timeout).unwrap()).unwrap();

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

        Self { context, nodes, discarded: discarded_pointer_clone }
    }


    pub fn run(&self, script_name: &str, code: &str)
    {
        self.context.read().unwrap().with(|ctx| {
            let result: Result<(), _> = ctx.eval(code);
            match result {
                Ok(_) => {}
                Err(e) => {println!("Script {script_name} failed to run: {e}")}
            }
        });
    }
}