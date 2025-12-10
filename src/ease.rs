use common::cli::client::EaseKind;

pub trait Curve {
    fn f(&self, x: f64) -> f64;
}

pub mod cubic_bezier;
pub mod static_curves;

pub fn ease_with<C>(from: f64, to: f64, progress: f64, curve: &C) -> f64
where
    C: Curve,
{
    from + (to - from) * curve.f(progress)
}

impl<T: Curve + ?Sized> Curve for Box<T> {
    fn f(&self, x: f64) -> f64 {
        (**self).f(x)
    }
}

pub fn create_easing_curve(ease_kind: EaseKind) -> Box<dyn Curve> {
    match ease_kind {
        EaseKind::No | EaseKind::Linear => Box::new(static_curves::Linear),

        EaseKind::CubicBezier(px1, py1, px2, py2) => Box::new(cubic_bezier::CubicBezier::new(
            mint::Vector2::from_slice(&[px1, py1]),
            mint::Vector2::from_slice(&[px2, py2]),
        )),

        EaseKind::Hold => Box::new(static_curves::Hold),
        EaseKind::Step => Box::new(static_curves::Step),
        EaseKind::EaseInQuad => Box::new(static_curves::EaseInQuad),
        EaseKind::EaseOutQuad => Box::new(static_curves::EaseOutQuad),
        EaseKind::EaseInOutQuad => Box::new(static_curves::EaseInOutQuad),
        EaseKind::EaseInCubic => Box::new(static_curves::EaseInCubic),
        EaseKind::EaseOutCubic => Box::new(static_curves::EaseOutCubic),
        EaseKind::EaseInOutCubic => Box::new(static_curves::EaseInOutCubic),
        EaseKind::EaseInQuart => Box::new(static_curves::EaseInQuart),
        EaseKind::EaseOutQuart => Box::new(static_curves::EaseOutQuart),
        EaseKind::EaseInOutQuart => Box::new(static_curves::EaseInOutQuart),
        EaseKind::EaseInQuint => Box::new(static_curves::EaseInQuint),
        EaseKind::EaseOutQuint => Box::new(static_curves::EaseOutQuint),
        EaseKind::EaseInOutQuint => Box::new(static_curves::EaseInOutQuint),
        EaseKind::EaseInSine => Box::new(static_curves::EaseInSine),
        EaseKind::EaseOutSine => Box::new(static_curves::EaseOutSine),
        EaseKind::EaseInOutSine => Box::new(static_curves::EaseInOutSine),
        EaseKind::EaseInExpo => Box::new(static_curves::EaseInExpo),
        EaseKind::EaseOutExpo => Box::new(static_curves::EaseOutExpo),
        EaseKind::EaseInOutExpo => Box::new(static_curves::EaseInOutExpo),
        EaseKind::EaseInCirc => Box::new(static_curves::EaseInCirc),
        EaseKind::EaseOutCirc => Box::new(static_curves::EaseOutCirc),
        EaseKind::EaseInOutCirc => Box::new(static_curves::EaseInOutCirc),
    }
}
