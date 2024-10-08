use std::collections::VecDeque;
use std::str::FromStr;

use super::{DayOutput, LogicError, PartResult};

#[derive(Clone)]
enum Operator {
    Add,
    Multiply,
}

#[derive(Clone)]
enum Operand {
    Literal(u64),
    Old,
}

impl FromStr for Operand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "old" => Self::Old,
            _ => Self::Literal(
                s.parse()
                    .map_err(|_| format!("Error parsing literal {s}"))?,
            ),
        })
    }
}

impl FromStr for Operator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Self::Add),
            "*" => Ok(Self::Multiply),
            &_ => Err("Unknown string".to_owned()),
        }
    }
}

struct Monkey {
    items: VecDeque<u64>,
    behaviour: MonkeyBehaviour,
    items_processed: u32,
}

struct ItemThrow {
    items: Vec<u64>,
    target: u32,
}

impl Monkey {
    fn new(behaviour: MonkeyBehaviour) -> Self {
        Self {
            items: VecDeque::from(behaviour.starting_items.clone()),
            behaviour,
            items_processed: 0,
        }
    }

    fn take_turn_p1(&mut self, false_throw: &mut ItemThrow, true_throw: &mut ItemThrow) {
        false_throw.target = self.behaviour.false_target;
        true_throw.target = self.behaviour.true_target;

        while !self.items.is_empty() {
            let item = self
                .items
                .pop_front()
                .expect("Queue to stop before it empties");

            let item = self.worry_level_operation(item);

            let item = item / 3;

            let is_divisable = (item % self.behaviour.test_div) == 0;

            if is_divisable {
                true_throw.items.push(item);
            } else {
                false_throw.items.push(item);
            }

            self.items_processed += 1;
        }
    }

    fn take_turn_p2(&mut self, false_throw: &mut ItemThrow, true_throw: &mut ItemThrow, c: u64) {
        false_throw.target = self.behaviour.false_target;
        true_throw.target = self.behaviour.true_target;

        while !self.items.is_empty() {
            let item = self
                .items
                .pop_front()
                .expect("Queue to stop before it empties");

            let item = self.worry_level_operation(item);

            let item = item % c;

            let is_divisable = (item % self.behaviour.test_div) == 0;

            if is_divisable {
                true_throw.items.push(item);
            } else {
                false_throw.items.push(item);
            }

            self.items_processed += 1;
        }
    }
    fn worry_level_operation(&self, level: u64) -> u64 {
        let operand = match self.behaviour.operation_operand {
            Operand::Literal(n) => n,
            Operand::Old => level,
        };

        match self.behaviour.operation_operator {
            Operator::Add => level + operand,
            Operator::Multiply => level * operand,
        }
    }

    fn receive_items(&mut self, throw: &mut ItemThrow) {
        throw
            .items
            .iter()
            .for_each(|item| self.items.push_back(*item));
    }
}

/// Stateless monkey settings
#[derive(Clone)]
struct MonkeyBehaviour {
    starting_items: Vec<u64>,
    operation_operator: Operator,
    operation_operand: Operand,
    test_div: u64,
    true_target: u32,
    false_target: u32,
}

fn get_num_from_char_iter(iter: impl Iterator<Item = char>) -> u32 {
    let a: String = iter
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();

    a.parse().unwrap()
}

struct MonkeyGame {
    monkeys: Vec<Monkey>,
    true_trow: ItemThrow,
    false_throw: ItemThrow,
    g: u64,
}

fn gcd(iter: impl Iterator<Item = u64>) -> u64 {
    iter.reduce(|a, b| a * b).unwrap()
}

impl MonkeyGame {
    fn new(monkeys: Vec<Monkey>) -> Self {
        let g = gcd(monkeys.iter().map(|m| m.behaviour.test_div));

        Self {
            true_trow: ItemThrow {
                items: Vec::new(),
                target: 0,
            },
            false_throw: ItemThrow {
                items: Vec::new(),
                target: 0,
            },
            monkeys,
            g,
        }
    }

