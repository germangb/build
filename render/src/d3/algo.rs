/// 1D open-ended Interval.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Interval([i32; 2]);

impl Interval {
    pub const EMPTY: Self = Self([0, 0]);

    pub fn left(&self) -> i32 {
        self.0[0]
    }

    pub fn right(&self) -> i32 {
        self.0[1]
    }

    pub fn new(l: i32, r: i32) -> Self {
        assert!(l <= r);
        Self([l, r])
    }

    pub fn contains(&self, point: i32) -> bool {
        !self.is_empty() && point >= self.0[0] && point <= self.0[1]
    }

    #[rustfmt::skip]
    pub fn intersect(&self, other: &Self) -> Self {
        if self.is_empty() || other.is_empty() { return Self::EMPTY; }
        if self.0[0] > other.0[0] { return other.intersect(self); }
        assert!(self.0[0] <= other.0[0]);
        if self.0[1] <= other.0[0] { return Self::EMPTY; }
        Self::new(other.0[0], other.0[1].min(self.0[1]))
    }

    pub fn is_empty(&self) -> bool {
        return self.0[1] <= self.0[0];
    }
}

/// To track window pixel coverage.
#[derive(Debug)]
pub struct Coverage {
    width: usize,
    height: usize,
    columns: Vec<Interval>,
    // number of empty intervals in 'columns'
    empty: usize,
}

impl Coverage {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            columns: vec![Interval::new(0, height as i32); width],
            empty: 0,
        }
    }

    pub fn intersect(&mut self, column: usize, int: &Interval) -> Interval {
        assert!(column < self.width);
        if self.columns[column].is_empty() {
            return Interval::EMPTY;
        }
        let int = self.columns[column].intersect(int);
        self.columns[column] = int;
        if self.columns[column].is_empty() {
            self.empty += 1;
        }
        return int;
    }

    pub fn column(&self, idx: usize) -> &Interval {
        &self.columns[idx]
    }

    /// Returns true if the pixel coverage is 100% i.e. there are no more pixels
    /// left to render in the window.
    pub fn is_full(&self) -> bool {
        return self.empty == self.width;
    }

    /// Reset pixel coverage to 0%
    pub fn clear(&mut self) {
        let h = self.height as i32;
        self.columns
            .iter_mut()
            .for_each(|int| *int = Interval::new(0, h));
        self.empty = 0;
    }
}

#[cfg(test)]
mod tests2 {
    use super::{Coverage, Interval};
    #[test]
    fn intersect_nonempty() {
        assert_eq!(
            Interval::new(0, 1),
            Interval::new(0, 1).intersect(&Interval::new(0, 1))
        );
        assert_eq!(
            Interval::new(0, 1),
            Interval::new(0, 1).intersect(&Interval::new(-2, 2))
        );
        assert_eq!(
            Interval::new(1, 2),
            Interval::new(1, 2).intersect(&Interval::new(0, 4))
        );
        assert_eq!(
            Interval::new(1, 2),
            Interval::new(0, 4).intersect(&Interval::new(1, 2))
        );
        assert_eq!(
            Interval::new(1, 2),
            Interval::new(0, 2).intersect(&Interval::new(1, 2))
        );
    }
    #[test]
    fn intersect_empty() {
        assert!(Interval::new(0, 1)
            .intersect(&Interval::new(1, 2))
            .is_empty());
        assert!(Interval::new(1, 2)
            .intersect(&Interval::new(0, 1))
            .is_empty());
        assert!(Interval::new(2, 3)
            .intersect(&Interval::new(0, 1))
            .is_empty());
        assert!(Interval::new(0, 1)
            .intersect(&Interval::new(2, 3))
            .is_empty());
        assert!(Interval::EMPTY.intersect(&Interval::new(0, 4)).is_empty());
        assert!(Interval::new(0, 4).intersect(&Interval::EMPTY).is_empty());
    }

    #[test]
    fn coverage() {
        let mut cov = Coverage::new(32, 32);
        for i in 0..32 {
            assert!(!cov.is_full());
            cov.intersect(i, &Interval::EMPTY);
            cov.intersect(i, &Interval::EMPTY);
        }
        assert!(cov.is_full());
    }
}
