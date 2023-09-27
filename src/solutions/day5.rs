use std::fmt::Display;
use std::ops::Index;
use std::ops::IndexMut;
use std::str::Chars;
use std::str::FromStr;

use crate::solutions::DayOutput;
use crate::solutions::PartResult;

use super::LogicError;

// "move 2 from 4 to 2"
#[derive(Debug)]
struct Command {
    count: i32,
    origin: i32,
    destination: i32,
}

impl FromStr for Command {
    type Err = ();

    // "move 2 from 4 to 2"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut i = s.chars();
        let count = scan_i32_from_char_mut(&mut i);
        let origin = scan_i32_from_char_mut(&mut i) - 1;
        let destination = scan_i32_from_char_mut(&mut i) - 1;

        Ok(Command {
            count,
            origin,
            destination,
        })
    }
}

fn scan_i32_from_char_mut(i: &mut Chars<'_>) -> i32 {
    let digit_as_string: String = i
        .by_ref() // Mutate the original iterator
        .skip_while(|c| !c.is_ascii_digit()) // Skip every non-digit
        .take_while(|c| c.is_ascii_digit()) // Take all the consecutive digits
        .collect();
    digit_as_string.parse().unwrap()
}

#[derive(Debug, Clone)]
struct Stacks(Vec<Vec<u8>>);

impl Display for Stacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(self.0.iter().for_each(|v| {
            let s: String = v.iter().map(|c| c.clone() as char).collect();
            write!(f, "{s}\n");
        }))
    }
}

impl Stacks {
    fn print_top_stack(&self) -> String {
        self.0
            .iter()
            .map(|v| v.last().unwrap().clone() as char)
            .collect()
    }
}

impl FromStr for Stacks {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let first_line = s.lines().next().expect("First line to exists");
        let stack_count: i32 = ((first_line.len() + 1) / 4).try_into().unwrap(); // Each line has 4 characters (3+1padding), last column lacks the final padding so we add that to cleanly devide

        let mut columns: Vec<Vec<u8>> = Vec::with_capacity(stack_count as usize);
        for _ in 0..stack_count {
            columns.push(vec![]);
        }

        // For each line in revserse
        // Skipping the line with only numbers
        s.lines().rev().skip(1).for_each(|line| {
            // For every column left to right
            for n in 0..stack_count {
                let character = line.as_bytes().get((n as usize) * 4 + 1).unwrap();
                if character.is_ascii_alphabetic() {
                    let v = columns.get_mut(n as usize).unwrap();
                    v.push(character.to_owned())
                }
            }
        });

        Ok(Stacks(columns))
    }
}

// https://adventofcode.com/2022/day/5
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let (stackStr, commandStr) = input.split_once("\n\n").expect("input to contain newlines");

    let commands: Vec<Command> = commandStr
        .lines()
        .map(|l| l.parse::<Command>())
        .map(|o| o.expect("valid command"))
        .collect();

    let mut part1_stack: Stacks = stackStr.parse().expect("succesful parse");
    let mut part2_stack: Stacks = part1_stack.clone();

    execute_p1_crane_commands(&mut part1_stack, &commands);
    let p1 = part1_stack.print_top_stack();

    execute_p2_crane_commands(&mut part2_stack, &commands);
    let p2 = part2_stack.print_top_stack();

    Ok(DayOutput {
        part1: Some(PartResult::Str(p1)),
        part2: Some(PartResult::Str(p2)),
    })
}

fn execute_p1_crane_commands(s: &mut Stacks, commands: &Vec<Command>) {
    commands.iter().for_each(|command| {
        for _ in 0..command.count {
            let container =
                s.0.index_mut(command.origin as usize)
                    .pop()
                    .expect("Stack not to empty");

            s.0.index_mut(command.destination as usize).push(container);
        }
    });
}

fn execute_p2_crane_commands(s: &mut Stacks, commands: &Vec<Command>) {
    commands.iter().for_each(|command| {
        let mut arm_stack = vec![];
        for _ in 0..command.count {
            arm_stack.push(
                s.0.index_mut(command.origin as usize)
                    .pop()
                    .expect("Stack not to empty"),
            );
        }

        for _ in 0..command.count {
            let c = arm_stack.pop().unwrap();
            s.0.index_mut(command.destination as usize).push(c);
        }
    });
}

#[cfg(test)]
mod tests {

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(5, super::solve)
    }
}
