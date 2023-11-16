use crate::range::Ranging;

#[derive(Default, Debug, Clone)]
pub struct RangeSet(pub Vec<i32>);

#[derive(PartialEq, Eq)]
enum RangeSlot {
    Start,
    End,
}

struct PositionReport {
    occupied: bool,
    in_range: bool,
    /// Index of the range that got found or the one to the left
    range_start_index: usize,
    index: usize,
}

impl From<Result<usize, usize>> for PositionReport {
    fn from(value: Result<usize, usize>) -> Self {
        let (index, hit_n) = match value {
            Ok(i) => (i, true),
            Err(i) => (i, false),
        };

        let is_low = index % 2 == 0;
        let range_start_index = if is_low { index } else { index - 1 };

        // We're in range of an range if we found either some empty space behind a high or exactly the low
        let in_range = (!hit_n && !is_low) || (hit_n && is_low);

        // match value {
        //     Ok(index) => PositionReport {
        //         occupied: RangeSlot::for_index(index),
        //         in_range: index % 2 == 0,
        //         range_start_index: if index % 2 == 0 { index } else { index - 1 },
        //         index,
        //     }, // We matched a number exactly
        //     Err(index) => PositionReport {
        //         occupied: RangeSlot::Nothing,
        //         in_range: index % 2 == 1,
        //         range_start_index: if index % 2 == 0 { index } else { index - 1 },
        //         index,
        //     }, // I
        // }

        PositionReport {
            occupied: hit_n,
            in_range,
            range_start_index,
            index,
        }
    }
}

impl RangeSlot {
    fn for_index(n: usize) -> RangeSlot {
        if n % 2 == 0 {
            RangeSlot::Start
        } else {
            RangeSlot::End
        }
    }
}

pub struct RangeIterator<'a>
where
// T: Iterator<Item = &'a (i32, i32)>,
{
    rs: &'a RangeSet,
    index: usize,
}

impl<'a> Iterator for RangeIterator<'a> {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        let left = *self.rs.0.get(self.index)?;
        let right = *self.rs.0.get(self.index + 1)?;
        self.index += 2;

        Some((left, right))
    }
}

impl RangeSet {
    pub fn len(&self) -> usize {
        self.0.len() / 2
    }

    pub fn overlapping_ranges(&self, range: (i32, i32)) -> Vec<(usize, i32, i32)> {
        let mut out = vec![];
        let left_index = self.position_report(&range.0);
        let right_index = self.position_report(&range.1);

        let first_range_index = left_index.range_start_index;
        let last_range_index = if right_index.in_range {
            right_index.range_start_index
        } else {
            first_range_index
        };

        let mut cur_index = first_range_index;

        while cur_index < right_index.index {
            let low = self.0.get(cur_index);
            let high = self.0.get(cur_index + 1);

            if low.is_none() || high.is_none() {
                break;
            }

            out.push((cur_index, *low.unwrap(), *high.unwrap()));
            cur_index += 2;
        }

        out
    }

