use std::{
    char,
    collections::HashSet,
    fmt::{self, Display, Write},
    str::FromStr,
};

use crate::vec2d::Vec2D;

use super::{DayOutput, LogicError};

const SEARCH_MAX_P1: i32 = 20;
const SEARCH_MAX_P2: i32 = 4000000;

fn is_number_char(char: &char) -> bool {
    char.is_ascii_digit() || char == &'-'
}
fn consume_number_from_char_iter<T>(iter: &mut T) -> i32
where
    T: Iterator<Item = char>,
{
    let chars: String = iter
        .skip_while(|char| !is_number_char(char))
        .take_while(is_number_char)
        .collect();

    chars.parse().expect("Chars to parse into numbers")
}

#[derive(Debug)]
struct Sensor {
    position: Vec2D<i32>,
    beacon_position: Vec2D<i32>,
    radius: i32,
}

impl Sensor {
    fn range_on_y_line(&self, y: i32) -> Option<Range> {
        let diff_y = (self.position.y - y).abs();
        let half_line_count = self.radius - diff_y;
        if half_line_count < 0 {
            None
        } else {
            Some(Range {
                lower: self.position.x - half_line_count.max(0),
                upper: self.position.x + half_line_count.max(0),
            })
        }
    }
}

impl FromStr for Sensor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut char_iter = s.chars();
        let pos_x = consume_number_from_char_iter(&mut char_iter);
        let pos_y = consume_number_from_char_iter(&mut char_iter);
        let sensor_x = consume_number_from_char_iter(&mut char_iter);
        let sensor_y = consume_number_from_char_iter(&mut char_iter);

        let position = Vec2D { x: pos_x, y: pos_y };
        let beacon_position = Vec2D {
            x: sensor_x,
            y: sensor_y,
        };

        Ok(Sensor {
            position,
            beacon_position,
            radius: position.distance_manhatten(&beacon_position),
        })
    }
}

#[derive(Debug)]
struct RangeSet {
    ranges: Vec<Range>,
}

impl Display for RangeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let max = self.ranges.iter().map(|r| r.upper).max().unwrap();

        for n in 0..=max {
            if self.is_in_range(n) {
                f.write_char('X')?
            } else {
                f.write_char(' ')?
            }
        }

        Ok(())
    }
}

impl RangeSet {
    fn insert(&mut self, mut new_range: Range) {
        loop {
            let merge_candidate = self
                .ranges
                .iter()
                .position(|range| range.overlaps(&new_range) || range.touches(&new_range));

            if let Some(overlapping_index) = merge_candidate {
                let old_range = self.ranges.remove(overlapping_index);
                new_range = old_range.merge(new_range)
            } else {
                self.ranges.push(new_range);
                return;
            }
        }
    }

    fn is_in_range(&self, n: i32) -> bool {
        self.ranges.iter().any(|r| n >= r.lower && n <= r.upper)
    }

    fn free_spot(&self, max: i32) -> bool {
        if self.ranges.len() == 1 {
            let range = self.ranges.get(0).unwrap();
            range.lower > 0 || range.upper < max
            // return self.ranges.get(0);
        } else {
            todo!()
        }
    }
}

#[derive(Debug)]
struct Range {
    lower: i32,
    upper: i32,
}

impl Range {
    fn overlaps(&self, other: &Self) -> bool {
        if other.upper < self.lower {
            return false;
        }
        if other.lower > self.upper {
            return false;
        }
        true
    }

    fn touches(&self, other: &Self) -> bool {
        if other.upper == self.lower - 1 {
            return true;
        }
        if other.lower == self.upper + 1 {
            return true;
        }
        false
    }

    fn merge(self, other: Self) -> Self {
        Range {
            lower: self.lower.min(other.lower),
            upper: self.upper.max(other.upper),
        }
    }

    fn size(&self) -> i32 {
        (self.upper - self.lower) + 1
    }
}

