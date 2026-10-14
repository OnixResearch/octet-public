fn allowed_depth(value: u32) -> bool {
    if value > 0 {
        if value > 1 {
            if value > 2 {
                if value > 3 {
                    if value > 4 {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn too_deep(value: u32) -> bool {
    if value > 0 {
        if value > 1 {
            if value > 2 {
                if value > 3 {
                    if value > 4 {
                        if value > 5 {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn main() {
    allowed_depth(5);
    too_deep(6);
}
