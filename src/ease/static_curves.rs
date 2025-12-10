use core::f64;

macro_rules! static_curve {
    ($name:ident ($param:ident) := $expression:expr) => {
        #[allow(unused)]
        pub struct $name;

        #[allow(unused)]
        impl $name {
            pub fn new() -> Self {
                Self
            }
        }

        impl $crate::ease::Curve for $name {
            fn f(&self, $param: f64) -> f64 {
                $expression
            }
        }
    };
}

static_curve!(Linear (x) := x);
static_curve!(Hold (_x) := 0.0);
static_curve!(Step (x) := x.round());

static_curve!(EaseInQuad (x) := x * x);
static_curve!(EaseOutQuad (x) := x * (2.0 - x));
static_curve!(EaseInOutQuad (x) := {
    if x < 0.5 {
        2.0 * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powf(2.0) / 2.0
    }
});

static_curve!(EaseInCubic (x) := x * x * x);
static_curve!(EaseOutCubic (x) := 1.0 + (x - 1.0) * (x - 1.0) * (x - 1.0));
static_curve!(EaseInOutCubic (x) := {
    if x < 0.5 {
        4.0 * x * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powf(3.0) / 2.0
    }
});

static_curve!(EaseInQuart (x) := x * x * x * x);
static_curve!(EaseOutQuart (x) := 1.0 - (1.0 - x).powf(4.0));
static_curve!(EaseInOutQuart (x) := {
    if x < 0.5 {
        8.0 * x * x * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powf(4.0) / 2.0
    }
});

static_curve!(EaseInQuint (x) := x * x * x * x * x);
static_curve!(EaseOutQuint (x) := 1.0 - (1.0 - x).powf(5.0));
static_curve!(EaseInOutQuint (x) := {
    if x < 0.5 {
        16.0 * x * x * x * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powf(5.0) / 2.0
    }
});

static_curve!(EaseInSine (x) := 1.0 - (x * f64::consts::PI / 2.0).cos());
static_curve!(EaseOutSine (x) := (x * f64::consts::PI / 2.0).sin());
static_curve!(EaseInOutSine (x) := (1.0 - (x * f64::consts::PI).cos()) / 2.0);

static_curve!(EaseInExpo (x) := if x == 0.0 { 0.0 } else { 2f64.powf(10.0 * x - 10.0) });
static_curve!(EaseOutExpo (x) := if x == 1.0 { 1.0 } else { 1.0 - 2f64.powf(-10.0 * x) });
static_curve!(EaseInOutExpo (x) := {
    if x == 0.0 {
        0.0
    } else if x == 1.0 {
        1.0
    } else if x < 0.5 {
        2f64.powf(20.0 * x - 10.0) / 2.0
    } else {
        (2.0 - 2f64.powf(-20.0 * x + 10.0)) / 2.0
    }
});

static_curve!(EaseInCirc (x) := 1.0 - (1.0 - x.powf(2.0)).sqrt());
static_curve!(EaseOutCirc (x) := (1.0 - ((x - 1.0).powf(2.0))).sqrt());
static_curve!(EaseInOutCirc (x) := {
    if x < 0.5 {
        (1.0 - (1.0 - (2.0 * x).powf(2.0)).sqrt()) / 2.0
    } else {
        (1.0 - (-2.0 * x + 2.0).powf(2.0)).sqrt() / 2.0
    }
});
