use std::cell::RefCell;
use std::rc::Rc;
use lazy_static::lazy_static;
use rquickjs::{Context, Error, Function};
use rquickjs::Runtime;
use crate::css_parser::CssParser;
use crate::tab::Tab;

lazy_static! {
    static ref RUNTIME_JS: String = include_str!("../assets/runtime.js").to_string();
}
pub struct JsContext {
    runtime: Runtime,
    context: Context,
    tab: Rc<RefCell<Tab>>
}

impl JsContext {
    pub fn new(tab: Rc<RefCell<Tab>>) -> Self {
        let runtime = Runtime::new().expect("Failed to create JS runtime");
        // Full context includes standard library features (JSON, etc.)
        let context = Context::full(&runtime).expect("Failed to create JS context");

        context.with(|ctx| {
            let log = |str: String| {println!("{}", str)};
            let querySelectorAll = |selector: String| {
                let selector = CssParser::new(selector.as_str()).selector().unwrap();
            };
            ctx.globals().set("rustLog", Function::new(ctx.clone(), log).unwrap()).unwrap();
            ctx.globals().set("rustQuerySelectorAll", Function::new(ctx.clone(), querySelectorAll).unwrap()).unwrap();

            let _: () = ctx.eval(RUNTIME_JS.as_str()).expect("JS Execution failed");
        });

        Self { runtime, context, tab }
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