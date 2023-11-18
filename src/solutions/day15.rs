use std::{
    char,
    collections::HashSet,
    fmt::{self, Display, Write},
    str::FromStr,
};

use crate::vec2d::{Vec2D, DOWN, LEFT, RIGHT, UP};

use crate::{
    range::{self, Ranging},
    rangeset::RangeIterator,
    rangeset::RangeSet,
};

use super::{DayOutput, LogicError};

const SEARCH_MAX_P1: i32 = 20;
const SEARCH_MAX_P2: i32 = 4000000;

/// Extends char::is_ascii_digit with '-' char to easily select negative numbers
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

enum Side {
    TopRight,
    BottomRight,
    BottomLeft,
    TopLeft,
}

struct ManhattenCircleRadiusIterator {
    radius: i32,
    side: Side,
    side_index: i32,
    current_position: Vec2D<i32>,
    current_direction: Vec2D<i32>,
    index: i32,
}

impl ManhattenCircleRadiusIterator {
    fn new(center: Vec2D<i32>, radius: i32) -> Self {
        ManhattenCircleRadiusIterator {
            radius,
            side: Side::TopRight,
            side_index: 0,
            index: 0,
            current_position: center + crate::vec2d::UP.scale(radius),
            current_direction: DOWN + RIGHT,
        }
    }
}

impl Iterator for ManhattenCircleRadiusIterator {
    type Item = Vec2D<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        let out = self.current_position;
        self.side_index += 1;
        self.index += 1;
        if self.index > self.radius * 4 {
            return None;
        }
        if self.side_index > self.radius {
            self.side_index = 1;
            self.side = match self.side {
                Side::TopRight => Side::BottomRight,
                Side::BottomRight => Side::BottomLeft,
                Side::BottomLeft => Side::TopLeft,
                Side::TopLeft => Side::TopRight,
            };

            self.current_direction = match self.side {
                Side::TopRight => DOWN + RIGHT,
                Side::BottomRight => DOWN + LEFT,
                Side::BottomLeft => UP + LEFT,
                Side::TopLeft => UP + RIGHT,
            };
        }

        self.current_position = self.current_position + self.current_direction;

