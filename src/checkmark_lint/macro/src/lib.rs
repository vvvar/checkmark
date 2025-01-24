extern crate proc_macro;
use proc_macro::TokenStream;

mod rule;
mod rule_test;

#[proc_macro_attribute]
pub fn rule(attr: TokenStream, item: TokenStream) -> TokenStream {
    rule::rule_impl(attr, item)
}

#[proc_macro_attribute]
pub fn rule_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    rule_test::rule_test_impl(attr, item)
}
