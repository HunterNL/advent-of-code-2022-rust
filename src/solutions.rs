use std::{fmt::Display, fs, io::Read, str::FromStr, time};

mod day1;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day16;
mod day2;
mod day3;
mod day4;
mod day5;
mod day6;
mod day7;
mod day8;
mod day9;

#[derive(Debug, PartialEq, Eq)]
pub enum PartResult {
    Int(i32),
    Str(String),
    UInt(u64),
}

static MISSING_OUTPUT_MESSAGE: &str = "<MISSING>";

impl FromStr for PartResult {
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(value
            .parse::<i32>()
            .map_or_else(|_| Self::Str(value.to_string()), Self::Int))
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

impl TryFrom<&str> for DayOutput {
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (left, right) = value.split_once(',').ok_or("Error splitting string")?;

        Ok(Self {
            part1: Some(PartResult::Str(left.to_owned())),
            part2: Some(PartResult::Str(right.to_owned())),
        })
    }

    type Error = &'static str;
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
    print_result(run_day(6, day6::solve));
    print_result(run_day(7, day7::solve));
    print_result(run_day(8, day8::solve));
    print_result(run_day(9, day9::solve));
    print_result(run_day(10, day10::solve));
    print_result(run_day(11, day11::solve));
    print_result(run_day(12, day12::solve));
    print_result(run_day(13, day13::solve));
    print_result(run_day(14, day14::solve));
    print_result(run_day(15, day15::solve));
    print_result(run_day(16, day16::solve));
}

impl Display for PartResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Int(a) => a.to_string(),
                Self::Str(b) => b.to_string(),
                Self::UInt(c) => c.to_string(),
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
            "Day {:2}: {:5}ms [{}|{}]",
            s.day_number,
            s.duration.as_millis(),
            s.values
                .part1
                .unwrap_or_else(|| PartResult::Str(MISSING_OUTPUT_MESSAGE.to_string())),
            s.values
                .part2
                .unwrap_or_else(|| PartResult::Str(MISSING_OUTPUT_MESSAGE.to_string())),
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
                Self::Part1 => write!(f, "Part 1"),
                Self::Part2 => write!(f, "Part 2"),
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
                    format!("{part} Expected {expected} got {actual}")
                }
                TestError::NoResult => "No result".to_owned(),
                TestError::NoInputFile(s) => format!("No input file {s}"),
            }
        }
    }

    enum NoSolutionError {
        NoFile,
        ParseFailure,
    }

    fn get_solution(day_number: i32) -> Result<DayOutput, NoSolutionError> {
        let path = format!("./data/solution/day{day_number}.txt");

        let file = read_file(&path).map_err(|_| NoSolutionError::NoFile)?;

        let doe = DayOutput::try_from(file.lines().next().ok_or(NoSolutionError::ParseFailure)?)
            .map_err(|_| NoSolutionError::ParseFailure)?;

        Ok(doe)
    }

    fn compare_result(
        expected: Option<PartResult>,
        actual: Option<PartResult>,
        part: Part,
    ) -> Result<(), TestError> {
        let e = expected.ok_or(TestError::NoResult)?;
        let i = actual.ok_or(TestError::NoResult)?;

        match e.to_string() == i.to_string() {
            // Ideally we'd decode types and check those, but this works fine
            true => Ok(()),
            false => Err(TestError::Failure(part, e.to_string(), i.to_string())),
        }
    }

    pub fn test_day(day_number: i32, solution: DayFn) -> Result<(), String> {
        let input =
            get_input(day_number).map_err(|file_error| TestError::NoInputFile(file_error.path))?;
        let expected = get_solution(day_number).map_err(|_| "Error getting solution")?;
        let actual = solution(&input).map_err(|e| e.0)?;

        compare_result(expected.part1, actual.part1, Part::Part1)?;
        compare_result(expected.part2, actual.part2, Part::Part2)?;

        Ok(())
    }
}