    fn run_round(&mut self, part: Part) {
        match part {
            Part::Part1 => {
                for i in 0..self.monkeys.len() {
                    self.monkeys
                        .get_mut(i)
                        .unwrap()
                        .take_turn_p1(&mut self.false_throw, &mut self.true_trow);
                    {
                        let true_monkey = self
                            .monkeys
                            .get_mut(self.true_trow.target as usize)
                            .unwrap();

                        true_monkey.receive_items(&mut self.true_trow);
                        self.true_trow.items.clear();
                    }
                    {
                        let false_monkey = self
                            .monkeys
                            .get_mut(self.false_throw.target as usize)
                            .unwrap();

                        false_monkey.receive_items(&mut self.false_throw);
                        self.false_throw.items.clear();
                    }
                }
            }
            Part::Part2 => {
                for i in 0..self.monkeys.len() {
                    self.monkeys.get_mut(i).unwrap().take_turn_p2(
                        &mut self.false_throw,
                        &mut self.true_trow,
                        self.g,
                    );
                    {
                        let true_monkey = self
                            .monkeys
                            .get_mut(self.true_trow.target as usize)
                            .unwrap();

                        true_monkey.receive_items(&mut self.true_trow);
                        self.true_trow.items.clear();
                    }
                    {
                        let false_monkey = self
                            .monkeys
                            .get_mut(self.false_throw.target as usize)
                            .unwrap();

                        false_monkey.receive_items(&mut self.false_throw);
                        self.false_throw.items.clear();
                    }
                }
            }
        }

        // for monkey in self.monkeys.iter_mut() {}
    }

    fn monkey_business(&self) -> u64 {
        let mut v: Vec<u32> = self.monkeys.iter().map(|m| m.items_processed).collect();

        v.sort_unstable();

        let i1: u64 = u64::from(v.pop().unwrap());
        let i2: u64 = u64::from(v.pop().unwrap());

        i1 * i2
    }
}

impl FromStr for MonkeyBehaviour {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut line_iter = s.lines();

        line_iter.next(); // Skip the monkey_id line

        let starting_line = line_iter.next().unwrap();
        let starting_items_comma_seperated: String = starting_line.chars().skip(18).collect();
        let starting_items: Vec<_> = starting_items_comma_seperated
            .split(',')
            .map(|s| s.trim().parse().unwrap())
            .collect();

        let operation_line_iter = line_iter.next().unwrap().chars();
        let mut operation_line_iter2 = operation_line_iter.skip(23);
        let operator: Operator = operation_line_iter2
            .next()
            .unwrap()
            .to_string()
            .parse()
            .unwrap();

        let i3 = operation_line_iter2.skip(1);
        let operand: Operand = i3.collect::<String>().parse().unwrap();

        let divider = get_num_from_char_iter(line_iter.next().unwrap().chars());
        let true_target = get_num_from_char_iter(line_iter.next().unwrap().chars());
        let false_target = get_num_from_char_iter(line_iter.next().unwrap().chars());

        Ok(Self {
            starting_items,
            operation_operator: operator,
            operation_operand: operand,
            test_div: u64::from(divider),
            true_target,
            false_target,
        })

        // lines
    }
}

enum Part {
    Part1,
    Part2,
}

// https://adventofcode.com/2022/day/11
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let behaviours: Vec<_> = input
        .split("\n\n")
        .map(|str| str.parse::<MonkeyBehaviour>().unwrap())
        .collect();

    let mut p1_game = MonkeyGame::new(behaviours.clone().into_iter().map(Monkey::new).collect());
    let mut p2_game = MonkeyGame::new(behaviours.into_iter().map(Monkey::new).collect());

    for _ in 0..20 {
        p1_game.run_round(Part::Part1);
    }
    for _ in 0..10_000 {
        p2_game.run_round(Part::Part2);
    }

    Ok(DayOutput {
        part1: Some(PartResult::UInt(p1_game.monkey_business())),
        part2: Some(PartResult::UInt(p2_game.monkey_business())),
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(11, super::solve)
    }
}
