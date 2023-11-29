use std::{
    collections::{HashMap, VecDeque},
    fmt::{Display, Write},
    str::FromStr,
    vec,
};

const MAX_CAVE_TIME: u32 = 30;

use crate::parsing::consume_when;

use super::{DayOutput, LogicError, PartResult};

static START_CAVE: CaveName = CaveName('A', 'A');

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
struct CaveName(char, char);

impl Display for CaveName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.0)?;
        f.write_char(self.1)?;

        Ok(())
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
struct CaveId(usize);

struct CaveSystem {
    caves: Vec<Cave>,
    caves_with_working_valve: Vec<CaveId>,
}

fn find_shortest(caves: &[Cave], origin: usize, target: usize) -> Vec<usize> {
    assert_ne!(origin, target);
    let mut frontier = VecDeque::new();
    frontier.push_back(vec![origin]);

    while let Some(path) = frontier.pop_front() {
        let cave_id = *path.last().unwrap();

        if cave_id == target {
            return path;
        }

        let cave = caves.get(cave_id).unwrap();
        cave.tunnels.iter().for_each(|tunnel_id| {
            let mut new_path = path.clone();
            new_path.push(tunnel_id.0);
            frontier.push_back(new_path);
        });
    }

    panic!("Path not found")
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
        let protocaves: Vec<CavePrototype> = input
            .lines()
            .map(|l| l.parse::<CavePrototype>().unwrap())
            .collect();

        let caves = Self::connect_protocaves(protocaves.as_slice());

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

        // Self::generate_paths(&mut caves);

        Self {
            caves,
            caves_with_working_valve,
        }
    }

    fn connect_protocaves(protocaves: &[CavePrototype]) -> Vec<Cave> {
        let mut name_to_id_map: HashMap<CaveName, CaveId> = HashMap::new();

        let mut caves: Vec<Cave> = protocaves
            .iter()
            .enumerate()
            .map(|(pos, cave)| Cave {
                id: CaveId(pos),
                name: cave.name,
                flow_rate: cave.flow_rate,
                paths: vec![],
                tunnels: vec![],
                tunnels_by_name: cave.tunnels.clone(),
            })
            .collect();

        for cave in &caves {
            name_to_id_map.insert(cave.name, cave.id);
        }

        caves.iter_mut().for_each(|cave| {
            cave.tunnels.extend(
                cave.tunnels_by_name
                    .iter()
                    .map(|name| *name_to_id_map.get(name).unwrap()),
            )
        });

        for origin_id in 0..caves.len() {
            for target_id in 0..caves.len() {
                if origin_id == target_id {
                    caves.get_mut(origin_id).unwrap().paths.push(255);
                    continue;
                }

                let shortest_path = find_shortest(caves.as_slice(), origin_id, target_id);

                caves
                    .get_mut(origin_id)
                    .unwrap()
                    .paths
                    .push(shortest_path.len() as u32)
            }
        }

        caves
    }

    fn cave_by_name(&self, cave_name: CaveName) -> Option<CaveId> {
        self.caves
            .iter()
            .position(|cave| cave.name == cave_name)
            .map(CaveId)
    }

    fn possible_goals(&self, valves_opened: u64) -> Vec<CaveId> {
        let mut out = vec![];
        for cave in &self.caves_with_working_valve {
            if 1 << cave.0 & valves_opened == 0 {
                out.push(*cave);
            }
        }
        out
    }
}

#[derive(Debug, Hash)]
struct Cave {
    id: CaveId,
    name: CaveName,
    flow_rate: u32,
    paths: Vec<u32>,      // Length of paths to other caves
    tunnels: Vec<CaveId>, // Direct neighbours
    tunnels_by_name: Vec<CaveName>,
}