    // This has grown into an insane tree of edge cases that should be faster then the wastfully slow fallback option
    // I'd love to simplify this somewhat but oh dear
    pub fn insert(&mut self, new_range: (i32, i32)) {
        let len = self.0.len();
        let left_index = self.position_report(&new_range.0);
        let right_index = self.position_report(&new_range.1);

        let right_index_hit_marker = !right_index.in_range && right_index.occupied;

        if new_range == (-1, 6) {
            println!("oh no")
        }

        if left_index.index == len || len == 0 {
            // We're inserting beyond any exising range, or the vector is simply empty
            self.0.push(new_range.0);
            self.0.push(new_range.1);
            return;
        }

        //Insert before first range
        if left_index.index == 0 && right_index.index == 0 && !right_index.in_range {
            self.0.insert(0, new_range.1);
            self.0.insert(0, new_range.0);
            return;
        }

        // if (left_index.in_range && right_index.index == len) {
        //     // We've reached beyond the end of the vector, scrap everything between and insert the new end
        //     self.0.drain(left_index.index + 1..);
        //     self.0.push(new_range.1);
        //     return;
        // }

        // Extend first range
        // if left_index.index == 0 && left_index.index != right_index.index {
        //     *self.0.get_mut(0).unwrap() = new_range.0;
        // }

        // Left side is an exact hit on the last range and the new range extends beyond the array
        if left_index.in_range
            // && left_index.occupied
            && right_index.index == len
            && left_index.range_start_index + 2 == right_index.range_start_index
        {
            *self.0.last_mut().unwrap() = new_range.1;
            return;
        }

        if (left_index.index + 1 == right_index.index) {
            if (!left_index.in_range && (right_index.in_range || right_index_hit_marker)) {
                *self.0.get_mut(left_index.range_start_index).unwrap() = new_range.0;
                return;
            }

            if (left_index.in_range && right_index.in_range || right_index_hit_marker) {
                return;
            }
        }

        if left_index.index == right_index.index {
            // Left and right would be inserted in the same spot or next to eachother and thus are sequential

            if left_index.occupied && !left_index.in_range {
                // Left slot is an end, extend it
                *self.0.get_mut(left_index.index).unwrap() = new_range.1;
                return;
            }

            if right_index.occupied && right_index.in_range {
                // Right slot is a a start, extend it
                *self.0.get_mut(right_index.index).unwrap() = new_range.1;
                return;
            }

            if !left_index.in_range && !right_index.in_range {
                // No overlap with anything, just insert
                self.0.insert(left_index.index, new_range.1); // Insert upper first!
                self.0.insert(left_index.index, new_range.0);
                return;
            }

            if left_index.in_range && right_index.in_range {
                return; // We're fully overlapping an exsisting range, just ignore and abort
            }

            if left_index.in_range
                && (right_index.in_range || (right_index.occupied && !right_index.in_range))
            {
                // Left side and right side are in range or on the exact order. We're overlapped by the exsisting range, ignore
                return;
            }
        }

        if (left_index.range_start_index + 2 == right_index.range_start_index) {
            // Positions hit two different sequential ranges

            if (left_index.occupied && !left_index.in_range)
                || left_index.in_range
                || left_index.index == 0
            {
                if (right_index.in_range || right_index_hit_marker) {
                    // Hit two ranges, overlapping both, just remove the entries keeping them seperate
                    self.0.remove(left_index.range_start_index + 1);
                    self.0.remove(left_index.range_start_index + 1);
                    return;
                }
            }

            // We're entirely overlapping an existing range
            if (!left_index.in_range && !left_index.occupied && !right_index.in_range) {
                *self.0.get_mut(left_index.range_start_index).unwrap() = new_range.0;
                *self.0.get_mut(left_index.range_start_index + 1).unwrap() = new_range.1;
                return;
            }
        }

        if left_index.index + 1 == right_index.index && !right_index.in_range {
            if left_index.occupied && !left_index.in_range {
                *self.0.get_mut(left_index.index).unwrap() = new_range.1;
                return;
            }
        }

        println!("SLOW {}, {}", new_range.0, new_range.1);
        // *c += 1;
        let overlaps = self.overlapping_ranges(new_range);
        // assert_ne!(overlaps.len(), 1); // Any code above should have handled the simple cases
        let mut remove_counter = 0;
        let mut range_accumelator = new_range;
        for overlap in overlaps.into_iter() {
            range_accumelator = range_accumelator.merge(&(overlap.1, overlap.2));
            self.0.remove(overlap.0 - remove_counter);
            self.0.remove(overlap.0 - remove_counter);
            remove_counter += 2;
        }

        self.insert(range_accumelator)
    }

    pub fn size(&self) -> i32 {
        self.iter_ranges().map(|r| r.range_size()).sum()
    }

    pub fn iter_ranges(&self) -> RangeIterator {
        RangeIterator { rs: self, index: 0 }
    }

    // fn index_of_n(&self, n: i32) -> usize {
    //     match self.0.binary_search(&n) {
    //         Ok(index) => index,
    //         Err(index) => index,
    //     }
    // }

    fn position_report(&self, n: &i32) -> PositionReport {
        self.0.binary_search(n).into() // If we got an error, check if the index is even or uneven
    }

    pub fn is_in_range(&self, n: i32) -> bool {
        self.position_report(&n).in_range
    }

    pub fn remove(&mut self, cut: (i32, i32)) {
        let len = self.0.len();
        let left_index = self.position_report(&cut.0);
        let right_index = self.position_report(&cut.1);
        if len == left_index.index {
            // Nothing to remove
            return;
        }

        // if !left_index.in_range && !right_index.in_range {
        //     return; // Nothing is in range, don't do anything
        // }

        if left_index.range_start_index == right_index.range_start_index {
            // Simple case, only one other range
            let low = self.0.get(left_index.range_start_index);

            if low.is_none() {
                // We're beyond any other range, ignore
                return;
            }
            let low = low.unwrap();

            let high = self
                .0
                .get(left_index.range_start_index + 1)
                .expect("range_start_index + 1 to exist");

            if !(*low, *high).overlaps(&cut) {
                // We don't overlap with the sole other range, ignore
                return;
            }

            if left_index.occupied && left_index.in_range {
                if right_index.occupied && !right_index.in_range {
                    // We match the sole other range exactly, remove it
                    self.0.remove(left_index.index);
                    self.0.remove(left_index.index); // Same index, popping shifts the second one back
                    return;
                }

                if *high > cut.1 {
                    // Left matches exactly, right extends beyond cut, adjust left
                    *self.0.get_mut(left_index.range_start_index).unwrap() = cut.1;
                    return;
                }
            }

            // if(right_index.occupied)

            if cut.contains_exclusive(&(*low, *high)) {
                // Cut entirely encompasses range, remove it

                self.0.remove(left_index.index);
                self.0.remove(left_index.index);
                return;

                // drop(low);
                // drop(high);
            }
        }

        // Complex situation, just scan, remove and re-insert
        // println!("w2");
        let ranges = self.overlapping_ranges(cut);
        let mut remove_count = 0;
        let mut new_to_insert = vec![];

        ranges.iter().for_each(|(index, low, high)| {
            self.0.remove(index - remove_count);
            self.0.remove(index - remove_count);
            remove_count += 2;

            new_to_insert.extend((*low, *high).remove(&cut));
        });

        new_to_insert.into_iter().for_each(|r| self.insert(r))
    }
}

