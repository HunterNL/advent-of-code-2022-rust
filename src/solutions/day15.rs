use std::{collections::HashSet, str::FromStr};

use crate::parsing::consume_number_from_char_iter;
use crate::vec2d::Vec2D;

use crate::{range::Ranging, rangeset::RangeSet};

use super::{DayOutput, LogicError};

const SEARCH_MAX_P2: i32 = 4_000_000;

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

impl Line {
    /// Takes two lines, 2 base apart, returns the line that runs between
    fn create_valley(&self, other: &Self) -> Self {
        assert_eq!(self.base + 2, other.base);

        Self {
            base: self.base + 1,
            offset: self.offset.max(other.offset),
            length: self.length.min(other.length),
        }
    }

    fn intersection_point(&self, other: &Self) -> Vec2D<i32> {
        let x = (-other.base + self.base) / 2;
        let y = (self.base + other.base) / 2;
        Vec2D { x, y }
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

        Ok(Self {
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
        Self { lower, upper }
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

    let up_line_valleys: Vec<Line> = up_lines
        .iter()
        .filter_map(|line| {
            up_lines
                .iter()
                .find(|other_line| line.base + 2 == other_line.base)
                .map(|other_line| line.create_valley(other_line))
        })
        .collect();

    let down_line_valleys: Vec<Line> = down_lines
        .iter()
        .filter_map(|line| {
            down_lines
                .iter()
                .find(|other_line| line.base + 2 == other_line.base)
                .map(|other_line| line.create_valley(other_line))
        })
        .collect();

    // Iterate over every combination of valley lines
    let intersection = up_line_valleys
        .iter()
        .find_map(|up_line| {
            down_line_valleys.iter().find_map(|down_line| {
                let position = up_line.intersection_point(down_line);
                if is_in_range(&position) && is_outside_sensor_range(sensors, &position) {
                    Some(position)
                } else {
                    None
                }
            })
        })
        .expect("Intersection should be found");

    assert!(is_in_range(&intersection));
    assert!(is_outside_sensor_range(sensors, &intersection));

    (intersection.x as u64) * 4_000_000 + intersection.y as u64
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

        assert_eq!(line_overlap_count(&sensors, 10), 26);
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
        assert_eq!(find_empty_spot(&sensors, SEARCH_MAX_P1), 56_000_011);
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
}