impl Display for Cave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Cave(flow rate=")?;
        f.write_fmt(format_args!("{}", self.flow_rate))?;
        f.write_str(" tunnels are ")?;
        let a = self
            .tunnels
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
    fn open_valve(&mut self, id: CaveId, rate: u32) {
        let valve: u64 = 1 << id.0;

        // Self {
        self.minutes += 1;
        self.relieved_pressure += self.open_valve_rate;
        self.open_valve_rate += rate;
        self.valves_opened |= valve;
        self.valves_opened_count += 1;
    }

    fn travel(&self, duration: u32, destination: CaveId, rate: u32) -> Self {
        let mut a = Self {
            minutes: self.minutes + duration,
            position: destination,
            relieved_pressure: self.relieved_pressure + (self.open_valve_rate * duration),
            ..*self
        };
        a.open_valve(destination, rate);
        a
    }

    fn idle(&self) -> u32 {
        let minutes_to_idle = MAX_CAVE_TIME - self.minutes;
        self.relieved_pressure + self.open_valve_rate * minutes_to_idle
    }

    fn has_opened_valve(&self, id: usize) -> bool {
        self.valves_opened & (1 << id) > 0
    }

    fn is_done(&self) -> bool {
        self.minutes == MAX_CAVE_TIME // || self.goal.is_none()
    }

    fn append_futures(&self, cave_system: &CaveSystem, queue: &mut Vec<Path>) {
        if self.is_done() {
            return;
        }

        let current_cave = cave_system.caves.get(self.position.0).unwrap();

        queue.extend(
            cave_system
                .caves_with_working_valve
                .iter()
                .filter(|cave_id| !self.has_opened_valve(cave_id.0))
                .filter_map(|target_id| {
                    // let target_cave = cave_system.caves.get(target_id.0).unwrap();
                    let travel_time = current_cave.paths.get(target_id.0).unwrap() - 1;
                    let target = cave_system.caves.get(target_id.0).unwrap();
                    // smallest_action_time = smallest_action_time.min(*path_len); // SIDE EFFECT
                    if self.minutes + travel_time + 1 > MAX_CAVE_TIME {
                        None
                    } else {
                        Some(self.travel(travel_time, *target_id, target.flow_rate))
                    }
                }),
        );
    }
}
struct CavePrototype {
    name: CaveName,
    tunnels: Vec<CaveName>,
    flow_rate: u32,
}

impl FromStr for CavePrototype {
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

        Ok(Self {
            flow_rate,
            name,
            tunnels,
        })
    }
}

fn find_biggest_release(cave_system: CaveSystem) -> u32 {
    let start_cave_id = cave_system
        .cave_by_name(START_CAVE)
        .expect("start cave should be present in cave_system");

    let mut queue = vec![Path {
        minutes: 0,
        open_valve_rate: 0,
        valves_opened: 0,
        valves_opened_count: 0,
        relieved_pressure: 0,
        position: start_cave_id,
    }];

    let mut biggest_release: u32 = 0;

    while let Some(path) = queue.pop() {
        let queue_size = queue.len();
        path.append_futures(&cave_system, &mut queue);
        let new_queue_size = queue.len();

        let is_done = queue_size == new_queue_size;

        if is_done {
            biggest_release = biggest_release.max(path.idle())
        }
    }

    biggest_release
}

// https://adventofcode.com/2022/day/16
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let caves = CaveSystem::from_str(input);
    println!(
        "Caves with valves: {}",
        caves.caves_with_working_valve.len()
    );
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

    // fn get_cave_id()

    // #[test]
    // fn sequence() {
    //     let caves = CaveSystem::from_str(EXAMPLE_INPUT);
    //     let start_cave_id = caves.cave_by_name(super::CaveName('A', 'A')).unwrap();
    //     let first_cave_id = caves.cave_by_name(super::CaveName('D', 'D')).unwrap();

    //     let mut path = Path {
    //         minutes: 0,
    //         open_valve_rate: 0,
    //         valves_opened: 0,
    //         valves_opened_count: 0,
    //         relieved_pressure: 0,
    //         position: start_cave_id,
    //     };

    //     assert_eq!(path.idle(), 0);

    //     path = path.travel(1, start_cave_id, 20);
    //     assert_eq!(path.minutes, 2);
    // assert_eq!(path.relieved_pressure, 0)

    // path = path.travel(2, destination, rate)

    // path.
    // }
}
