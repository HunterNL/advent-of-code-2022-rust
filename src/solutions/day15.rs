use std::{collections::HashSet, str::FromStr};

use crate::vec2d::Vec2D;

use crate::{range::Ranging, rangeset::RangeSet};

use super::{DayOutput, LogicError};

const SEARCH_MAX_P2: i32 = 4_000_000;

/// Extends `char::is_ascii_digit` with `'-'` to easily select negative numbers
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

struct Line {
    /// Where the line meets the y axis (x=0)
    base: i32,

    /// Distance from the axis to the start of the line
    offset: i32,

    /// Length of the line
    length: i32,
}

enum LineDirection {
    Up,
    Down,
}

impl LineDirection {
    fn get_vec(&self) -> Vec2D<i32> {
        match self {
            LineDirection::Up => Vec2D { x: 1, y: -1 },
            LineDirection::Down => Vec2D { x: 1, y: 1 },
        }
    }
}

struct LineIterator {
    offset: i32,
    max_offset: i32,
    base: i32,
    dir: LineDirection,
}

impl Iterator for LineIterator {
    type Item = Vec2D<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.max_offset {
            return None;
        }
        self.offset += 1;

        let dir = self.dir.get_vec();
        Some(Vec2D { x: 0, y: self.base } + dir.scale(self.offset - 1))
    }
}

impl Line {
    fn iter(&self, dir: LineDirection) -> LineIterator {
        LineIterator {
            offset: self.offset,
            max_offset: self.offset + self.length,
            base: self.base,
            dir,
        }
    }

    /// Takes two lines, 2 base apart, returns the line that runs between
    fn create_valley(&self, other: &Self) -> Line {
        assert_eq!(self.base + 2, other.base);

        Line {
            base: self.base + 1,
            offset: self.offset.max(other.offset),
            length: self.length.min(other.length),
        }
    }
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

    fn lines_up(&self) -> [Line; 2] {
        let bottomright: Line = Line {
            base: self.position.y + self.radius + self.position.x,
            length: self.radius + 1,
            offset: self.position.x,
        };
        let topleft: Line = Line {
            base: self.position.y - self.radius + self.position.x,
            length: self.radius + 1,
            offset: self.position.x - self.radius,
        };

        [bottomright, topleft]
    }