fn line_overlap_count(sensors: &[Sensor], y: i32) -> i32 {
    let mut set = RangeSet { ranges: vec![] };
    let beacon_set: HashSet<Vec2D<i32>> = sensors.iter().map(|s| s.beacon_position).collect();
    let beacons: Vec<Vec2D<i32>> = beacon_set.into_iter().collect();

    sensors
        .iter()
        .filter_map(|s| s.range_on_y_line(y))
        .for_each(|r| set.insert(r));

    let overlap_count: i32 = set.ranges.iter().map(|r| r.size()).sum();

    let beacons_in_range = beacons
        .iter()
        .filter(|beacon_pos| beacon_pos.y == y)
        .filter(|beacon_pos| set.is_in_range(beacon_pos.x))
        .count();

    overlap_count - beacons_in_range as i32
}

fn make_sensors(input: &str) -> Vec<Sensor> {
    input
        .lines()
        .map(|s| s.parse::<Sensor>().unwrap())
        .collect()
}

// https://adventofcode.com/2022/day/15
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let sensors = make_sensors(input);
    Ok(DayOutput {
        part1: Some(super::PartResult::Int(line_overlap_count(
            &sensors, 2_000_000,
        ))),
        part2: Some(super::PartResult::Int(0)),
    })
}

#[cfg(test)]
mod tests {

    use crate::solutions::day15::{line_overlap_count, Range};

    use super::{make_sensors, RangeSet, Sensor};

    #[test]
    // #[ignore = "wip"]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(15, super::solve)
    }

    #[test]
    fn example() {
        let input = "Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3";

        let sensors = make_sensors(input);

        assert_eq!(line_overlap_count(&sensors, 10), 26)
    }

    #[test]
    fn radius() {
        let sensor: Sensor = "Sensor at x=0, y=0: closest beacon is at x=1, y=0"
            .parse()
            .unwrap();

        assert_eq!(sensor.radius, 1)
    }

    #[test]
    fn size_on_line() {
        {
            let sensor: Sensor = "Sensor at x=5, y=0: closest beacon is at x=7, y=0"
                .parse()
                .unwrap();

            let range = sensor.range_on_y_line(1);

            let range = range.unwrap();

            assert_eq!(range.lower, 4);
            assert_eq!(range.upper, 6);
            assert_eq!(range.size(), 3)
        }
        {
            let sensor: Sensor = "Sensor at x=5, y=0: closest beacon is at x=7, y=0"
                .parse()
                .unwrap();

            let range = sensor.range_on_y_line(0);

            let range = range.unwrap();

            assert_eq!(range.lower, 3);
            assert_eq!(range.upper, 7);
            assert_eq!(range.size(), 5);
        }
        {
            let sensor: Sensor = "Sensor at x=5, y=0: closest beacon is at x=7, y=0"
                .parse()
                .unwrap();

            let range = sensor.range_on_y_line(2);

            let range = range.unwrap();

            assert_eq!(range.lower, 5);
            assert_eq!(range.upper, 5);
            assert_eq!(range.size(), 1);
        }
        {
            let sensor: Sensor = "Sensor at x=5, y=0: closest beacon is at x=7, y=0"
                .parse()
                .unwrap();

            let range = sensor.range_on_y_line(3);

            println!("{:?}", range);

            assert!(range.is_none())
        }
    }

    #[test]
    fn range_set_merge() {
        let mut set = RangeSet { ranges: vec![] };

        set.insert(super::Range { lower: 0, upper: 1 });

        set.insert(super::Range { lower: 1, upper: 2 });

        assert_eq!(set.ranges.len(), 1);

        let entry = set.ranges.remove(0);

        assert_eq!(entry.lower, 0);
        assert_eq!(entry.upper, 2);
    }

    #[test]
    fn range_set_touching_merge() {
        let mut set = RangeSet { ranges: vec![] };

        set.insert(super::Range { lower: 0, upper: 1 });

        set.insert(super::Range { lower: 2, upper: 3 });

        assert_eq!(set.ranges.len(), 1);

        let entry = set.ranges.remove(0);

        assert_eq!(entry.lower, 0);
        assert_eq!(entry.upper, 3);
    }

    #[test]
    fn size() {
        let r = Range { lower: 2, upper: 3 };
        assert_eq!(r.size(), 2)
    }
}
