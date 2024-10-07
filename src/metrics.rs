use mesura::{Counter, Gauge};

pub struct ViewMetrics {
    pub updates: Counter,
    pub elements_shown: Counter,
    pub cascades: Counter,
    pub layouts: Counter,
    pub styles: Gauge,
    pub cascade: CascadeMetrics,
}

impl ViewMetrics {
    pub fn new() -> ViewMetrics {
        Self {
            updates: Counter::new("bumaga_view_updates"),
            elements_shown: Counter::new("bumaga_view_elements_shown"),
            cascades: Counter::new("bumaga_view_cascades"),
            layouts: Counter::new("bumaga_view_layouts"),
            styles: Gauge::new("bumaga_view_styles"),
            cascade: CascadeMetrics::new(),
        }
    }
}

pub struct CascadeMetrics {
    pub matches_static: Counter,
    pub matches_dynamic: Counter,
    pub apply_ok: Counter,
    pub apply_error: Counter,
}

impl CascadeMetrics {
    pub fn new() -> Self {
        Self {
            matches_static: Counter::with_labels("bumaga_cascade_matches", ["method"], ["static"]),
            matches_dynamic: Counter::with_labels(
                "bumaga_cascade_matches",
                ["method"],
                ["dynamic"],
            ),
            apply_ok: Counter::with_labels("bumaga_cascade_apply", ["result"], ["ok"]),
            apply_error: Counter::with_labels("bumaga_cascade_apply", ["result"], ["error"]),
        }
    }
}
