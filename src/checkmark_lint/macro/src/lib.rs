extern crate proc_macro;

use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemFn, LitStr, PathSegment, ReturnType, Type, TypePath};

#[derive(Debug, FromMeta)]
struct RuleMacroArgs {
    requirement: String,
    rationale: String,
    documentation: String,
    additional_links: Vec<LitStr>,
    is_fmt_fixable: bool,
}

#[proc_macro_attribute]
pub fn rule(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Because we re-use token stream for darling later.
    let item_c = item.clone();

    // Parse the function item.
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let fn_args = &input_fn.sig.inputs;
    let fn_return_type = &input_fn.sig.output;
    // Validate the function return type.
    if !check_return_type(fn_return_type) {
        return proc_macro::TokenStream::from(
            darling::Error::custom("return type should be Vec<checkmark_lint_common::Violation>")
                .with_span(&fn_return_type.span())
                .write_errors(),
        );
    }

    // Create the struct with the same functionality.
    let struct_name = syn::Ident::new(
        &fn_name.to_string().to_uppercase().to_string(),
        fn_name.span(),
    );

    // Extract metadata in types suitable for generating Metadata.
    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let _input = syn::parse_macro_input!(item_c as ItemFn);
    let args = match RuleMacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let rule_code = struct_name.to_string().to_uppercase();
    let requirement = args.requirement;
    let rationale = args.rationale;
    let documentation = args.documentation;
    let additional_links = args.additional_links;
    let is_fmt_fixable = args.is_fmt_fixable;

    // Validate input.
    let mut validation_errors: Vec<darling::Error> = vec![];
    if requirement.is_empty() {
        validation_errors.push(darling::Error::custom("requirement cant be empty"));
    }
    if rationale.is_empty() {
        validation_errors.push(darling::Error::custom("rationale cant be empty"));
    }
    if documentation.is_empty() {
        validation_errors.push(darling::Error::custom("documentation cant be empty"));
    }
    if !validation_errors.is_empty() {
        let compile_errors = validation_errors
            .iter()
            .map(|e| e.clone().write_errors())
            .collect::<proc_macro2::TokenStream>();
        return proc_macro::TokenStream::from(compile_errors);
    }

    let output = quote! {
        #[derive(Default)]
        pub struct #struct_name;

        impl checkmark_lint_common::Rule for #struct_name {
            fn metadata(&self) -> checkmark_lint_common::Metadata {
                checkmark_lint_common::Metadata {
                    code: #rule_code,
                    requirement: #requirement,
                    rationale: #rationale,
                    documentation: url_macro::url!(#documentation),
                    additional_links: vec![#(url_macro::url!(#additional_links)),*],
                    is_fmt_fixable: #is_fmt_fixable,
                }
            }

            fn check(&self, #fn_args) #fn_return_type {
                #fn_body
            }
        }
    };

    output.into()
}

/// Checks that return type is Vec<Violation>.
fn check_return_type(return_type: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = return_type {
        if let Type::Path(TypePath { path, .. }) = ty.as_ref() {
            // Check if the last segment in the path is "Vec"
            if let Some(PathSegment { ident, arguments }) = path.segments.last() {
                if ident == "Vec" {
                    // Further check if the generic argument is "Violation"
                    if let syn::PathArguments::AngleBracketed(args) = arguments {
                        for arg in &args.args {
                            if let syn::GenericArgument::Type(Type::Path(type_path)) = arg {
                                if type_path.path.is_ident("Violation") {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

#[derive(Debug, FromMeta)]
struct RuleTestMacroArgs {
    markdown: String,
}

#[proc_macro_attribute]
pub fn rule_test(attr: TokenStream, item: TokenStream) -> TokenStream {
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
