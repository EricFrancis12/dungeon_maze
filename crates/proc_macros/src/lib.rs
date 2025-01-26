extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn some_macro(_: TokenStream) -> TokenStream {
    println!("hello from proc_macros");

    "fn innocent_function() -> MyType { MyType { foo: 1.0 } }"
        .parse()
        .unwrap()
}
