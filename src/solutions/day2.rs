use crate::solutions::DayOutput;

use super::PartResult;

#[derive(Debug)]
struct GuideLine(i32, i32);

const fn radial_dif(a: i32, b: i32) -> i32 {
    ((a - b) + 3) % 3
}

impl GuideLine {
    /// Returns the score of this [`GuideLine`].
    fn score_p1(&self) -> i32 {
        let w1: i32 = radial_dif(self.1, self.0);
        let win_score: i32 = (w1 + 4) % 3;
        let shape_score: i32 = self.1;
        shape_score + 1 + win_score * 3
    }

    fn score_p2(&self) -> i32 {
        let mine: i32 = (self.0 + self.1 + 2) % 3;

        let win_score: i32 = self.1 * 3;
        let piece_score = mine + 1;

        // println!(
        //     "They play {}, I need to {} so I play {}, scoring {} for the win and {} for the piece",
        //     self.0, self.1, mine, win_score, piece_score
        // );

        win_score + piece_score
    }
}

impl From<&str> for GuideLine {
    fn from(value: &str) -> Self {
        let b = value.as_bytes();
        Self((b[0] - b'A').into(), (b[2] - b'X').into())
    }
}

// https://adventofcode.com/2022/day/2
pub fn solve(input: &str) -> DayOutput {
    let lines: Vec<GuideLine> = input
        .split('\n')
        .filter(|s| s.len() == 3)
        .map(GuideLine::from)
        .collect();

    let part1 = lines.iter().map(GuideLine::score_p1).sum();
    let part2 = lines.iter().map(GuideLine::score_p2).sum();

    DayOutput {
        part1: Some(PartResult::Int(part1)),
        part2: Some(PartResult::Int(part2)),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_radial_dif() {
        assert_eq!(super::radial_dif(2, 1), 1);
        assert_eq!(super::radial_dif(1, 1), 0);
        assert_eq!(super::radial_dif(0, 1), 2);
        assert_eq!(super::radial_dif(0, 2), 1);
        assert_eq!(super::radial_dif(2, 0), 2);
    }

    #[test]
    fn test_example() {
        let g1: super::GuideLine = "A Y".into();
        let g2: super::GuideLine = "B X".into();
        let g3: super::GuideLine = "C Z".into();
        let g4: super::GuideLine = "A Z".into(); //OP: ROCK ME: SISSORS

        assert_eq!(g1.score_p1(), 8);
        assert_eq!(g2.score_p1(), 1);
        assert_eq!(g3.score_p1(), 6);
        assert_eq!(g4.score_p1(), 3);
    }

    #[test]
    fn test_example_part2() {
        let g1: super::GuideLine = "A Y".into();
        let g2: super::GuideLine = "B X".into();
        let g3: super::GuideLine = "C Z".into();

        assert_eq!(g1.score_p2(), 4);
        assert_eq!(g2.score_p2(), 1);
        assert_eq!(g3.score_p2(), 7);
    }

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(2, super::solve)
    }
}
