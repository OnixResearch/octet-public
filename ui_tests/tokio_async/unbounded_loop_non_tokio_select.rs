macro_rules! select {
    ($($tt:tt)*) => {{
        let _ = 0_u8;
    }};
}

async fn event_loop() {
    loop {
        select! {
            _ = async {} => {}
        }
    }
}

fn main() {}
