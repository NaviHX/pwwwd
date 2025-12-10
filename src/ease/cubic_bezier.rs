use crate::ease::Curve;
use mint::Vector2;

pub struct CubicBezier {
    p1: Vector2<f64>,
    p2: Vector2<f64>,
    sample_table: [f64; SAMPLE_TABLE_SIZE],
}

const SAMPLE_TABLE_SIZE: usize = 20;
const STEP_SIZE: f64 = 1.0 / (SAMPLE_TABLE_SIZE - 1) as f64;
const NEWTON_ITERATIONS: usize = 4;
const NEWTON_MIN_SLOPE: f64 = 1e-2;
const BINARY_SEARCH_PRECISION: f64 = 1e-6;

impl CubicBezier {
    #[inline]
    fn xa(&self) -> f64 {
        3.0 * self.p1.x - 3.0 * self.p2.x + 1.0
    }

    #[inline]
    fn xb(&self) -> f64 {
        -6.0 * self.p1.x + 3.0 * self.p2.x
    }

    #[inline]
    fn xc(&self) -> f64 {
        3.0 * self.p1.x
    }

    #[inline]
    fn xd(&self) -> f64 {
        0.0
    }

    #[inline]
    fn ya(&self) -> f64 {
        3.0 * self.p1.y - 3.0 * self.p2.y + 1.0
    }

    #[inline]
    fn yb(&self) -> f64 {
        -6.0 * self.p1.y + 3.0 * self.p2.y
    }

    #[inline]
    fn yc(&self) -> f64 {
        3.0 * self.p1.y
    }

    #[inline]
    fn yd(&self) -> f64 {
        0.0
    }

    #[inline]
    pub fn x(&self, t: f64) -> f64 {
        ((self.xa() * t + self.xb()) * t + self.xc()) * t + self.xd()
    }

    #[inline]
    pub fn y(&self, t: f64) -> f64 {
        ((self.ya() * t + self.yb()) * t + self.yc()) * t + self.yd()
    }

    #[inline]
    pub fn x_derive(&self, t: f64) -> f64 {
        (self.xa() * 3.0 * t + self.xb() * 2.0) * t + self.xc()
    }

    fn newton_raphson(&self, x: f64, mut guess: f64) -> f64 {
        for _ in 0..NEWTON_ITERATIONS {
            let slope = self.x_derive(guess);
            if slope == 0.0 {
                break;
            }

            let current_x = self.x(guess) - x;
            guess -= current_x / slope;
        }

        guess
    }

    fn binary_search(&self, x: f64, mut l: f64, mut r: f64) -> f64 {
        let mut current_t = 0.0;
        let mut current_x: f64 = 0.0;
        let mut i = 0;
        let mut has_run_once = false;

        while !has_run_once || current_x.abs() > BINARY_SEARCH_PRECISION {
            has_run_once |= true;
            current_t = l + (r - l) / 2.0;
            current_x = self.x(current_t) - x;

            if current_x > 0.0 {
                r = current_t;
            } else {
                l = current_t;
            }

            i += 1;
        }

        current_t
    }

    pub fn t_for_x(&self, x: f64) -> f64 {
        let interval_includes_x = self
            .sample_table
            .windows(2)
            .enumerate()
            .map(|(i, borders)| (i, borders[0], borders[1]))
            .find(|&(_, l, r)| l <= x && x <= r);

        match interval_includes_x {
            Some((i, l, r)) => {
                let dist = x - l;
                let guess_for_t = (i as f64 + dist) * STEP_SIZE;
                let slope = self.x_derive(guess_for_t);

                if slope >= NEWTON_MIN_SLOPE {
                    self.newton_raphson(x, guess_for_t)
                } else {
                    self.binary_search(x, l, r)
                }
            }

            // If `x` is out of range, clamp to border.
            None => {
                if x > 1.0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    pub fn new(p1: Vector2<f64>, p2: Vector2<f64>) -> Self {
        let mut curve = Self {
            p1,
            p2,
            sample_table: [0.0; SAMPLE_TABLE_SIZE],
        };

        for i in 0..SAMPLE_TABLE_SIZE {
            curve.sample_table[i] = curve.x(i as f64 * STEP_SIZE);
        }

        curve
    }
}

impl Curve for CubicBezier {
    fn f(&self, x: f64) -> f64 {
        let t = self.t_for_x(x);
        self.y(t)
    }
}
