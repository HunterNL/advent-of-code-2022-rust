use crate::solutions::DayOutput;
use crate::solutions::PartResult;

use super::LogicError;

use std::convert::TryFrom;

struct Rucksack {
    full_string: String,
    left: String,
    right: String,
}

fn char_priority(c: char) -> Option<i32> {
    if c.is_uppercase() {
        Some(c.to_digit(36)? + 17)
    } else {
        Some(c.to_digit(36)? - 9)
    }
    .and_then(|u| u.try_into().ok())
}

impl Rucksack {
    fn priority_item_value(&self) -> Option<i32> {
        self.left
            .chars()
            .find_map(|left_char| {
                self.right
                    .chars()
                    .find_map(|right_char| (left_char == right_char).then_some(right_char))
            })
            .and_then(char_priority)
    }
}

impl TryFrom<&str> for Rucksack {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mid = value.len() / 2;
        let (left, right) = value.split_at(mid);

        Ok(Self {
            full_string: value.to_owned(),
            left: left.to_owned(),
            right: right.to_owned(),
        })
    }
}

fn find_badge(sacks: &[Rucksack]) -> char {
    let mut s: Vec<String> = vec![String::new(), String::new(), String::new()];
    let s2: Vec<String> = sacks.iter().map(|f| f.full_string.clone()).collect();

    s[0..3].clone_from_slice(&s2[0..3]);

    s.sort_by_key(String::len);

    let smallest = &s[0];

    smallest
        .chars()
        .find_map(|ch| s.iter().skip(1).all(|sack| sack.contains(ch)).then_some(ch))
        .expect("Smallest character")
}

// https://adventofcode.com/2022/day/3
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let rucksacks: Result<Vec<Rucksack>, ()> = input.lines().map(TryInto::try_into).collect();

    let rucksacks = rucksacks.map_err(|_| LogicError("Error parsing rucksacks".to_owned()))?;

    let priority_item_sum = rucksacks
        .iter()
        // .inspect(|f| println!("{:?}", f.priority_item_value()))
        .filter_map(Rucksack::priority_item_value)
        .sum();

    let badge_sum: i32 = rucksacks
        .chunks(3)
        .map(find_badge)
        .filter_map(char_priority)
        .sum();

    Ok(DayOutput {
        part1: Some(PartResult::Int(priority_item_sum)),
        part2: Some(PartResult::Int(badge_sum)),
    })
}

#[cfg(test)]
mod tests {
    use super::Rucksack;

    #[test]
    fn example() -> Result<(), ()> {
        assert_eq!(
            TryInto::<Rucksack>::try_into("vJrwpWtwJgWrhcsFMMfFFhFp")?
                .priority_item_value()
                .ok_or(())?,
            16
        );

        assert_eq!(
            TryInto::<Rucksack>::try_into("jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL")?
                .priority_item_value()
                .ok_or(())?,
            38
        );

        Ok(())
    }

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(3, super::solve)
    }
}
