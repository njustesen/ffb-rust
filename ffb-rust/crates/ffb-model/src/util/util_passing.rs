use crate::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.util.UtilPassing`.
///
/// Contains the `findInterceptors` and `findValidPassBlockEndCoordinates` methods
/// in the Java source.  These methods need a live `Game` reference and are
/// therefore fully implemented in `ffb-engine`.  The standalone geometry helpers
/// are ported here.
pub struct UtilPassing;

/// Java: `UtilPassing.RULER_WIDTH = 1.74`.
pub const RULER_WIDTH: f64 = 1.74;

impl UtilPassing {
    /// Java: private `canIntercept(thrower, target, interceptor)`.
    ///
    /// Returns `true` if `interceptor` lies within `RULER_WIDTH` of the line from
    /// `thrower` to `target` and is between thrower and target.
    pub fn can_intercept(
        thrower: FieldCoordinate,
        target: FieldCoordinate,
        interceptor: FieldCoordinate,
    ) -> bool {
        let rx = (target.x - thrower.x) as f64;
        let ry = (target.y - thrower.y) as f64;
        let ix = (interceptor.x - thrower.x) as f64;
        let iy = (interceptor.y - thrower.y) as f64;
        let a = (rx - ix).powi(2) + (ry - iy).powi(2);
        let b = ix.powi(2) + iy.powi(2);
        let c = rx.powi(2) + ry.powi(2);
        let d1 = (ry * (ix + 0.5) - rx * (iy + 0.5)).abs();
        let d2 = (ry * (ix + 0.5) - rx * (iy - 0.5)).abs();
        let d3 = (ry * (ix - 0.5) - rx * (iy + 0.5)).abs();
        let d4 = (ry * (ix - 0.5) - rx * (iy - 0.5)).abs();
        let min_d = d1.min(d2).min(d3).min(d4);
        (c > a) && (c > b) && (RULER_WIDTH > 2.0 * min_d / c.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_intercept_on_line() {
        // Square exactly in the middle of a straight pass (0,0) → (10,0)
        // interceptor at (5,0) should be interceptable.
        let thrower   = FieldCoordinate::new(0, 0);
        let target    = FieldCoordinate::new(10, 0);
        let mid       = FieldCoordinate::new(5, 0);
        assert!(UtilPassing::can_intercept(thrower, target, mid));
    }

    #[test]
    fn cannot_intercept_far_aside() {
        let thrower   = FieldCoordinate::new(0, 0);
        let target    = FieldCoordinate::new(10, 0);
        let aside     = FieldCoordinate::new(5, 5); // way off the line
        assert!(!UtilPassing::can_intercept(thrower, target, aside));
    }
}
