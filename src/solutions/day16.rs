use std::{
    collections::{HashMap, VecDeque},
    fmt::{Display, Write},
    str::FromStr,
    vec,
};

const MAX_CAVE_TIME: u32 = 30;

use crate::parsing::consume_when;

use super::{DayOutput, LogicError, PartResult};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
struct CaveName(char, char);

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
struct CaveId(usize);

impl Display for CaveName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.0)?;
        f.write_char(self.1)?;

        Ok(())
    }
}

static START_CAVE: CaveName = CaveName('A', 'A');

struct CaveSystem {
    caves: Vec<Cave>,
    caves_with_working_valve: Vec<CaveId>,
}

// struct SearchPath(Vec<usize>);

fn find_shortest(caves: &[Cave], origin: usize, target: usize) -> usize {
    assert_ne!(origin, target);
    let mut frontier = VecDeque::new();
    frontier.push_back(vec![origin]);

    while let Some(path) = frontier.pop_front() {
        let cave_id = *path.last().unwrap();

        if cave_id == target {
            return *path.get(1).unwrap();
        }

        let cave = caves.get(cave_id).unwrap();
        cave.tunnels_id.iter().for_each(|tunnel_id| {
            let mut new_path = path.clone();
            new_path.push(tunnel_id.0);
            frontier.push_back(new_path);
        });
    }

    panic!("Path not found")
}

impl CaveSystem {
    /// Goes overy every cave and assigns the new
    fn generate_paths(caves: &mut Vec<Cave>) {
        for origin in 0..caves.len() {
            let mut paths = vec![];

            for target in 0..caves.len() {
                if origin == target {
                    paths.push(0xDEADBEEF);
                    continue;
                }
                paths.push(Self::best_step_towards(caves, origin, target));
            }

            caves.get_mut(origin).unwrap().paths = paths;
        }
    }
    /// Finds the best step to take from the `origin` cave to the `target` cave
    fn best_step_towards(caves: &[Cave], origin: usize, target: usize) -> usize {
        find_shortest(caves, origin, target)
    }

    fn possible_goals(&self, valves_opened: u64) -> Vec<CaveId> {
        let mut out = vec![];
        for cave in &self.caves_with_working_valve {
            if 1 << cave.0 & valves_opened == 0 {
                out.push(*cave);
            }
        }
        // if (out.len() < 6) {
        //     println!("{}", out.len());
        // }
        out
    }
}

impl Display for CaveSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for cave in self.caves.iter() {
            f.write_fmt(format_args!("{cave}"))?;
            f.write_char('\n')?;
        }

        Ok(())
    }
}

impl CaveSystem {
    fn from_str(input: &str) -> Self {
        let mut caves: Vec<Cave> = input.lines().map(|l| l.parse::<Cave>().unwrap()).collect();
        let mut name_map: HashMap<CaveName, CaveId> = HashMap::new();

        caves.iter().enumerate().for_each(|(index, cave)| {
            name_map.insert(cave.name, CaveId(index));
        });

        // For every cave, update its tunnels_by_id to refer to the index of
        caves.iter_mut().for_each(|cave| {
            let tunnels_by_id = cave
                .tunnels_name
                .iter()
                .map(|tunnel_name| *name_map.get(tunnel_name).expect("To find matching name"))
                .collect();

            cave.tunnels_id = tunnels_by_id
        });

        caves
            .iter_mut()
            .enumerate()
            .for_each(|(id, cave)| cave.id = CaveId(id));

        let caves_with_working_valve: Vec<CaveId> = caves
            .iter()
            .enumerate()
            .filter(|(_, cave)| cave.flow_rate > 0)
            .map(|a| CaveId(a.0))
            .collect();

        let mut default_valve_state: u64 = 0;

        caves_with_working_valve
            .iter()
            .for_each(|id| default_valve_state |= 1 << id.0);

        Self::generate_paths(&mut caves);

        Self {
            caves,
            caves_with_working_valve,
        }
    }
}

#[derive(Debug, Hash)]
struct Cave {
    id: CaveId,
    name: CaveName,
    flow_rate: u32,
    tunnels_name: Vec<CaveName>,
    tunnels_id: Vec<CaveId>,
    paths: Vec<usize>,
}

impl Display for Cave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Cave(flow rate=")?;
        f.write_fmt(format_args!("{}", self.flow_rate))?;
        f.write_str(" tunnels are ")?;
        let a = self
            .tunnels_id
            .iter()
            .map(|t| t.0.to_string())
            .collect::<Vec<String>>()
            .join(",");
        f.write_str(&a)?;

        Ok(())
    }
}

struct Path {
    minutes: u32,
    open_valve_rate: u32,
    valves_opened: u64,
    valves_opened_count: usize,
    relieved_pressure: u32,
    position: CaveId,
    goal: Option<CaveId>,
}

fn possible_goals(valves_opened: u64, cave_count: u32) -> Vec<CaveId> {
    let mut out = vec![];
    for n in 0..cave_count {
        if 1 << n & valves_opened == 0 {
            out.push(CaveId(n as usize));
        }
    }
    out
}

