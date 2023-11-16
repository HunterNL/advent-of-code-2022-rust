#[derive(Clone, PartialEq, Eq)]
pub struct Range {
    pub low: i32,
    pub high: i32,
}

pub trait Ranging {
    // fn from_ordered(low: i32, high: i32) -> Range;

    // fn from_unordered(a: i32, b: i32) -> Range;

    fn range_size(&self) -> i32;

    fn overlaps(&self, other: &Self) -> bool;

    fn contains_exclusive(&self, other: &Self) -> bool;

    fn contains_inclusive(&self, other: &Self) -> bool;

    fn overlap(&self, other: &Self) -> (i32, i32);

    fn touches(&self, other: &Self) -> bool;

    fn remove(&self, cut: &Self) -> Vec<(i32, i32)>;

    fn merge(&self, other: &Self) -> Self;
}

fn from_ordered(low: i32, high: i32) -> Range {
    Range { low, high }
}

fn from_unordered(a: i32, b: i32) -> Range {
    if a < b {
        Range { low: a, high: b }
    } else {
        Range { low: b, high: a }
    }
}

impl Ranging for (i32, i32) {
    fn range_size(&self) -> i32 {
        self.1 - self.0
    }

    fn overlaps(&self, other: &Self) -> bool {
        if other.1 < self.0 {
            return false;
        }
        if other.0 > self.1 {
            return false;
        }
        true
    }

    // Inclusive contain, identical ranges match
    fn contains_inclusive(&self, other: &Self) -> bool {
        self.0 <= other.0 && self.1 >= other.1
    }

    // Must outrange entirely
    fn contains_exclusive(&self, other: &Self) -> bool {
        self.0 < other.0 && self.1 > other.1
    }

    fn overlap(&self, other: &Self) -> (i32, i32) {
        if self.overlaps(other) {
            return other.clone();
        }
        if other.overlaps(self) {
            return self.clone();
        }

        if self.1 > other.1 {
            (self.0, other.1)
        } else {
            (other.0, self.1)
        }
    }

    fn touches(&self, other: &Self) -> bool {
        if other.1 == self.0 {
            return true;
        }
        if other.0 == self.1 {
            return true;
        }
        false
    }

    fn remove(&self, cut: &Self) -> Vec<(i32, i32)> {
        if cut.contains_inclusive(self) {
            return vec![];
        }

        if self.contains_exclusive(cut) {
            return vec![(self.0, cut.0), (cut.1, self.1)];
        }

        if self.1 > cut.1 {
            return vec![(cut.1, self.1)];
        }

        if self.0 < cut.0 {
            return vec![(self.0, cut.0)];
        }

        panic!("Unknown state")
    }

    fn merge(&self, other: &Self) -> Self {
        (self.0.min(other.0), self.1.max(other.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove() {
        let range = (17, 21);
        let cut = (20, 21);

        assert_eq!(range.remove(&cut), vec![(17, 20)])
    }
}
