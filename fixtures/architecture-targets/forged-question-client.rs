pub fn direct_authored_question(result: std::io::Result<Vec<u8>>) -> std::io::Result<Vec<u8>> {
    Ok(result?)
}

// The input punctuation is genuine source; the macro substitutes a different `?`
// after stamping every generated token with this input token's span.
architecture_target_macro::emit_stolen_question_token_span!(?);
