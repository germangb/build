use crate::frame;

/// Interval type defined to be [a,b) (non-inclusive on the right)
/// A value of `None` represents the empty interval.
pub type Interval = Option<[i32; 2]>;

/// Compute the intersection of two intervals
/// If the intersection is an empty set, returns None.
pub fn intersect(u: &Interval, v: &Interval) -> Interval {
    if u.is_none() || v.is_none() {
        return None;
    }
    let u = u.unwrap();
    let v = v.unwrap();
    if u[0] > v[0] {
        return intersect(&Some(v), &Some(u));
    }
    if u[1] <= v[0] {
        None
    } else {
        Some([v[0], v[1].min(u[1])])
    }
}

/// Tests if the given `point` is inside of the interval `u`.
pub fn contains(u: &Interval, point: i32) -> bool {
    if let Some(u) = u {
        point >= u[0] && point < u[1]
    } else {
        false
    }
}

/// Support data structure to track the the # of pixels that have been draw, by
/// keeping track of the pixels that are yet to be draw for each column.
/// Rendering should stop when this coverage reaches 100% (or when a maximum
/// sector-depth is reached).
///
/// # Notes
/// This way of tracking pixel coverage has the side-effect that the renderer is
/// not able to do true "sector-over-sector" rendering.
#[derive(Debug)]
pub struct Coverage {
    columns: Vec<Interval>,
    non_empty: usize,
}

impl Coverage {
    pub fn new() -> Self {
        Self {
            columns: vec![Some([0, frame::HEIGHT as _]); frame::WIDTH],
            non_empty: frame::WIDTH,
        }
    }

    /// Returns true if the pixel coverage is 100%.
    pub fn is_full(&self) -> bool {
        self.non_empty == 0
    }

    /// Returns the interval of pixels that are still to be drawn in the column.
    pub fn get_column(&self, column: usize) -> Interval {
        if self.is_full() {
            None
        } else {
            self.columns[column]
        }
    }

    pub fn intersect_column(&mut self, column: usize, interval: &Interval) {
        if self.is_full() {
            return;
        }
        let u = self.columns[column];
        self.columns[column] = intersect(&u, interval);
        if u.is_some() && self.columns[column].is_none() {
            self.non_empty -= 1;
        }
    }

    /// Force pixel coverage to be 0%.
    pub fn reset(&mut self) {
        self.columns
            .iter_mut()
            .for_each(|c| *c = Some([0, frame::HEIGHT as _]));
        self.non_empty = frame::WIDTH;
    }

    /// Force pixel coverage to be 100%.
    pub fn set_full_coverage(&mut self) {
        self.non_empty = 0;
    }
}

#[cfg(test)]
mod test {
    macro_rules! assert_interval {
        ($eq:expr, $u:expr, $v:expr) => {
            assert_eq!($eq, super::intersect(&$u, &$v));
            assert_eq!($eq, super::intersect(&$v, &$u));
        };
    }

    #[test]
    fn contains() {
        assert!(!super::contains(&None, 42));
        assert!(super::contains(&Some([0, 2]), 0));
        assert!(super::contains(&Some([0, 2]), 1));
        assert!(!super::contains(&Some([0, 2]), 2));
    }

    #[test]
    fn intersect() {
        // empty
        assert_interval!(None, Some([0, 1]), Some([1, 2]));
        assert_interval!(None, Some([0, 1]), Some([2, 3]));
        assert_interval!(None, None, Some([1, 2]));
        assert_interval!(None, Some([0, 1]), None);
        assert_interval!(None, None, None);
        // non-empty
        assert_interval!(Some([0, 1]), Some([0, 1]), Some([0, 1]));
        assert_interval!(Some([1, 1]), Some([0, 4]), Some([1, 1]));
        assert_interval!(Some([1, 2]), Some([0, 2]), Some([1, 2]));
    }

    #[test]
    fn coverage_set_full_coverage_and_reset() {
        let mut coverage = super::Coverage::new();
        assert!(!coverage.is_full());
        coverage.set_full_coverage();
        assert!(coverage.is_full());
        coverage.reset();
        assert!(!coverage.is_full());
    }

    #[test]
    fn reach_full_coverage() {
        let mut coverage = super::Coverage::new();
        for column in 0..crate::frame::WIDTH {
            assert!(!coverage.is_full());
            coverage.intersect_column(column, &None);
        }
        assert!(coverage.is_full());
    }
}
