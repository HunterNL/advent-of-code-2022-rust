use std::collections::VecDeque;
use std::str::FromStr;

use super::{DayOutput, LogicError, PartResult};

enum Operator {
    Add,
    Multiply,
}

enum Operand {
    Literal(u32),
    Old,
}

impl FromStr for Operand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "old" => Operand::Old,
            _ => Operand::Literal(
                s.parse()
                    .map_err(|_| format!("Error parsing literal {}", s).to_owned())?,
            ),
        })
    }
}

impl FromStr for Operator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Operator::Add),
            "*" => Ok(Operator::Multiply),
            &_ => Err("Unknown string".to_owned()),
        }
    }
}

struct Monkey {
    items: VecDeque<u32>,
    behaviour: MonkeyBehaviour,
    items_processed: u32,
}

struct ItemThrow {
    items: Vec<u32>,
    target: u32,
}

impl Monkey {
    fn new(behaviour: MonkeyBehaviour) -> Monkey {
        Monkey {
            items: VecDeque::from(behaviour.starting_items.clone()),
            behaviour,
            items_processed: 0,
        }
    }

    fn take_turn(&mut self, false_throw: &mut ItemThrow, true_throw: &mut ItemThrow) {
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
                true_throw.items.push(item)
            } else {
                false_throw.items.push(item)
            }

            self.items_processed += 1
        }
    }

    fn worry_level_operation(&self, level: u32) -> u32 {
        let operand = match self.behaviour.operation_operand {
            Operand::Literal(n) => n,
            Operand::Old => level,
        };

        // println!("{} {}", level, operand);

        match self.behaviour.operation_operator {
            Operator::Add => level + operand,
            Operator::Multiply => level * operand,
        }
    }

    fn receive_items(&mut self, throw: &mut ItemThrow) {
        throw
            .items
            .iter()
            .for_each(|item| self.items.push_back(*item))
    }
}

/// Stateless monkey settings
struct MonkeyBehaviour {
    monkey_id: u32,
    starting_items: Vec<u32>,
    operation_operator: Operator,
    operation_operand: Operand,
    test_div: u32,
    true_target: u32,
    false_target: u32,
}

fn get_num_from_char_iter(iter: impl Iterator<Item = char>) -> u32 {
    let a: String = iter
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();

    a.parse().unwrap()
}

struct MonkeyGame {
    monkeys: Vec<Monkey>,
    true_trow: ItemThrow,
    false_throw: ItemThrow,
}

impl MonkeyGame {
    fn new(monkeys: Vec<Monkey>) -> MonkeyGame {
        MonkeyGame {
            monkeys,
            true_trow: ItemThrow {
                items: Vec::new(),
                target: 0,
            },
            false_throw: ItemThrow {
                items: Vec::new(),
                target: 0,
            },
        }
    }

    fn run_round(&mut self) {
        for i in 0..self.monkeys.len() {
            self.monkeys
                .get_mut(i)
                .unwrap()
                .take_turn(&mut self.false_throw, &mut self.true_trow);
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

        // for monkey in self.monkeys.iter_mut() {}
    }

    fn monkey_business(&self) -> u32 {
        let mut v: Vec<u32> = self.monkeys.iter().map(|m| m.items_processed).collect();
        v.sort();
        v.pop().unwrap() * v.pop().unwrap()
    }
}

impl FromStr for MonkeyBehaviour {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut line_iter = s.lines();

        let id_line = line_iter.next().unwrap();
        let id: u32 = id_line
            .chars()
            .nth(7)
            .unwrap()
            .to_digit(10)
            .expect("digit < 10");

        let starting_line = line_iter.next().unwrap();
        let starting_items_comma_seperated: String = starting_line.chars().skip(18).collect();
        let starting_items: Vec<u32> = starting_items_comma_seperated
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

        Ok(MonkeyBehaviour {
            monkey_id: id,
            starting_items,
            operation_operator: operator,
            operation_operand: operand,
            test_div: divider,
            true_target,
            false_target,
        })

        // lines
    }
}

// https://adventofcode.com/2022/day/11
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let monkeys: Vec<Monkey> = input
        .split("\n\n")
        .map(|str| str.parse::<MonkeyBehaviour>().unwrap())
        .map(Monkey::new)
        .collect();
    let mut game = MonkeyGame::new(monkeys);

    let a = game.monkeys.get(0).unwrap().behaviour.test_div;
    let a = game
        .monkeys
        .iter()
        .skip(1)
        .fold(a, |a, b| a * b.behaviour.test_div);

    for _ in 0..20 {
        game.run_round();
    }

    Ok(DayOutput {
        part1: Some(PartResult::Int(game.monkey_business() as i32)),
        part2: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(11, super::solve)
    }
}
