macro_rules! make_flag {
    () => {
        let enabled = true;
        let _use_it = enabled;
    };
}

fn from_macro() {
    make_flag!();
}

fn not_from_macro() {
    let ready = true;
    let _a = ready;
}

fn main() {
    from_macro();
    not_from_macro();
}
