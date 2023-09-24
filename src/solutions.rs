use std::{error::Error, fmt::Display, fs, io::Read, time};

mod day1;
mod day2;

#[derive(Debug, PartialEq, Eq)]
pub enum PartResult {
    Int(i32),
}

// impl Display for PartResult {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "{}", self)
//     }
// }

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
        let parsed_values: Vec<i32> = value
            .split(',')
            .filter_map(|v| v.parse::<i32>().ok())
            .collect();

        // Abort if we've ended up with anything other than 2 values
        if parsed_values.len() != 2 {
            return Self {
                part1: None,
                part2: None,
            };
        }

        // let part1 = parsed_values.get(0).map(|&v| PartResult::Int(v));
        // let p1  = parsed_values.get(0);
        // let part1 = p1.fl
        // let p2  = parsed_values.get(1).flatten().map(|&f| PartResult::Int(f));

        Self {
            #[allow(clippy::get_first)]
            part1: parsed_values.get(0).map(|&f| -> PartResult { f.into() }),
            part2: parsed_values.get(1).map(|&f| -> PartResult { f.into() }),
        }
    }
}

pub struct SolutionOutput {
    values: DayOutput,
    duration: time::Duration,
    day_number: i32,
}

// enum DayResult{
//     NoInput,
//     DayOutput
// }

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

type DayFn = fn(&str) -> DayOutput;

fn run_day(n: i32, solution: DayFn) -> Result<SolutionOutput, NoInputFileErr> {
    let r = get_input(n)?;

    let time_start = time::Instant::now();
    let output = solution(&r);
    let duration = time_start.elapsed();

    Ok(SolutionOutput {
        values: output,
        duration,
        day_number: n,
    })
}

// #[derive(Debug)]
// pub struct Day {
//     number: i32,
//     duration: Option<time::Duration>,
//     output: Option<DayOutput>,
//     solution: fn(&str) -> DayOutput,
// }

// impl Day {
//     pub fn new(number: i32, solution: fn(&str) -> DayOutput) -> Self {
//         Self {
//             number,
//             duration: None,
//             output: None,
//             solution,
//         }
//     }

//     // fn run_solution(mut self) -> SolutionOutput {
//     //     let input = get_input(self.number).map(op)
//     //     self.output = self.solution(g);

//     //     SolutionOutput { values: (), duration: (), day_number: () }
//     // }

//     fn test(self) {}
// }

impl Display for PartResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Int(a) => a,
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

        // let p2: String = match self.part2.as_ref() {
        //     Some(n) => n.to_string(),
        //     None => "None".to_owned(),
        // };

        // self.part1.map_or("none", |p| p.fmt(&f));
        // f.write_str("|");
        // self.part1.map_or("none", |p| p.fmt(&f));
        // self.part2.and_then(|f2| f2.fmt(f));

        write!(f, "{p1}|{p2}")
    }
}

// fn print_if_ok(o: Result<DayOutput, Error>) {
//     match o {
//         Ok(some) => println!("{some}"),
//         Err(e) => println!("Error: {e}"),
//     }
// }

fn print_result(r: Result<SolutionOutput, NoInputFileErr>) {
    match r {
        Ok(s) => println!(
            "Day {:2}: {:4}ms [{}|{}]",
            s.day_number,
            s.duration.as_millis(),
            s.values.part1.unwrap_or(PartResult::Int(-1)),
            s.values.part2.unwrap_or(PartResult::Int(-1))
        ),
        Err(err) => println!(
            "Error getting input file for day {}",
            err.day_number.unwrap_or(-1)
        ),
    }
}

pub fn run() {
    print_result(run_day(1, day1::solve));
    print_result(run_day(2, day2::solve));
    // let 2 = Day::new(2, day2::solve);

    // print_if_ok(d1);
    // print_if_ok(d2);
}

// fn get_solution(day_number: i32) -> DayOutput {
//     let file_path = format!("./data/input/day{day_number}.txt");
// }

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
    // let file_path = format!("./data/input/day{day_number}.txt");
    // let mut file_contents = String::new();

    // fs::File::open(file_path)
    //     .map(|mut f| f.read_to_string(&mut file_contents))
    //     .map(|_| file_contents)
    //     .map_err(|_| NoInputFileErr { day_number })

    // Ok(file_contents)
    // fs::File::open(file_path)
    //     .map(|mut f| f.read_to_string(&mut file_contents))
    //     .and(Ok(file_contents))
    //     .ok()

    // file.read_to_string(&mut file_contents)
    //     .and(Ok(file_contents))
    //     .ok()
}

pub mod testutils {
    pub enum Part {
        Part1,
        Part2,
    }

    pub enum TestError {
        Failure(Part, i32, i32),
        Noinput,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    enum TestError {
        Failure(i32, String, String),
        NoInput,
        NoInputFile(String),
        NoResult,
    }

    impl From<TestError> for String {
        fn from(value: TestError) -> Self {
            match value {
                TestError::Failure(part, expected, actual) => {
                    format!("Expected {} got {}", expected, actual)
                }
                TestError::NoInput => format!("No input {}", ""),
                TestError::NoResult => "No result".to_owned(),
                TestError::NoInputFile(s) => format!("No input file {}", s),
            }
        }
    }

    fn get_solution(day_number: i32) -> Result<DayOutput, NoInputFileErr> {
        let path = format!("./data/solution/day{day_number}.txt");
        // read_file(&path).map_or(
        //     DayOutput {
        //         part1: None,
        //         part2: None,
        //     },
        //     |f| DayOutput::from(f.as_ref()),
        // )

        read_file(&path).map(|str| DayOutput::from(str.as_ref()))
    }

    fn compare_result(
        expected: Option<PartResult>,
        actual: Option<PartResult>,
    ) -> Result<(), TestError> {
        let e = expected.ok_or(TestError::NoResult)?;
        let i = actual.ok_or(TestError::NoResult)?;

        match e == i {
            true => Ok(()),
            false => Err(TestError::Failure(1, i.to_string(), e.to_string())),
        }
    }

    pub fn test_day(day_number: i32, solution: DayFn) -> Result<(), String> {
        let input =
            get_input(day_number).map_err(|file_error| TestError::NoInputFile(file_error.path))?;
        let expected = get_solution(day_number)?;
        let actual = solution(&input);

        compare_result(expected.part1, actual.part1)?;
        compare_result(expected.part2, actual.part2)?;

        Ok(())
    }
}
