use std::{
    cmp::{self, Ordering},
    iter::Peekable,
    str::FromStr,
};

use super::{DayOutput, LogicError};

#[derive(Debug, PartialEq, Eq)]
enum ListItem {
    List(Vec<ListItem>),
    Int(i32),
}

impl PartialOrd for ListItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn compare_lists(left_list: &Vec<ListItem>, right_list: &Vec<ListItem>) -> std::cmp::Ordering {
    let fallback = left_list.len().cmp(&right_list.len());

    left_list
        .iter()
        .zip(right_list.iter())
        .find_map(|(left, right)| match left.cmp(right) {
            std::cmp::Ordering::Less => Some(std::cmp::Ordering::Less),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(std::cmp::Ordering::Greater),
        })
        .unwrap_or(fallback)
}

impl Ord for ListItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            ListItem::List(left_list) => match other {
                ListItem::List(right_list) => compare_lists(left_list, right_list),
                ListItem::Int(right_int) => {
                    compare_lists(left_list, &vec![ListItem::Int(*right_int)])
                }
            },
            ListItem::Int(left_int) => match other {
                ListItem::List(right_list) => {
                    compare_lists(&vec![ListItem::Int(*left_int)], right_list)
                }
                ListItem::Int(right_int) => left_int.cmp(right_int),
            },
        }
    }
}

// fn parse(iter: Peekable<Iterator<Item = char>>) -> Option<i32> {

fn read_int<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Option<ListItem> {
    let mut s = String::new();
    while let Some(digit) = iter.next_if(char::is_ascii_digit) {
        s.push(digit)
    }

    s.parse().map(ListItem::Int).ok()
}

fn read_item<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Option<ListItem> {
    let peek = *iter.peek().unwrap();
    if peek == '[' {
        read_list(iter)
    } else {
        read_int(iter)
    }
}

// Reads a list, iterator should not have consumed the starting bracket
fn read_list<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Option<ListItem> {
    assert_eq!(
        iter.next().unwrap(),
        '[',
        "Should open with an open bracket"
    ); // Consume the open bracket

    let mut out = vec![];

    loop {
        if let Some(item) = read_item(iter) {
            out.push(item)
        }

        if iter.next_if_eq(&']').is_some() {
            return Some(ListItem::List(out));
        }

        assert_eq!(
            iter.next().expect("Not to overrun iter"),
            ',',
            "Should consume a comma after a list item"
        )
    }
}

impl FromStr for ListItem {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.chars().peekable();

        read_item(&mut iter).ok_or("Parse error".to_owned())
    }
}

fn sum_indexes(packages: &[ListItem]) -> usize {
    let mut score: usize = 0;

    for chunks in packages.chunks(2).enumerate() {
        if chunks.1[0].cmp(&chunks.1[1]) == Ordering::Less {
            score += chunks.0 + 1
        }
    }
    score
}

fn decoder_key(mut packages: Vec<ListItem>) -> i32 {
    packages.push(ListItem::from_str("[[2]]").expect("divider 2 to parse"));
    packages.push(ListItem::from_str("[[6]]").expect("divider 6 to parse"));

    packages.sort();

    let scantarget_1 = ListItem::from_str("[[2]]").expect("divider 2 to parse");
    let scantarget_2 = ListItem::from_str("[[6]]").expect("divider 6 to parse");

    let pos_1 = packages
        .iter()
        .position(|item| *item == scantarget_1)
        .expect("To find scan target 1")
        + 1;
    let pos_2 = packages
        .iter()
        .position(|item| *item == scantarget_2)
        .expect("to find scan target 2")
        + 1;

    (pos_1 * pos_2) as i32
}

// https://adventofcode.com/2022/day/13
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let lines: Result<Vec<ListItem>, _> = input
        .lines()
        .filter(|line| !line.is_empty())
        .map(ListItem::from_str)
        .collect();

    let lines = lines.expect("Everything to parse");
    let index_sum = sum_indexes(&lines);

    Ok(DayOutput {
        part1: Some(super::PartResult::Int(index_sum as i32)),
        part2: Some(super::PartResult::Int(decoder_key(lines))),
    })
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, str::FromStr};

    use crate::solutions::day13::{decoder_key, sum_indexes, ListItem};

    fn test_strs(left: &str, right: &str, expected_ordering: std::cmp::Ordering) {
        assert_eq!(
            left.parse::<ListItem>()
                .expect("left side should parse")
                .cmp(&right.parse::<ListItem>().expect("Right side should parse")),
            expected_ordering
        )
    }

    fn parse_example_input() -> Vec<ListItem> {
        let input = "[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]
";

        let lines: Result<Vec<ListItem>, _> = input
            .lines()
            .filter(|line| !line.is_empty())
            .map(ListItem::from_str)
            .collect();

        lines.expect("Everything to parse")
    }

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(13, super::solve)
    }

    #[test]
    fn int() {
        test_strs("1", "2", Ordering::Less);
        test_strs("2", "1", Ordering::Greater);
        test_strs("4", "4", Ordering::Equal);
    }

    #[test]
    fn list_simple() {
        test_strs("[1]", "[2]", Ordering::Less);
        test_strs("[2]", "[1]", Ordering::Greater);
    }

    #[test]
    fn examples() {
        test_strs("[1,1,3,1,1]", "[1,1,5,1,1]", Ordering::Less);
        test_strs("[[1],[2,3,4]]", "[[1],4]", Ordering::Less);
        test_strs("[9]", "[[8,7,6]]", Ordering::Greater);

        test_strs("[[4,4],4,4]", "[[4,4],4,4,4]", Ordering::Less);

        test_strs("[7,7,7,7]", "[7,7,7]", Ordering::Greater);

        test_strs("[]", "[3]", Ordering::Less);

        test_strs("[[[]]]", "[[]]", Ordering::Greater);

        test_strs(
            "[1,[2,[3,[4,[5,6,7]]]],8,9]",
            "[1,[2,[3,[4,[5,6,0]]]],8,9]",
            Ordering::Greater,
        );
    }

    #[test]
    fn example_count() {
        assert_eq!(sum_indexes(&parse_example_input()), 13)
    }

    #[test]
    fn example_decoder() {
        assert_eq!(decoder_key(parse_example_input()), 140)
    }
}
