use crate::solutions::DayOutput;
use crate::solutions::PartResult;

// https://adventofcode.com/2022/day/1
pub fn solve(input: &str) -> DayOutput {
    let elfs: Vec<&str> = input.split("\n\n").collect();

    let mut elf_calories = elfs
        .iter()
        .map(|line| {
            str::split(line, "\n")
                .filter_map(|line| line.parse::<i32>().ok())
                .sum::<i32>()
        })
        .collect::<Vec<i32>>();

    let max_elf_calories = *(elf_calories.iter().max().expect("Valid sum"));

    elf_calories.sort_by(|a, b| b.cmp(a)); // Sort in reverse

    let top3_elf_calories = elf_calories[0..3].iter().sum();

    DayOutput {
        part1: Some(PartResult::Int(max_elf_calories)),
        part2: Some(PartResult::Int(top3_elf_calories)),
    }
}

mod tests {

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(1, super::solve)
    }
}