        Some(out)
    }
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

    fn area(&self) -> i32 {
        let rplus = self.radius + 1;
        (self.radius * self.radius) + (rplus * rplus)
    }

    fn common_area(&self, other: &Self) -> i32 {
        let distance = self.position.distance_manhatten(&other.position) - 1;
        let overlap_depth = -(distance - (self.radius + other.radius));
        if overlap_depth <= 0 {
            return 0;
        }

        let x_dif = (self.position.x - other.position.x).abs() - 1;
        let y_dif = (self.position.y - other.position.y).abs() - 1;

        let diff = x_dif.min(y_dif) + 1;

        let smaller_radius = self.radius.min(other.radius);
        let smaller_cross_section = (smaller_radius - 1) * 2 + 1;
        smaller_cross_section * overlap_depth
    }

    fn cross_section(&self) -> i32 {
        (self.radius - 1) * 2 + 1
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

// #[derive(Debug)]
// struct RangeSet {
//     ranges: Vec<Range>,
// }

// impl Display for RangeSet {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let max = self.ranges.iter().map(|r| r.upper).max().unwrap();

//         for n in 0..=max {
//             if self.is_in_range(n) {
//                 f.write_char('X')?
//             } else {
//                 f.write_char(' ')?
//             }
//         }

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod rangeset_tests {
//     use super::RangeSet2;

//     #[test]
//     fn is_in_range() {
//         let rs = RangeSet2(vec![0, 10]);
//         assert!(rs.is_in_range(0));
//         assert!(rs.is_in_range(10));
//         assert!(rs.is_in_range(5));

//         assert!(!rs.is_in_range(-1));
//         assert!(!rs.is_in_range(-5));
//         assert!(!rs.is_in_range(11));
//     }

//     // #[test]
//     // fn insert_extend() {
//     //     let mut rs = RangeSet2(vec![0, 5, 10, 15]);
//     //     rs.insert(&(15, 20).into());
//     //     assert_eq!(rs.0, vec![0, 5, 10, 20])
//     // }
// }

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
        // .inspect(|f| )
        .for_each(|r| {
            // println!("Adding {} {} to line ", r.lower, r.upper + 1);
            set.insert((r.lower, r.upper + 1));
            // println!("Sum: {}", set.size())
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

fn find_empty_spot(sensors: &[Sensor], max: i32) -> u64 {
    // let mut ranges: Vec<RangeSet> = vec![];
    // ranges.reserve(max as usize);

    // ranges.fill(Default::default());
    // for _ in 0..max {
    //     ranges.push(RangeSet::new_with_capacity(8));
    // }

    let is_in_range = |vec: &Vec2D<i32>| vec.x > 0 && vec.x <= max && vec.y > 0 && vec.y <= max;

    // for sensor in s {

    //Find a sensor...
    let pos = sensors.iter().find_map(|sensor| {
        // ... where a point just outside its radius
        ManhattenCircleRadiusIterator::new(sensor.position, sensor.radius + 1)
            .filter(is_in_range)
            .find(|test_position| {
                // Is outside the radius of all sensors

                sensors
                    .iter()
                    .all(|sensor| sensor.position.distance_manhatten(test_position) > sensor.radius)
            })
    });

    //     let upper_y = sensor.position.y - sensor.radius;
    //     let lower_y = sensor.position.y + sensor.radius;

    //     for y in upper_y.max(0)..lower_y.min(max) {
    //         let sensor_line_range = sensor.range_on_y_line(y).expect(
    //             "Sensor should have some range on this line, we're already limiting to radius",
    //         );

    //         let rs = ranges.get_mut(y as usize).unwrap();

    //         // let rs = ranges.get_mut(y as usize).unwrap();
    //         // let size_before = rs.size();
    //         // if sensor_line_range.lower == 6 && sensor_line_range.upper == 10 {
    //         // println!("Oh oh");
    //         // }

    //         // println!("====================");
    //         // println!("Before: {:?}", rs);
    //         // println!(
    //         //     "Removing {} {}",
    //         //     sensor_line_range.lower,
    //         //     sensor_line_range.upper + 1,
    //         // );
    //         rs.insert((
    //             sensor_line_range.lower.max(0),
    //             sensor_line_range.upper.min(max) + 1,
    //         ));
    //         // rs.remove((sensor_line_range.lower, sensor_line_range.upper + 1));
    //         // println!("After: {:?}", rs);

    //         // let size_after = rs.size();
    //         // if size_after > size_before {
    //         //     println!("Size grew!");
    //         //     panic!();
    //         // }

    //         // ranges
    //         //     .get_mut(y as usize)
    //         //     .unwrap()
    //         //     .remove((sensor_line_range.lower, sensor_line_range.upper + 1))
    //     }
    // }
    if pos.is_none() {
        panic!("oh no");
    }
    let pos = pos.unwrap();

    // let y = ranges.iter().position(|r| r.len() == 2).expect("A find");

    // let line = ranges.get(y).expect("One find");
    // let range = line.iter_ranges().next().expect("An entry");

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

    use crate::{
        solutions::day15::{
            find_empty_spot, line_overlap_count, Range, SEARCH_MAX_P1, SEARCH_MAX_P2,
        },
        vec2d::{Vec2D, DOWN, LEFT, RIGHT, UP},
    };

    use super::{make_sensors, ManhattenCircleRadiusIterator, RangeSet, Sensor};

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
    fn area() {
        let mut s = Sensor {
            beacon_position: Vec2D { x: 0, y: 0 },
            position: Vec2D { x: 0, y: 0 },
            radius: 0,
        };

        s.radius = 1;
        assert_eq!(s.area(), 5);

        s.radius = 2;
        assert_eq!(s.area(), 13);

        s.radius = 3;
        assert_eq!(s.area(), 25);

        s.radius = 4;
        assert_eq!(s.area(), 41);
    }

    #[test]
    fn cross_section() {
        assert_eq!(test_sensor(0, 0, 1).cross_section(), 1);
        assert_eq!(test_sensor(0, 0, 2).cross_section(), 3);
        assert_eq!(test_sensor(0, 0, 3).cross_section(), 5);
        assert_eq!(test_sensor(0, 0, 4).cross_section(), 7);
    }

    #[test]
    fn common_area_none() {
        let left = test_sensor(5, 0, 1);
        let right = test_sensor(8, 0, 1);

        assert_eq!(left.common_area(&right), 0)
    }

    #[test]
    fn common_area_in_axis() {
        let left = test_sensor(5, 0, 1);
        let right = test_sensor(7, 0, 1);

        assert_eq!(left.common_area(&right), 1)
    }

    fn v(x: i32, y: i32) -> Option<Vec2D<i32>> {
        Some(Vec2D { x, y })
    }

    #[test]
    fn circle_iter() {
        let mut iter_r1 = ManhattenCircleRadiusIterator::new(Vec2D { x: 0, y: 0 }, 1);

        assert_eq!(iter_r1.next(), Some(UP));
        assert_eq!(iter_r1.next(), Some(RIGHT));
        assert_eq!(iter_r1.next(), Some(DOWN));
        assert_eq!(iter_r1.next(), Some(LEFT));
        assert_eq!(iter_r1.next(), None);

        let mut iter_r2 = ManhattenCircleRadiusIterator::new(Vec2D { x: 0, y: 0 }, 2);

        assert_eq!(iter_r2.next(), v(0, -2));
        assert_eq!(iter_r2.next(), v(1, -1));
        assert_eq!(iter_r2.next(), v(2, 0));
        assert_eq!(iter_r2.next(), v(1, 1));
        assert_eq!(iter_r2.next(), v(0, 2));
        assert_eq!(iter_r2.next(), v(-1, 1));
        assert_eq!(iter_r2.next(), v(-2, 0));
        assert_eq!(iter_r2.next(), v(-1, -1));
        assert_eq!(iter_r2.next(), None);
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