    fn lines_down(&self) -> [Line; 2] {
        let topright: Line = Line {
            base: self.position.y - self.radius - self.position.x,
            length: self.radius + 1,
            offset: self.position.x,
        };
        let bottomleft: Line = Line {
            base: self.position.y + self.radius - self.position.x,
            length: self.radius + 1,
            offset: self.position.x - self.radius,
        };

        [topright, bottomleft]
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct Range {
    lower: i32,
    upper: i32,
}

impl From<(i32, i32)> for Range {
    fn from((lower, upper): (i32, i32)) -> Self {
        Range { lower, upper }
    }
}

fn line_overlap_count(sensors: &[Sensor], y: i32) -> i32 {
    // return 0;
    let mut set = RangeSet::default();
    let beacon_set: HashSet<Vec2D<i32>> = sensors.iter().map(|s| s.beacon_position).collect();
    let beacons: Vec<Vec2D<i32>> = beacon_set.into_iter().collect();

    sensors
        .iter()
        .filter_map(|s| s.range_on_y_line(y))
        .for_each(|r| {
            set.insert((r.lower, r.upper + 1));
        });

    let overlap_count: i32 = set.iter_ranges().map(|r| r.range_size()).sum();

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

fn is_outside_sensor_range(sensors: &[Sensor], position: &Vec2D<i32>) -> bool {
    sensors
        .iter()
        .all(|sensor| sensor.position.distance_manhatten(position) > sensor.radius)
}

fn find_empty_spot(sensors: &[Sensor], max: i32) -> u64 {
    let is_in_range = |vec: &Vec2D<i32>| vec.x > 0 && vec.x <= max && vec.y > 0 && vec.y <= max;

    let mut up_lines: Vec<Line> = sensors
        .iter()
        .flat_map(|s| s.lines_up().into_iter())
        .collect();
    let mut down_lines: Vec<Line> = sensors
        .iter()
        .flat_map(|s| s.lines_down().into_iter())
        .collect();

    up_lines.sort_unstable_by_key(|l| l.base);
    down_lines.sort_unstable_by_key(|l| l.base);

    let possible_up_lines: Vec<Line> = up_lines
        .iter()
        .filter_map(|line| {
            up_lines
                .iter()
                .find(|other_line| line.base + 2 == other_line.base)
                .map(|other_line| line.create_valley(other_line))
        })
        .collect();

    let possible_down_lines: Vec<Line> = down_lines
        .iter()
        .filter_map(|line| {
            down_lines
                .iter()
                .find(|other_line| line.base + 2 == other_line.base)
                .map(|other_line| line.create_valley(other_line))
        })
        .collect();

    let pos = possible_up_lines.iter().find_map(|line| {
        line.iter(LineDirection::Up)
            .filter(is_in_range)
            .find(|pos| is_outside_sensor_range(sensors, pos))
    });

    let pos = pos.or_else(|| {
        possible_down_lines.iter().find_map(|line| {
            line.iter(LineDirection::Down)
                .filter(is_in_range)
                .find(|pos| is_outside_sensor_range(sensors, pos))
        })
    });

    let pos = pos.expect("Should find something by now");

    (pos.x as u64) * 4000000 + pos.y as u64
}

// https://adventofcode.com/2022/day/15
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let sensors = make_sensors(input);

    Ok(DayOutput {
        part1: Some(super::PartResult::Int(line_overlap_count(
            &sensors, 2_000_000,
        ))),
        // part2: None,
        part2: Some(super::PartResult::UInt(find_empty_spot(
            &sensors,
            SEARCH_MAX_P2,
        ))),
    })
}

#[cfg(test)]
mod tests {

    const SEARCH_MAX_P1: i32 = 20;

    use crate::{
        solutions::day15::{find_empty_spot, line_overlap_count},
        vec2d::Vec2D,
    };

    use super::{make_sensors, Sensor};

    #[test]
    // #[ignore = "wip"]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(15, super::solve)
    }

    fn test_sensor(x: i32, y: i32, radius: i32) -> Sensor {
        Sensor {
            position: Vec2D { x, y },
            beacon_position: Vec2D { x: 0, y: 0 },
            radius,
        }
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
    fn example_p2() {
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
        assert_eq!(find_empty_spot(&sensors, SEARCH_MAX_P1), 56000011)
    }

    #[test]
    fn lines_up() {
        /*
        x------
        |
        |
        |    2
        |   212
        5| 21012
        |   212
        |    2
        |
        |
        10|
        |
        |
        |
         */
        let [bottomright, topleft] = test_sensor(5, 5, 2).lines_up();
        assert_eq!(bottomright.base, 12);
        assert_eq!(bottomright.length, 3);
        assert_eq!(bottomright.offset, 5);

        assert_eq!(topleft.base, 8);
        assert_eq!(topleft.length, 3);
        assert_eq!(topleft.offset, 3);
    }

    #[test]
    fn lines_down() {
        let [topright, bottomleft] = test_sensor(5, 5, 2).lines_down();
        assert_eq!(topright.base, -2);
        assert_eq!(topright.length, 3);
        assert_eq!(topright.offset, 5);

        assert_eq!(bottomleft.base, 2);
        assert_eq!(bottomleft.length, 3);
        assert_eq!(bottomleft.offset, 3);
    }

    #[test]
    fn line_iter() {
        let [topright, _] = test_sensor(5, 5, 2).lines_down();

        let mut iter = topright.iter(super::LineDirection::Down);
        assert_eq!(iter.next(), Some(Vec2D { x: 5, y: 3 }));
        assert_eq!(iter.next(), Some(Vec2D { x: 6, y: 4 }));
        assert_eq!(iter.next(), Some(Vec2D { x: 7, y: 5 }));
        assert_eq!(iter.next(), None);
    }
}

//     #[test]
//     fn radius() {
//         let sensor: Sensor = "Sensor at x=0, y=0: closest beacon is at x=1, y=0"
//             .parse()
//             .unwrap();

//         assert_eq!(sensor.radius, 1)
//     }

//     #[test]
//     fn size_on_line() {
//         {
//             let sensor: Sensor = "Sensor at x=5, y=0: closest beacon is at x=7, y=0"
//                 .parse()
//                 .unwrap();

//             let range = sensor.range_on_y_line(1);

//             let range = range.unwrap();

//             assert_eq!(range.lower, 4);
//             assert_eq!(range.upper, 6);
//             assert_eq!(range.size(), 3)
//         }
//         {
//             let sensor: Sensor = "Sensor at x=5, y=0: closest beacon is at x=7, y=0"
//                 .parse()
//                 .unwrap();

//             let range = sensor.range_on_y_line(0);

//             let range = range.unwrap();

//             assert_eq!(range.lower, 3);
//             assert_eq!(range.upper, 7);
//             assert_eq!(range.size(), 5);
//         }
//         {
//             let sensor: Sensor = "Sensor at x=5, y=0: closest beacon is at x=7, y=0"
//                 .parse()
//                 .unwrap();

//             let range = sensor.range_on_y_line(2);

//             let range = range.unwrap();

//             assert_eq!(range.lower, 5);
//             assert_eq!(range.upper, 5);
//             assert_eq!(range.size(), 1);
//         }
//         {
//             let sensor: Sensor = "Sensor at x=5, y=0: closest beacon is at x=7, y=0"
//                 .parse()
//                 .unwrap();

//             let range = sensor.range_on_y_line(3);

//             println!("{:?}", range);

//             assert!(range.is_none())
//         }
//     }

//     #[test]
//     fn range_set_merge() {
//         let mut set = RangeSet { ranges: vec![] };

//         set.insert(super::Range { lower: 0, upper: 1 });

//         set.insert(super::Range { lower: 1, upper: 2 });

//         assert_eq!(set.ranges.len(), 1);

//         let entry = set.ranges.remove(0);

//         assert_eq!(entry.lower, 0);
//         assert_eq!(entry.upper, 2);
//     }

//     #[test]
//     fn range_set_touching_merge() {
//         let mut set = RangeSet { ranges: vec![] };

//         set.insert(super::Range { lower: 0, upper: 1 });

//         set.insert(super::Range { lower: 2, upper: 3 });

//         assert_eq!(set.ranges.len(), 1);

//         let entry = set.ranges.remove(0);

//         assert_eq!(entry.lower, 0);
//         assert_eq!(entry.upper, 3);
//     }

//     #[test]
//     fn range_remove() {
//         let big_range: Range = (0, 10).into();
//         let small_range: Range = (4, 6).into();

//         let slices = big_range.remove(&small_range);

//         let left_cut: Range = (0, 3).into();
//         let right_cut: Range = (7, 10).into();

//         assert_eq!(*slices.get(0).unwrap(), left_cut);
//         assert_eq!(*slices.get(1).unwrap(), right_cut);
//     }

//     #[test]
//     fn size() {
//         let r = Range { lower: 2, upper: 3 };
//         assert_eq!(r.size(), 2)
//     }
// }
