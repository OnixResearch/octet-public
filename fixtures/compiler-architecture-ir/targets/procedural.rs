extern crate proc_macro;

#[proc_macro]
pub fn observe(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = std::env::var_os("NEVER_EXECUTED");
    input
}
