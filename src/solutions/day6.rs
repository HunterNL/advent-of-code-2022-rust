use super::{DayOutput, LogicError, PartResult};

fn find_first_unique_character_window(haystack: &str, window_size: usize) -> Option<i32> {
    let b = haystack.as_bytes();
    for i in 0..(b.len() - window_size) {
        let slice: &[u8] = &b[i..i + window_size];
        if has_unqiue_characters(slice) {
            return i32::try_from(i + window_size).ok();
        }
    }

    None
}

fn has_unqiue_characters(slice: &[u8]) -> bool {
    for (i1, c1) in slice.iter().enumerate() {
        for (i2, c2) in slice.iter().enumerate() {
            if i1 == i2 {
                continue;
            };
            if c1 == c2 {
                return false;
            }
        }
    }

    true
}

// https://adventofcode.com/2022/day/6
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let p1 = find_first_unique_character_window(input, 4).expect("valid input");
    let p2 = find_first_unique_character_window(input, 14).expect("valid input");

    Ok(DayOutput {
        part1: Some(PartResult::Int(p1)),
        part2: Some(PartResult::Int(p2)),
    })
}

#[cfg(test)]
mod tests {
    use super::find_first_unique_character_window;

    #[test]
    fn example1() {
        assert_eq!(
            find_first_unique_character_window(
                "mjqjpqmgbljsphdztnvjfqwrcgsmlb".to_string().as_str(),
                4
            )
            .unwrap(),
            7
        );
    }

    #[test]
    fn example2() {
        assert_eq!(
            find_first_unique_character_window(
                "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw".to_string().as_str(),
                4
            )
            .unwrap(),
            11
        );
    }

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(6, super::solve)
    }
}
