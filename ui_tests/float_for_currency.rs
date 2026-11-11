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

// Should NOT warn: currency words only match whole words, so `balancer`,
// `feedback`, and `priceless` are not currency names
struct Weights {
    pub load_balancer_weight: f64,
    pub feedback_score: f32,
    pub priceless_rank: f64,
}

// Should warn: whole currency words, including plural forms
struct Ledger {
    pub monthly_fees: f64,
    pub balances_total: f64,
}

// Should warn: skip words only match whole `_`-separated tokens
struct Charges {
    pub registration_fee: f64,
    pub operation_cost: f64,
    pub corporate_budget: f32,
}

// Should NOT warn: `rate` and `ratio` are whole skip-word tokens
struct Pricing {
    pub fee_rate: f64,
    pub price_ratio: f32,
}

// Should warn: float return type of a currency-named function
fn total_cost() -> f64 {
    1.0
}

// Should NOT warn: skip word token in the function name
fn fee_rate() -> f64 {
    0.5
}

// Should NOT warn: integer return type
fn cost_micros() -> u64 {
    1
}

impl Usage {
    // Should warn: float return type of a currency-named method
    fn price(&self) -> f32 {
        1.0
    }

    // Should NOT warn: not a currency name
    fn latency(&self) -> f64 {
        1.0
    }
}

trait Quote {
    // Should warn: the trait declaration owns the signature
    fn quote_amount(&self) -> f64;
}

impl Quote for Usage {
    // Should NOT warn: signature dictated by the trait
    fn quote_amount(&self) -> f64 {
        self.cost_usd
    }
}

fn main() {
    let usage = Usage { cost_usd: 1.0 };
    let _ = Breakdown { percentage: 50.0 };
    let _ = Metrics { latency: 1.0 };
    let _ = Charges {
        registration_fee: 1.0,
        operation_cost: 1.0,
        corporate_budget: 1.0,
    };
    let _ = Pricing {
        fee_rate: 0.1,
        price_ratio: 0.2,
    };
    let _ = total_cost();
    let _ = fee_rate();
    let _ = cost_micros();
    let _ = usage.price();
    let _ = usage.latency();
}
