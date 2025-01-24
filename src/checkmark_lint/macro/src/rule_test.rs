use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[derive(Debug, FromMeta)]
struct RuleTestMacroArgs {
    markdown: String,
}

pub fn rule_test_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Because we re-use token stream for darling later.
    let item_c = item.clone();

    // Parse the function item.
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let fn_args = &input_fn.sig.inputs;

    // Extract metadata in types suitable for generating Metadata.
    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let _input = syn::parse_macro_input!(item_c as ItemFn);
    let args = match RuleTestMacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let markdown = args.markdown;

    let output = quote! {
        #[test]
        fn #fn_name () {
            use pretty_assertions::assert_eq;
            let closure = |#fn_args| {
                #fn_body
            };
            let file = MarkDownFile {
                path: String::from("this/is/a/dummy/path/to/a/file.md"),
                content: String::from(#markdown),
                issues: vec![],
            };
            let ast = common::ast::parse(&file.content).unwrap();
            let mut config = Config::default();
            closure(&ast, &file, &mut config);
        }
    };

    output.into()
}
