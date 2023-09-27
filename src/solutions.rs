use std::{fmt::Display, fs, io::Read, str::FromStr, time};

mod day1;
mod day2;
mod day3;
mod day4;
mod day5;

#[derive(Debug, PartialEq, Eq)]
pub enum PartResult {
    Int(i32),
    Str(String),
}

impl FromStr for PartResult {
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.parse::<i32>() {
            Ok(i) => Self::Int(i),
            Err(_) => Self::Str(value.to_string()),
        })
    }

    type Err = ();
}

impl From<i32> for PartResult {
    fn from(val: i32) -> Self {
        Self::Int(val)
    }
}
#[derive(Debug)]
pub struct DayOutput {
    part1: Option<PartResult>,
    part2: Option<PartResult>,
}

impl From<&str> for DayOutput {
    fn from(value: &str) -> Self {
        let (left, right) = value.split_once(',').unwrap();

        Self {
            part1: left.parse().ok(),
            part2: right.parse().ok(),
        }
    }
}

pub struct SolutionOutput {
    values: DayOutput,
    duration: time::Duration,
    day_number: i32,
}

pub struct NoInputFileErr {
    path: String,
    day_number: Option<i32>,
}

impl From<NoInputFileErr> for String {
    fn from(val: NoInputFileErr) -> Self {
        val.to_string()
    }
}

impl Display for NoInputFileErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "No file found for day {}: {}",
            self.day_number.unwrap_or(0),
            self.path
        )
    }
}

pub struct LogicError(String);

enum DayError {
    NoInputFileErr(String),
    LogicError(String),
}

type DayFn = fn(&str) -> Result<DayOutput, LogicError>;

fn run_day(n: i32, solution: DayFn) -> Result<SolutionOutput, DayError> {
    let r = get_input(n).map_err(|er| DayError::NoInputFileErr(er.path))?;

    let time_start = time::Instant::now();
    let output = solution(&r);
    let duration = time_start.elapsed();

    output
        .map(|o| SolutionOutput {
            values: o,
            duration,
            day_number: n,
        })
        .map_err(|e| DayError::LogicError(e.0))
}

pub fn run() {
    print_result(run_day(1, day1::solve));
    print_result(run_day(2, day2::solve));
    print_result(run_day(3, day3::solve));
    print_result(run_day(4, day4::solve));
    print_result(run_day(5, day5::solve));
}

impl Display for PartResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Int(a) => a.to_string(),
                Self::Str(b) => b.to_string(),
            }
        )
    }
}

impl Display for DayOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p1 = self
            .part1
            .as_ref()
            .map_or_else(|| "None".to_owned(), std::string::ToString::to_string);

        let p2 = self
            .part2
            .as_ref()
            .map_or_else(|| "None".to_owned(), std::string::ToString::to_string);

        write!(f, "{p1}|{p2}")
    }
}

fn print_result(r: Result<SolutionOutput, DayError>) {
    match r {
        Ok(s) => println!(
            "Day {:2}: {:4}ms [{}|{}]",
            s.day_number,
            s.duration.as_millis(),
            s.values.part1.unwrap_or(PartResult::Int(-1)),
            s.values.part2.unwrap_or(PartResult::Int(-1))
        ),
        Err(err) => match err {
            DayError::NoInputFileErr(s) => println!("Error getting file {s}"),
            DayError::LogicError(s) => println!("Error during solve: {s}"),
        },
    }
}

fn read_file(path: &str) -> Result<String, NoInputFileErr> {
    let mut file_contents = String::new();

    fs::File::open(path)
        .map(|mut f| f.read_to_string(&mut file_contents))
        .map(|_| file_contents)
        .map_err(|_| NoInputFileErr {
            path: path.to_owned(),
            day_number: None,
        })
}

fn get_input(day_number: i32) -> Result<String, NoInputFileErr> {
    read_file(format!("./data/input/day{day_number}.txt").as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub enum Part {
        Part1,
        Part2,
    }

    impl Display for Part {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Part::Part1 => write!(f, "Part 1"),
                Part::Part2 => write!(f, "Part 2"),
            }
        }
    }

    enum TestError {
        Failure(Part, String, String),
        NoInputFile(String),
        NoResult,
    }

    impl From<TestError> for String {
        fn from(value: TestError) -> Self {
            match value {
                TestError::Failure(part, expected, actual) => {
                    format!("{} Expected {} got {}", part, expected, actual)
                }
                TestError::NoResult => "No result".to_owned(),
                TestError::NoInputFile(s) => format!("No input file {}", s),
            }
        }
    }

    fn get_solution(day_number: i32) -> Result<DayOutput, NoInputFileErr> {
        let path = format!("./data/solution/day{day_number}.txt");
        read_file(&path).map(|str| DayOutput::from(str.as_ref()))
    }

    fn compare_result(
        expected: Option<PartResult>,
        actual: Option<PartResult>,
        part: Part,
    ) -> Result<(), TestError> {
        let e = expected.ok_or(TestError::NoResult)?;
        let i = actual.ok_or(TestError::NoResult)?;

        match e == i {
            true => Ok(()),
            false => Err(TestError::Failure(part, e.to_string(), i.to_string())),
        }
    }

    pub fn test_day(day_number: i32, solution: DayFn) -> Result<(), String> {
        let input =
            get_input(day_number).map_err(|file_error| TestError::NoInputFile(file_error.path))?;
        let expected = get_solution(day_number)?;
        let actual = solution(&input).map_err(|e| e.0.to_string())?;

        compare_result(expected.part1, actual.part1, Part::Part1)?;
        compare_result(expected.part2, actual.part2, Part::Part2)?;

        Ok(())
    }
}
