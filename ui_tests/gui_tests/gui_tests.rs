// A downstream crate whose source path contains `ui_tests/` or `tests/fixtures`,
// such as `crates/gui_tests/src/lib.rs`, is production code: the import and
// path-naming lints check it like any other source file.
#![crate_type = "lib"]

mod widget {
    pub struct Button;

    pub trait Draw {
        fn draw(&self) {}
    }

    // Should warn: `path_segment_repetition`, `widget` repeats the module name.
    pub fn widget_count() -> u32 {
        0
    }
}

// Should warn: `non_trait_imports`.
use crate::widget::Button;

// Should warn: `renamed_imports` and `non_trait_imports`.
use std::collections::BTreeMap as Registry;

// Should NOT warn: trait imports stay allowed for method resolution.
use crate::widget::Draw;

pub fn render(button: Button, registry: Registry<u8, u8>) -> u32 {
    let _ = (button, registry);
    widget::widget_count()
}

pub fn draw_all(items: &[&dyn Draw]) {
    for item in items {
        item.draw();
    }
}
