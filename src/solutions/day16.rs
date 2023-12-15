use core::panic;
use std::{
    collections::HashMap,
    fmt::{Display, Write},
    str::FromStr,
    vec,
};

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

impl From<(char, char)> for CaveName {
    fn from(value: (char, char)) -> Self {
        CaveName(value.0, value.1)
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
struct CaveId(usize);

impl From<CaveId> for usize {
    fn from(value: CaveId) -> Self {
        value.0
    }
}

struct CaveSystem {
    caves: Vec<Cave>,
    caves_with_working_valve: Vec<CaveId>,
}

fn explore_round(
    caves: &[Cave],
    closed_set: &mut HashMap<CaveId, u32>,
    frontier: Vec<CaveId>,
    round: u32,
) -> Vec<CaveId> {
    let mut new_frontier = vec![];

    for cave_id in frontier {
        closed_set.entry(cave_id).or_insert(round);
        // closed_set.insert(cave_id, round);

        let cave = caves.iter().find(|c| c.id == cave_id).unwrap();

        for tunnel in &cave.tunnels {
            if !closed_set.contains_key(tunnel) {
                new_frontier.push(*tunnel)
            }
        }
    }

    new_frontier
}

fn calc_distances(caves: &mut Vec<Cave>, origin: usize) {
    let mut seen = HashMap::new();
    let mut frontier = vec![CaveId(origin)];

    // Build up closed set
    let mut round = 0;
    while !frontier.is_empty() {
        frontier = explore_round(caves, &mut seen, frontier, round);
        round += 1;
    }

    for cave_id in 0..caves.len() {
        if cave_id == origin {
            caves.get_mut(origin).unwrap().paths.push(255);
            continue;
        }
        caves
            .get_mut(origin)
            .unwrap()
            .paths
            .push(*seen.get(&CaveId(cave_id)).unwrap());
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
            calc_distances(&mut caves, origin_id)
        }

        caves
    }

    fn cave_by_name(&self, cave_name: CaveName) -> Option<CaveId> {
        self.caves
            .iter()
            .position(|cave| cave.name == cave_name)
            .map(CaveId)
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

#[derive(Clone, Debug)]
struct World {
    minutes: u32,
    open_valve_rate: u32,
    valves_opened: u64,
    valves_opened_count: usize,
    relieved_pressure: u32,
}

impl World {
    fn new() -> Self {
        World {
            minutes: 0,
            open_valve_rate: 0,
            valves_opened: 0,
            valves_opened_count: 0,
            relieved_pressure: 0,
        }
    }

    fn is_valve_open(&self, id: CaveId) -> bool {
        let valve: u64 = 1 << id.0;
        self.valves_opened & valve > 0
    }

    fn closed_valves<'a>(&'a self, cave_system: &'a CaveSystem) -> impl Iterator<Item = &CaveId> {
        cave_system
            .caves_with_working_valve
            .iter()
            .filter(|cave| !self.is_valve_open(**cave))
    }

    fn open_valve(&mut self, id: CaveId, rate: u32) -> bool {
        if self.is_valve_open(id) {
            return true;
        }
        let valve: u64 = 1 << id.0;

        self.open_valve_rate += rate;
        self.valves_opened |= valve;
        self.valves_opened_count += 1;

        false
    }

    fn advance_time(&mut self, duration: u32) {
        self.minutes += duration;
        self.relieved_pressure += self.open_valve_rate * duration;
    }

    fn advance_time_to(&mut self, time: u32) {
        assert!(self.minutes <= time); // equal = nop
        self.advance_time(time - self.minutes);
    }

    fn pressure_at_time(&self, time: u32) -> u32 {
        assert!(time >= self.minutes);
        let duration = time - self.minutes;
        self.relieved_pressure + (self.open_valve_rate * duration)
    }
}

#[derive(Clone, Debug)]
struct Path {
    world: World,
    me: Traveler,
    elephant: Traveler,
}

#[derive(Clone, Debug)]
struct Traveler {
    position: CaveId,
    goal: Goal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Goal {
    MoveTo(CaveId, u32, u32),
    Idle,
    None,
}

impl Traveler {
    fn is_action_time(&self, time: u32) -> bool {
        match self.goal {
            Goal::MoveTo(_, t, _) => t == time,
            Goal::Idle => false,
            Goal::None => true,
        }
    }
}

impl Path {
    fn futures(
        &mut self,
        cave_system: &CaveSystem,
        queue: &mut Vec<Path>,
        max_cave_time: u32,
        left_options: &mut Vec<Goal>,
        right_options: &mut Vec<Goal>,
        max: &mut u32,
    ) {
        let time = self.world.minutes;
        if time == max_cave_time {
            let res = self.world.pressure_at_time(max_cave_time);
            if res > *max {
                *max = res;
            }
            return;
        }

        if time > max_cave_time || (self.me.goal == Goal::Idle && self.elephant.goal == Goal::Idle)
        {
            return;
        }

        left_options.clear();
        right_options.clear();

        if self.me.is_action_time(time) {
            let abort = match &self.me.goal {
                Goal::MoveTo(id, _, rate) => {
                    self.me.position = *id;
                    self.world.open_valve(*id, *rate)
                }
                Goal::Idle => panic!("Unepexted idle hit2"),
                Goal::None => false,
            };
            if abort {
                return;
            } else {
                let me_cave = cave_system.caves.get(self.me.position.0).unwrap();
                left_options.push(Goal::Idle);
                left_options.extend(
                    self.world
                        .closed_valves(cave_system)
                        .filter(|cave| {
                            let effect_time = me_cave.paths.get(cave.0).unwrap() + 1;
                            self.world.minutes + effect_time < max_cave_time
                        })
                        .map(|cave| {
                            let effect_time = me_cave.paths.get(cave.0).unwrap() + 1;
                            let rate = cave_system.caves.get(cave.0).unwrap().flow_rate;
                            Goal::MoveTo(*cave, self.world.minutes + effect_time, rate)
                        }),
                );
            }
        } else {
            left_options.push(self.me.goal.clone());
        }

        if self.elephant.is_action_time(time) {
            let abort = match &self.elephant.goal {
                Goal::MoveTo(id, _, rate) => {
                    self.elephant.position = *id;
                    self.world.open_valve(*id, *rate)
                }
                Goal::Idle => panic!("Unepexted idle hit2"),
                Goal::None => false,
            };
            if abort {
                return;
            } else {
                let ele_cave = cave_system.caves.get(self.elephant.position.0).unwrap();
                right_options.push(Goal::Idle);
                right_options.extend(
                    self.world
                        .closed_valves(cave_system)
                        .filter(|cave| {
                            let effect_time = ele_cave.paths.get(cave.0).unwrap() + 1;
                            self.world.minutes + effect_time < max_cave_time
                        })
                        .map(|cave| {
                            let effect_time = ele_cave.paths.get(cave.0).unwrap() + 1;
                            let rate = cave_system.caves.get(cave.0).unwrap().flow_rate;
                            Goal::MoveTo(*cave, self.world.minutes + effect_time, rate)
                        }),
                );
            }

            // return self.world.pressure_at_time(max_cave_time);
        } else {
            right_options.push(self.elephant.goal.clone());
        }

        left_options.iter().for_each(|left_option| {
            right_options.iter().for_each(|right_option| {
                let mut p = self.clone();
                p.me.goal = left_option.clone();
                p.elephant.goal = right_option.clone();
                queue.push(p);
            });
        });
    }

    fn next_action_time(&self, max_cave_time: u32) -> u32 {
        let me_time = match self.me.goal {
            Goal::MoveTo(_, time, _) => time,
            Goal::Idle => max_cave_time,
            Goal::None => 0,
        };

        let ele_time = match self.elephant.goal {
            Goal::MoveTo(_, time, _) => time,
            Goal::Idle => max_cave_time,
            Goal::None => 0,
        };

        me_time.min(ele_time).min(max_cave_time)
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

fn find_biggest_release(cave_system: &CaveSystem) -> u32 {
    let start_cave_id = cave_system
        .cave_by_name(START_CAVE)
        .expect("start cave should be present in cave_system");

    let initial_path = Path {
        // history: vec![],
        // debug: 0,
        world: World::new(),
        me: Traveler {
            position: start_cave_id,
            goal: Goal::None,
        },
        elephant: Traveler {
            position: start_cave_id,
            goal: Goal::Idle,
        },
    };

    let mut queue = vec![initial_path];

    let mut biggest_release: u32 = 0;
    // let mut iter = 0;

    let mut left = vec![];
    let mut right = vec![];

    while let Some(mut path) = queue.pop() {
        // path.world.advance_time_to(path.next_action_time());
        // biggest_release = biggest_release.max(path.world.pressure_at_time(30));
        path.world.advance_time_to(path.next_action_time(30));
        // biggest_release = pressure.max(biggest_release);

        path.futures(
            cave_system,
            &mut queue,
            30,
            &mut left,
            &mut right,
            &mut biggest_release,
        );
    }

    biggest_release
}

fn find_biggest_release_with_elephant(cave_system: &CaveSystem) -> u32 {
    let start_cave_id = cave_system
        .cave_by_name(START_CAVE)
        .expect("start cave should be present in cave_system");

    let mut queue = vec![Path {
        // history: vec![],
        // debug: 0,
        world: World::new(),
        me: Traveler {
            position: start_cave_id,
            goal: Goal::None,
        },
        elephant: Traveler {
            position: start_cave_id,
            goal: Goal::None,
        },
    }];

    let mut left = vec![];
    let mut right = vec![];

    let mut biggest_release: u32 = 0;

    while let Some(mut path) = queue.pop() {
        // path.resolve_actions(cave_system, 26);
        // biggest_release = biggest_release.max(path.world.pressure_at_time(26));
        path.world.advance_time_to(path.next_action_time(26));
        path.futures(
            cave_system,
            &mut queue,
            26,
            &mut left,
            &mut right,
            &mut biggest_release,
        );
    }

    biggest_release
}

// https://adventofcode.com/2022/day/16
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let caves = CaveSystem::from_str(input);

    // println!("{}", caves);
    let pressure = find_biggest_release(&caves);
    // let p2 = find_biggest_release_with_elephant(&caves);

    // println!("{},{}", pressure, p2);
    // let p2 = 0;

    if true {
        Ok(DayOutput {
            part1: Some(PartResult::UInt(pressure as u64)),
            part2: Some(PartResult::Str("it slow".to_owned())),
        })
    } else {
        Ok(DayOutput {
            part1: Some(PartResult::UInt(pressure as u64)),
            part2: Some(PartResult::UInt(0)),
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::solutions::day16::CaveSystem;

    use super::{find_biggest_release, find_biggest_release_with_elephant, START_CAVE};

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
    #[ignore = "performance"]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(16, super::solve)
    }

    #[test]
    fn example() {
        let caves = CaveSystem::from_str(EXAMPLE_INPUT);
        let pressure = find_biggest_release(&caves);

        assert_eq!(pressure, 1651);
    }

    #[test]
    fn example_p2() {
        let caves = CaveSystem::from_str(EXAMPLE_INPUT);
        let pressure = find_biggest_release_with_elephant(&caves);

        assert_eq!(pressure, 1707)
    }

    #[test]
    fn example_pathfinding() {
        let caves = CaveSystem::from_str(EXAMPLE_INPUT);
        let start_cave = caves.cave_by_name(START_CAVE).unwrap();
        let c = caves.caves.get(start_cave.0).unwrap();

        [('D', 'D'), ('I', 'I'), ('B', 'B')]
            .into_iter()
            .map(|a| a.into())
            .map(|name| caves.cave_by_name(name).unwrap())
            .for_each(|neighbour_cave_id| {
                assert_eq!(*c.paths.get(neighbour_cave_id.0).unwrap(), 1);
            });
    }
}
