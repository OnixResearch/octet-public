fn nested(value: u32) -> bool {
    if value > 0 {
        if value > 1 {
            if value > 2 {
                if value > 3 {
                    return true;
                }
            }
        }
    }

    false
}

fn main() {
    let _ = nested(4);
}