// //u32  PPPPPPPP

impl Path {
    #[must_use]
    fn open_valve(&self, id: CaveId, rate: u32) -> Self {
        let valve: u64 = 1 << id.0;

        Self {
            minutes: self.minutes + 1,
            open_valve_rate: self.open_valve_rate + rate,
            valves_opened: self.valves_opened | valve,
            // valves_remaining: self.valves_remaining & ^valve,
            valves_opened_count: self.valves_opened_count + 1,
            relieved_pressure: self.relieved_pressure + self.open_valve_rate, // Exclude the rate, as the newly opened valve doesn't count yet
            ..*self
        }
    }

    fn travel(&self, destination: CaveId) -> Self {
        Self {
            minutes: self.minutes + 1,
            position: destination,
            relieved_pressure: self.relieved_pressure + self.open_valve_rate,
            ..*self
        }
    }

    fn idle(&self) -> u32 {
        let minutes_to_idle = MAX_CAVE_TIME - self.minutes;
        self.relieved_pressure + self.open_valve_rate * minutes_to_idle

        // Self {
        //     minutes: self.minutes + minutes_to_idle,
        //     relieved_pressure: ,
        //     ..*self
        // }
    }

    fn set_goal(&self, goal_cave: Option<CaveId>) -> Self {
        Self {
            goal: goal_cave,
            ..*self
        }
    }

    fn has_opened_valve(&self, id: usize) -> bool {
        self.valves_opened & (1 << id) > 0
    }

    fn is_done(&self) -> bool {
        self.minutes == MAX_CAVE_TIME || self.goal.is_none()
    }

    fn append_futures(&self, cave_system: &CaveSystem, queue: &mut Vec<Path>) {
        assert!(!self.is_done());

        let current_cave = cave_system.caves.get(self.position.0).unwrap();

        // We've reached the goal
        if self.goal.is_some_and(|goal| goal == self.position) {
            let path = self.open_valve(self.position, current_cave.flow_rate);

            let goals = cave_system.possible_goals(path.valves_opened);

            // We're done
            if goals.is_empty() {
                queue.push(path.set_goal(None));
            } else {
                //Pick a new goal
                for goal in goals {
                    queue.push(path.set_goal(Some(goal)));
                }
            }
        } else {
            // Else travel to the goal
            let next_id = current_cave
                .paths
                .get(self.goal.expect("Goal to exsist").0)
                .expect("Path to have a path to goal");

            queue.push(self.travel(CaveId(*next_id)));
        }
    }
}

impl FromStr for Cave {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars().skip(6);
        let a = chars.next().unwrap();
        let b = chars.next().unwrap();
        let name = CaveName(a, b);
        let flow_rate = consume_when(&mut chars, &char::is_ascii_digit)
            .iter()
            .collect::<String>()
            .parse()
            .expect("Valid flow rate");

        let mut tunnels = vec![];

        loop {
            let id = consume_when(&mut chars, &char::is_ascii_uppercase);
            if id.is_empty() {
                break;
            }
            tunnels.push(CaveName(*id.first().unwrap(), *id.last().unwrap()))
        }

        Ok(Cave {
            id: CaveId(0xDEADBEEF), //I'm prematurely optimizing Option away without measuring or profiling, sue me
            flow_rate,
            name,
            tunnels_name: tunnels,
            tunnels_id: vec![],
            paths: vec![],
        })
    }
}

fn find_biggest_release(cave_system: CaveSystem) -> u32 {
    let start_cave_id = cave_system
        .caves
        .iter()
        .position(|cave| cave.name == START_CAVE)
        .expect("start cave should be present in cave_system");

    let mut queue = vec![];

    let goals = cave_system.possible_goals(0);

    queue.extend(goals.into_iter().map(|caveid| Path {
        minutes: 0,
        open_valve_rate: 0,
        valves_opened: 0,
        valves_opened_count: 0,
        relieved_pressure: 0,
        position: CaveId(start_cave_id),
        goal: Some(caveid),
    }));

    let mut biggest_release: u32 = 0;

    while let Some(path) = queue.pop() {
        if path.is_done() {
            biggest_release = biggest_release.max(path.idle())
        } else {
            path.append_futures(&cave_system, &mut queue);
        }
    }

    biggest_release
}

// https://adventofcode.com/2022/day/16
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let caves = CaveSystem::from_str(input);
    // caves.generate_paths();
    let pressure = find_biggest_release(caves);

    Ok(DayOutput {
        part1: Some(PartResult::UInt(pressure as u64)),
        part2: None,
    })
}

#[cfg(test)]
mod tests {

    use crate::solutions::day16::CaveSystem;

    use super::find_biggest_release;

    static EXAMPLE_INPUT: &str = "Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II";

    #[test]
    #[ignore = "wip"]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(16, super::solve)
    }

    #[test]
    fn example() {
        let caves = CaveSystem::from_str(EXAMPLE_INPUT);
        let pressure = find_biggest_release(caves);

        assert_eq!(pressure, 1651);
    }
}
