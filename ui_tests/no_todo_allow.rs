#![feature(register_tool)]
#![register_tool(tigerstyle)]

// justified stub while API lands
#[allow(tigerstyle::no_todo)]
fn tool_scoped_function_level_allowed() {
    todo!("allowed tool-scoped function-level stub");
}

#[allow(tigerstyle::no_todo)] // justified inline tool-scoped stub while API lands
fn tool_scoped_same_line_allowed() {
    todo!("allowed tool-scoped same-line stub");
}

fn tool_scoped_block_level_allowed() {
    // justified tool-scoped block stub while API lands
    #[allow(tigerstyle::no_todo)]
    {
        todo!("allowed tool-scoped block-level stub");
    }
}

// justified bare stub while API lands
#[allow(no_todo)]
fn bare_function_level_allowed() {
    todo!("allowed bare function-level stub");
}

#[allow(no_todo)] // justified inline bare stub while API lands
fn bare_same_line_allowed() {
    todo!("allowed bare same-line stub");
}

fn bare_block_level_allowed() {
    // justified bare block stub while API lands
    #[allow(no_todo)]
    {
        todo!("allowed bare block-level stub");
    }
}

/// doc comments do not justify todo escapes
#[allow(tigerstyle::no_todo)]
fn tool_scoped_doc_comment_still_warns() {
    todo!("tool-scoped doc comment should still warn");
}

/* block comments do not justify todo escapes */
#[allow(no_todo)]
fn bare_block_comment_still_warns() {
    todo!("bare block comment should still warn");
}

// blank line breaks adjacency

#[allow(tigerstyle::no_todo)]
fn tool_scoped_blank_line_still_warns() {
    todo!("tool-scoped blank line should still warn");
}

#[allow(no_todo)]
fn bare_missing_comment_still_warns() {
    todo!("bare missing comment should still warn");
}

fn main() {
    tool_scoped_function_level_allowed();
    tool_scoped_same_line_allowed();
    tool_scoped_block_level_allowed();
    bare_function_level_allowed();
    bare_same_line_allowed();
    bare_block_level_allowed();
}