#[cfg(test)]
mod tests {
    use std::default;

    use super::*;

    fn expect<T>(a: T, b: T, msg: &'static str) -> Result<(), String>
    where
        T: Eq,
    {
        match a.eq(&b) {
            true => Ok(()),
            false => {
                let mut str = String::from("Expected ");
                str.push_str(msg);
                Err(str)
            }
        }
    }

    #[test]
    fn insert() {
        let mut range = RangeSet::default();
        range.insert((5, 10));

        assert_eq!(range.len(), 1);

        range.insert((15, 20));

        assert_eq!(range.len(), 2);

        range.insert((1, 3));

        assert_eq!(range.len(), 3);
    }
    #[test]
    fn insert_merge_right() {
        let mut range = RangeSet::default();
        range.insert((5, 10));
        range.insert((10, 15));

        assert_eq!(range.len(), 1);
    }
    #[test]
    fn insert_merge_left() {
        let mut range = RangeSet::default();
        range.insert((10, 15));
        range.insert((5, 10));

        assert_eq!(range.len(), 1);
    }
    #[test]
    fn in_range() -> Result<(), String> {
        let mut range = RangeSet::default();
        range.insert((5, 10));
        expect(range.is_in_range(5), true, "5 to be in range")?;
        expect(range.is_in_range(9), true, "9 to be in range")?;

        expect(range.is_in_range(10), false, "10 to be out of range")?;
        expect(range.is_in_range(4), false, "4 to be out of range")?;

        Ok(())
    }

    #[test]
    fn remove_exact() {
        let mut range = RangeSet::default();
        range.insert((10, 20));
        assert_eq!(range.len(), 1);
        range.remove((10, 20));
        assert_eq!(range.len(), 0);
    }

    #[test]
    fn dont_be_slow_when_inserting_beyond_end() {
        let mut range = RangeSet::default();
        range.insert((3, 5));
        range.insert((8, 10));
        range.insert((13, 15));
        range.insert((9, 16));

        assert_eq!(range.len(), 2)
    }

    #[test]
    // #[ignore = "reason"]
    fn remove_center() {
        let mut range = RangeSet::default();
        range.insert((10, 20));

        range.remove((12, 15));

        assert_eq!(range.len(), 2);

        let ranges: Vec<(i32, i32)> = range.iter_ranges().collect();

        assert_eq!(*ranges.get(0).unwrap(), (10, 12));
        assert_eq!(*ranges.get(1).unwrap(), (15, 20))
    }

    #[test]
    // #[ignore = "reason"]
    fn remove_all() {
        let mut range = RangeSet::default();
        range.insert((10, 20));

        range.remove((9, 21));

        assert_eq!(range.len(), 0);
    }

    #[test]
    fn remove_overlap_lower() {
        let mut range = RangeSet::default();
        range.insert((10, 20));

        range.remove((5, 15));

        assert_eq!(range.len(), 1);

        assert_eq!(range.iter_ranges().next().unwrap(), (15, 20));
    }

    #[test]
    fn remove_overlap_upper() {
        let mut range = RangeSet::default();
        range.insert((10, 20));

        range.remove((15, 25));

        assert_eq!(range.len(), 1);

        assert_eq!(range.iter_ranges().next().unwrap(), (10, 15));
    }

    #[test]
    fn remove() {
        let mut rs = RangeSet::default();
        rs.insert((17, 21));

        rs.remove((20, 21));

        assert_eq!(rs.len(), 1);

        assert_eq!(rs.iter_ranges().next().unwrap(), (17, 20));
    }

    #[test]
    fn remove_more() {
        //[0, 6, 11, 12, 15, 21]

        let mut rs = RangeSet::default();
        rs.insert((0, 6));
        rs.insert((11, 12));
        rs.insert((15, 21));

        rs.remove((-8, 13));

        assert_eq!(rs.len(), 1);

        assert_eq!(rs.iter_ranges().next().unwrap(), (15, 21));
    }

    #[test]
    fn overlapping_ranges() {
        let mut rs = RangeSet::default();
        rs.insert((6, 8));
        rs.insert((17, 21));

        let overlap = rs.overlapping_ranges((6, 11));
        assert_eq!(overlap, vec![(0, 6, 8)])
    }

    #[test]
    fn edge_cases() {
        {
            let mut range = RangeSet::default();
            range.insert((10, 20));
            range.remove((5, 10));
            assert_eq!(range.iter_ranges().next().unwrap(), (10, 20));
        }
        {
            let mut range = RangeSet::default();
            range.insert((10, 20));
            range.remove((15, 20));
            assert_eq!(
                range.iter_ranges().next().unwrap(),
                (10, 15),
                "Should properly trim right"
            );
        }
        {
            let mut range = RangeSet::default();
            range.insert((10, 20));
            range.remove((10, 15));
            assert_eq!(
                range.iter_ranges().next().unwrap(),
                (15, 20),
                "Should properly trim left"
            );
        }
    }
}
