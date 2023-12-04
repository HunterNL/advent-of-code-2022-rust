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
        closed_set.insert(cave_id, round);

        let cave = caves.iter().find(|c| c.id == cave_id).unwrap();

        for tunnel in &cave.tunnels {
            if !closed_set.contains_key(tunnel) {
                new_frontier.push(*tunnel)
            }
        }
    }

    new_frontier
}

// fn starting_paths(c: &CaveSystem) -> Vec<Path> {}

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

#[derive(Clone)]
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

    fn open_valve(&mut self, id: CaveId, rate: u32) {
        let valve: u64 = 1 << id.0;

        if self.is_valve_open(id) {
            panic!("Trying to open already open valve");
        }

        self.open_valve_rate += rate;
        self.valves_opened |= valve;
        self.valves_opened_count += 1;
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

    fn has_opened_valve(&self, id: usize) -> bool {
        self.valves_opened & (1 << id) > 0
    }

    // fn has_time_left(&self, max_time: u32) -> bool {
    //     self.minutes < max_time
    // }
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

#[derive(Clone)]
struct Path {
    world: World,
    me: Traveler,
    elephant: Traveler,
}

#[derive(Clone)]
struct Traveler {
    position: CaveId,
    goal: Option<CaveId>,
    goal_time: u32,
}

impl Traveler {
    fn is_action_time(&self, time: u32) -> bool {
        self.goal_time == time
    }

    fn act(&mut self, caves: &[Cave], world: &mut World) -> bool {
        if self.goal.is_some() {
            let goal_cave_id = self.goal.unwrap();
            let goal_cave = caves.get(goal_cave_id.0).unwrap();
            if world.is_valve_open(goal_cave_id) {
                return true;
            }
            world.open_valve(goal_cave_id, goal_cave.flow_rate);
            self.position = goal_cave_id;
            self.goal = None;
        }

        false

        // Set goal time?
    }
}

trait CavePath {
    fn append_futures<T>(&self, cave_system: &CaveSystem, queue: &mut Vec<T>);
}

impl Path {
    /// Advances the world to the point where new goals need to be assigned
    fn resolve_actions(&mut self, cave_system: &CaveSystem) -> bool {
        self.world.advance_time_to(self.next_action_time());
        let current_time = self.world.minutes;
        if self.me.is_action_time(current_time)
            && self.elephant.is_action_time(current_time)
            && self.me.goal.is_some()
            && self.elephant.goal.is_some()
            && self.me.goal.eq(&self.elephant.goal)
        {
            // self.me.act(&cave_system.caves, &mut self.world);
            // self.elephant.position = self.elephant.goal.unwrap();
            // self.elephant.goal = None;
        }

        if self.me.is_action_time(current_time) {
            let discard = self.me.act(&cave_system.caves, &mut self.world);
            if discard {
                return true;
            }
        }

        if self.elephant.is_action_time(current_time) {
            let discard = self.elephant.act(&cave_system.caves, &mut self.world);
            if discard {
                return true;
            }
        }
        false
    }

    fn futures(&mut self, cave_system: &CaveSystem, queue: &mut Vec<Path>, max_cave_time: u32) {
        // println!("Futures");
        if self.me.goal.is_none() && self.elephant.goal.is_some() {
            let current_cave = cave_system.caves.get(self.me.position.0).unwrap();

            queue.extend(
                cave_system
                    .caves_with_working_valve
                    .iter()
                    .filter(|cave_id| {
                        !self.world.has_opened_valve(cave_id.0)
                        // && !self
                        //     .elephant
                        //     .goal
                        //     .is_some_and(|ele_goal| ele_goal == **cave_id)
                    })
                    .filter_map(|target_id| {
                        let travel_time = current_cave.paths.get(target_id.0).unwrap();
                        // let target = cave_system.caves.get(target_id.0).unwrap();

                        if self.world.minutes + travel_time + 1 >= max_cave_time {
                            // +1 for effect time
                            None
                        } else {
                            let mut p = self.clone();
                            p.me.goal = Some(*target_id);
                            p.me.goal_time = p.world.minutes + travel_time + 1;
                            Some(p)
                        }
                    }),
            );
            return;
        }

        if self.elephant.goal.is_none() && self.me.goal.is_some() {
            let current_cave = cave_system.caves.get(self.elephant.position.0).unwrap();

            queue.extend(
                cave_system
                    .caves_with_working_valve
                    .iter()
                    .filter(|cave_id| {
                        !self.world.has_opened_valve(cave_id.0)
                        // && !self.me.goal.is_some_and(|me_goal| me_goal == **cave_id)
                    })
                    .filter_map(|target_id| {
                        let travel_time = current_cave.paths.get(target_id.0).unwrap();
                        // let target = cave_system.caves.get(target_id.0).unwrap();

                        if self.world.minutes + travel_time + 1 >= max_cave_time {
                            // +1 for effect time
                            None
                        } else {
                            let mut p = self.clone();
                            p.elephant.goal = Some(*target_id);
                            p.elephant.goal_time = p.world.minutes + travel_time + 1;
                            Some(p)
                        }
                    }),
            );
            return;
        }

        if self.elephant.goal.is_none() && self.me.goal.is_none() {
            // println!("Complicated case");
            // Complicated case

            let me_cave = cave_system.caves.get(self.me.position.0).unwrap();
            let elephant_cave = cave_system.caves.get(self.elephant.position.0).unwrap();

            cave_system
                .caves_with_working_valve
                .iter()
                .filter(|cave_id| !self.world.has_opened_valve(cave_id.0))
                .for_each(|me_cave_target_id| {
                    let me_travel_time = me_cave.paths.get(me_cave_target_id.0).unwrap();
                    let set_left_goal = self.world.minutes + me_travel_time + 1 < max_cave_time;

                    cave_system
                        .caves_with_working_valve
                        .iter()
                        .filter(|cave_id| !self.world.has_opened_valve(cave_id.0))
                        .for_each(|elephant_cave_target| {
                            if *me_cave_target_id == *elephant_cave_target {
                                return; // Do not allow setting the same goal on both travelers
                            }

                            let elephant_travel_time =
                                elephant_cave.paths.get(elephant_cave_target.0).unwrap();

                            let set_right_target =
                                self.world.minutes + elephant_travel_time + 1 < max_cave_time;

                            let mut p = self.clone();

                            if set_left_goal {
                                p.me.goal = Some(*me_cave_target_id);
                                p.me.goal_time = p.world.minutes + me_travel_time + 1;
                            }

                            if set_right_target {
                                p.elephant.goal = Some(*elephant_cave_target);
                                p.elephant.goal_time = p.world.minutes + elephant_travel_time + 1;
                            }

                            if set_left_goal || set_right_target {
                                queue.push(p);
                            }
                        });
                });
            return;
        }

        panic!("Shouldn't happen");
    }

    fn next_action_time(&self) -> u32 {
        self.me.goal_time.min(self.elephant.goal_time)
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

    let far_future = 99;

    let mut initial_path = Path {
        world: World::new(),
        me: Traveler {
            position: start_cave_id,
            goal: None,
            goal_time: 0,
        },
        elephant: Traveler {
            position: start_cave_id,
            goal: Some(start_cave_id),
            goal_time: far_future,
        },
    };

    initial_path
        .world
        .advance_time_to(initial_path.next_action_time());

    let mut queue = vec![initial_path];

    let mut biggest_release: u32 = 0;

    while let Some(mut path) = queue.pop() {
        if path.resolve_actions(cave_system) {
            continue;
        }
        biggest_release = biggest_release.max(path.world.pressure_at_time(30));
        path.futures(cave_system, &mut queue, 30);
    }

    biggest_release
}

fn find_biggest_release_with_elephant(cave_system: &CaveSystem) -> u32 {
    let start_cave_id = cave_system
        .cave_by_name(START_CAVE)
        .expect("start cave should be present in cave_system");

    let mut initial_path = Path {
        world: World::new(),
        me: Traveler {
            position: start_cave_id,
            goal: None,
            goal_time: 0,
        },
        elephant: Traveler {
            position: start_cave_id,
            goal: None,
            goal_time: 0,
        },
    };

    initial_path
        .world
        .advance_time_to(initial_path.next_action_time());

    let mut queue = vec![initial_path];

    let mut biggest_release: u32 = 0;

    while let Some(mut path) = queue.pop() {
        biggest_release = biggest_release.max(path.world.pressure_at_time(26));
        if path.resolve_actions(cave_system) {
            continue;
        }
        biggest_release = biggest_release.max(path.world.pressure_at_time(26));
        path.futures(cave_system, &mut queue, 26);
    }

    biggest_release
}

// https://adventofcode.com/2022/day/16
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let caves = CaveSystem::from_str(input);
    let pressure = find_biggest_release(&caves);
    let p2 = find_biggest_release_with_elephant(&caves);
    // let p2 = 0;

    Ok(DayOutput {
        part1: Some(PartResult::UInt(pressure as u64)),
        part2: Some(PartResult::UInt(p2 as u64)),
    })
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
    #[ignore = "wip"]
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
