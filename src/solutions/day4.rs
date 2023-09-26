use crate::solutions::DayOutput;
use crate::solutions::PartResult;

use super::LogicError;

struct Range {
    lower: i32, //Inclusive
    upper: i32, //Exclusive
}

struct Pair {
    left: Range,
    right: Range,
}

impl TryFrom<&str> for Pair {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (left, right) = value
            .split_once(',')
            .ok_or_else(|| "Error spliting string into pair".to_owned())?;

        Ok(Self {
            left: left.try_into().map_err(|_| "Error splitting left")?,
            right: right.try_into().map_err(|_| "Error spliting right")?,
        })
    }
}

impl TryFrom<&str> for Range {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let line = value
            .split_once('-')
            .ok_or("Error spliting string into range")?;

        let left: i32 = line
            .0
            .parse()
            .map_err(|_| "Error parsing left".to_owned())?;

        let right: i32 = line
            .1
            .parse()
            .map_err(|_| "Error parsing right".to_owned())?;

        Ok(Self {
            lower: left,
            upper: right + 1,
        })
    }
}

impl Range {
    fn is_contained_in(&self, other: &Self) -> bool {
        self.lower <= other.lower && self.upper >= other.upper
    }

    fn overlaps(&self, other: &Self) -> bool {
        !(self.upper <= other.lower || self.lower >= other.upper)
    }
}

// https://adventofcode.com/2022/day/4
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let pairs: Vec<Pair> = input
        .lines()
        .map(|p| Pair::try_from(p).expect("succesful parse"))
        .collect();

    let contained_pair_count: i32 = pairs
        .iter()
        .map(|pair| {
            i32::from(
                pair.left.is_contained_in(&pair.right) || pair.right.is_contained_in(&pair.left),
            )
        })
        .sum();

    let overlapping_pair_count: i32 = pairs
        .iter()
        .map(|pair| i32::from(pair.left.overlaps(&pair.right)))
        .sum();

    // pairs.map(|)

    Ok(DayOutput {
        part1: Some(PartResult::Int(contained_pair_count)),
        part2: Some(PartResult::Int(overlapping_pair_count)),
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(4, super::solve)
    }
}
