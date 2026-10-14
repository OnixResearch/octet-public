fn iter_next_loop_still_warns(items: Vec<u32>) {
    let mut iter = items.into_iter();
    while let Some(value) = iter.next() {
        std::hint::black_box(value);
    }
}

fn main() {
    iter_next_loop_still_warns(vec![1, 2, 3]);
}
