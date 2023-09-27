use super::{DayOutput, LogicError};

// https://adventofcode.com/2022/day/6
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    Ok(DayOutput {
        part1: None,
        part2: None,
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(6, super::solve)
    }
}
