#![warn(float_for_currency)]

// Should warn: float type for currency field
struct Usage {
    pub cost_usd: f64,
}

// Should NOT warn: percentage is legitimately float
struct Breakdown {
    pub percentage: f32,
}

// Should NOT warn: not a currency name
struct Metrics {
    pub latency: f64,
}

fn main() {
    let _ = Usage { cost_usd: 1.0 };
    let _ = Breakdown { percentage: 50.0 };
    let _ = Metrics { latency: 1.0 };
}
