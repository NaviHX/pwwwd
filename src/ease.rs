use std::borrow::Borrow;

pub trait Curve {
    fn f(&self, x: f64) -> f64;
}

// pub mod cubic_bezier;
pub mod static_curves;

fn ease_with<C>(from: f64, to: f64, progress: f64, curve: impl Borrow<C>) -> f64
where
    C: Curve,
{
    from + (to - from) * curve.borrow().f(progress)
}
