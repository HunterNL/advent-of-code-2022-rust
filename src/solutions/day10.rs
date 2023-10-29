use std::str::FromStr;

use super::{DayOutput, LogicError, PartResult};

enum Instruction {
    Noop,
    Addx(i32),
}

impl FromStr for Instruction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "noop" {
            return Ok(Instruction::Noop);
        }
        let (_, num) = s.split_once(' ').ok_or("Couldn't split")?;

        Ok(Instruction::Addx(num.parse().unwrap()))
    }
}
struct Cpu {
    register: i32,
    program: Vec<Instruction>,
    program_counter: usize,
    cycle_delay: usize,
    cycle_count: usize,
}

impl Cpu {
    fn new_with_program(program: impl Iterator<Item = Instruction>) -> Self {
        Cpu {
            cycle_count: 0,
            cycle_delay: 0,
            register: 1,
            program: program.collect(),
            program_counter: 0,
        }
    }

    fn cycle_times(&mut self, n: usize) {
        for _i in 0..n {
            self.cycle()
        }
    }
    fn signal_strenght(&self) -> i32 {
        (self.cycle_count + 1) as i32 * self.register
    }

    fn run_to_count(&mut self, count: usize) {
        while self.cycle_count < count {
            self.cycle()
        }
    }

    fn cycle(&mut self) {
        self.cycle_count += 1;

        let current_instruction = self
            .program
            .get(self.program_counter)
            .expect("program counter not to overflow");

        match current_instruction {
            Instruction::Noop => self.program_counter += 1,
            Instruction::Addx(n) => {
                if self.cycle_delay == 0 {
                    self.cycle_delay = 1;
                } else {
                    self.cycle_delay = 0;
                    self.program_counter += 1;
                    self.register += n;
                }
            }
        }
    }
}

// https://adventofcode.com/2022/day/10
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let mut cpu = Cpu::new_with_program(
        input
            .lines()
            .map(|line| line.parse::<Instruction>().unwrap()),
    );

    let mut signal_sum = 0;
    cpu.run_to_count(19); // 20
    signal_sum += cpu.signal_strenght();

    cpu.cycle_times(40); // 60
    signal_sum += cpu.signal_strenght();

    cpu.cycle_times(40); // 100
    signal_sum += cpu.signal_strenght();

    cpu.cycle_times(40); //140
    signal_sum += cpu.signal_strenght();

    cpu.cycle_times(40); // 180
    signal_sum += cpu.signal_strenght();

    cpu.cycle_times(40); // 220
    signal_sum += cpu.signal_strenght();

    Ok(DayOutput {
        part1: Some(PartResult::Int(signal_sum)),
        part2: None,
    })
}

#[cfg(test)]
mod tests {
    use super::Cpu;

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(10, super::solve)
    }

    #[test]
    fn example_1() -> Result<(), String> {
        let input = ["noop", "addx 3", "addx -5"];

        let mut cpu = Cpu::new_with_program(input.iter().map(|line| line.parse().unwrap()));

        cpu.cycle(); //1st
        assert_eq!(cpu.register, 1);

        cpu.cycle(); //2nd
        assert_eq!(cpu.register, 1);

        cpu.cycle(); //3rd
        assert_eq!(cpu.register, 4);

        cpu.cycle(); //4rd
        assert_eq!(cpu.register, 4);

        cpu.cycle(); //5th
        assert_eq!(cpu.register, -1);

        Ok(())
    }

    #[test]
    fn example_2() -> Result<(), String> {
        let input: String = "addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop"
            .to_owned();

        let mut cpu = Cpu::new_with_program(input.lines().map(|line| line.parse().unwrap()));

        cpu.cycle_times(19);
        assert_eq!(cpu.register, 21, "Stop 1: CPU register != 21");
        assert_eq!(cpu.signal_strenght(), 420, "Stop 1: Signal strenght != 420");
        cpu.cycle();

        cpu.cycle_times(39);
        assert_eq!(cpu.register, 19, "Stop 2: CPU register != 19");
        assert_eq!(
            cpu.signal_strenght(),
            1140,
            "Stop 2: Signal strenght != 1140"
        );
        cpu.cycle();

        Ok(())
    }
}
