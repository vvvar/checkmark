pub mod js_runtime {
    use std::rc::Rc;
    use deno_core::error::AnyError;
    use std::path::Path;

    pub async fn run(file_path: &str) -> Result<(), AnyError> {
        let main_module = deno_core::resolve_path(file_path, Path::new("."))?;
        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
            ..Default::default()
        });
    
        let mod_id = js_runtime.load_main_module(&main_module, None).await?;
        let result = js_runtime.mod_evaluate(mod_id);
        js_runtime.run_event_loop(false).await?;
        result.await?
    }

    pub async fn format_markdown(content: &str) -> Result<(), AnyError> {
        let main_module = deno_core::resolve_path("/Users/vvoinov/Documents/repos/md-checker/src/js/index.js", Path::new("."))?;
        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
            ..Default::default()
        });
        let mod_id = js_runtime.load_main_module(&main_module, None).await?;
        let result = js_runtime.mod_evaluate(mod_id);
        js_runtime.run_event_loop(false).await?;
        result.await?
        // let md_result = js_runtime.execute_script("format_markdown", format!("console.log(await format_markdown(`{}`))", content).into());
        // let val = md_result.unwrap();
        // let mut scope = &mut js_runtime.handle_scope();
        // Ok(val.to_rust_string_lossy(scope));
    }

}
