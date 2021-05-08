/// Interval type defined to be [a,b) (non-inclusive on the right)
/// A value of `None` represents the empty interval.
pub type Interval = Option<[i32; 2]>;

/// Create an interval.
pub fn interval(a: i32, b: i32) -> Interval {
    if a != b {
        Some([a, b])
    } else {
        None
    }
}

/// Computes the intersection of two intervals.
pub fn intersect(u: &Interval, v: &Interval) -> Interval {
    if u.is_none() || v.is_none() {
        return None;
    }
    let u = u.unwrap();
    let v = v.unwrap();
    assert_ne!(u[0], u[1]);
    assert_ne!(v[0], v[1]);
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
        assert_ne!(u[0], u[1]);
        point >= u[0] && point < u[1]
    } else {
        false
    }
}

/// Support data structure to keep track of the # of pixels that have been draw,
/// by keeping track of the pixels that are yet to be drawn for each column.
///
/// Each column is represented by an interval (the range of pixels not yet
/// drawn). You update the columns by intersecting it with a new interval (for
/// example, when a vertical line has been drawn).
///
/// Rendering should stop when this coverage has reached 100% (i.e., no pixels
/// remain to be drawn for every column).
#[derive(Debug)]
pub struct PixelCoverage {
    width: usize,
    height: usize,
    columns: Vec<Interval>,
    non_empty: usize,
}

impl PixelCoverage {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            columns: vec![Some([0, height as _]); width],
            non_empty: width,
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

    pub fn intersect(&mut self, column: usize, interval: &Interval) {
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
        let w = self.width as _;
        self.columns.iter_mut().for_each(|c| *c = Some([0, w]));
        self.non_empty = self.width;
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
        assert_interval!(Some([1, 2]), Some([0, 4]), Some([1, 2]));
        assert_interval!(Some([1, 2]), Some([0, 2]), Some([1, 2]));
    }

    #[test]
    fn reach_full_coverage() {
        let mut coverage = super::PixelCoverage::new(512, 512);
        for column in 0..512 {
            assert!(!coverage.is_full());
            coverage.intersect(column, &None);
        }
        assert!(coverage.is_full());
    }
}
