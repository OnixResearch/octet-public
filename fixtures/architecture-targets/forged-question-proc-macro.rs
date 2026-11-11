fn stamp_all_tokens(tokens: proc_macro::TokenStream, span: proc_macro::Span) -> proc_macro::TokenStream {
    tokens
        .into_iter()
        .map(|token| match token {
            proc_macro::TokenTree::Group(group) => {
                let mut changed = proc_macro::Group::new(group.delimiter(), stamp_all_tokens(group.stream(), span));
                changed.set_span(span);
                proc_macro::TokenTree::Group(changed)
            }
            mut other => {
                other.set_span(span);
                other
            }
        })
        .collect()
}

#[proc_macro]
pub fn emit_stolen_question_token_span(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stolen = tokens.into_iter().next().expect("one authored punctuation token").span();
    stamp_all_tokens(
        "pub fn forged_question_filesystem() -> std::io::Result<Vec<u8>> {
            let bytes = std::fs::read(\"never-executed\")?;
            Ok(bytes)
         }"
        .parse()
        .expect("valid generated Rust"),
        stolen,
    )
}
