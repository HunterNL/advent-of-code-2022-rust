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

        // let mut default_valve_state: u64 = 0;

        // caves_with_working_valve
        // .iter()
        // .for_each(|id| default_valve_state |= 1 << id.0);

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

    // fn possible_goals(&self, valves_opened: u64) -> Vec<CaveId> {
    //     let mut out = vec![];
    //     for cave in &self.caves_with_working_valve {
    //         if 1 << cave.0 & valves_opened == 0 {
    //             out.push(*cave);
    //         }
    //     }
    //     out
    // }
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

    // fn has_time_left(&self, max_time: u32) -> bool {
    //     self.minutes < max_ti            me
    // }
}

// fn possible_goals(valves_opened: u64, cave_count: u32) -> Vec<CaveId> {
//     let mut out = vec![];
//     for n in 0..cave_count {
//         if 1 << n & valves_opened == 0 {
//             out.push(CaveId(n as usize));
//         }
//     }
//     out
// }

#[derive(Clone, Debug)]
struct Path {
    world: World,
    me: Traveler,
    elephant: Traveler,
    // debug: u8,
    // history: Vec<HistoryItem>,
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

// #[derive(Clone, Debug)]
// enum HistoryItem {
//     MoveComplete(bool, u32, usize),
//     ValveOpened(bool, u32, usize),
//     GoalReachedOpen(bool, u32, usize),
//     GoalConflictReset(u32),
//     FoundValveOpen(bool, u32, usize),
//     SetOpenValveGoal(bool, u32, usize),
//     WentToIdle(bool, u32, usize),
// }

// enum Action {
//     Move,
//     Open,
//     Reserve,
// }

impl Traveler {
    fn is_action_time(&self, time: u32) -> bool {
        match self.goal {
            Goal::MoveTo(_, t, _) => t == time,
            Goal::Idle => false,
            Goal::None => true,
        }
    }

    // fn set_idling(&mut self) {
    //     self.goal = Some(Goal::Idle);
    //     self.goal_time = 99;
    // }

    // fn append_futures(
    //     &self,
    //     cave_system: &CaveSystem,
    //     queue: &mut Vec<Path>,
    //     path: &Path,
    //     max_cave_time: u32,
    //     is_elephant: bool,
    //     minutes: u32,
    // ) {
    //     let current_cave = cave_system.caves.get(self.position.0).unwrap();
    //     let option_count = cave_system
    //         .caves_with_working_valve
    //         .iter()
    //         .filter(|cave_id| {
    //             !path.world.is_valve_open(**cave_id)
    //                 && !(self.position == **cave_id)
    //                 && minutes + current_cave.paths.get(cave_id.0).unwrap() < max_cave_time
    //         })
    //         .count();

    //     if option_count == 0 {
    //         let mut p = path.clone();
    //         if is_elephant {
    //             p.elephant.set_idling()
    //         } else {
    //             p.me.set_idling();
    //         }
    //         queue.push(p);
    //         return;
    //     }

    //     queue.extend(
    //         cave_system
    //             .caves_with_working_valve
    //             .iter()
    //             .filter(|cave_id| {
    //                 !path.world.is_valve_open(**cave_id)
    //                     && !(self.position == **cave_id)
    //                     && minutes + current_cave.paths.get(cave_id.0).unwrap() < max_cave_time
    //             })
    //             .map(|target_id| {
    //                 let mut p = path.clone();
    //                 let travel_time = current_cave.paths.get(target_id.0).unwrap();

    //                 if is_elephant {
    //                     // p.elephant.goal = Some(Goal::MoveTo(*target_id));
    //                     // p.elephant.goal_time = path.world.minutes + travel_time;
    //                 } else {
    //                     // p.me.goal = Some(Goal::MoveTo(*target_id));
    //                     // p.me.goal_time = path.world.minutes + travel_time;
    //                 }
    //                 p
    //             }),
    //     );
    // }

    // fn act(
    //     &mut self,
    //     caves: &[Cave],
    //     world: &mut World,
    //     // hist: &mut Vec<HistoryItem>,
    //     is_elephant: bool,
    //     inhibit_valve: bool,
    // ) -> Option<CaveId> {
    //     if let Some(goal) = &self.goal {
    //         match goal {
    //             Goal::MoveTo(caveid) => {
    //                 self.position = *caveid;
    //                 // hist.push(HistoryItem::MoveComplete(
    //                 //     is_elephant,
    //                 //     world.minutes,
    //                 //     self.position.0,
    //                 // ));

    //                 if world.is_valve_open(self.position) || inhibit_valve {
    //                     self.goal = None;
    //                     // hist.push(HistoryItem::FoundValveOpen(
    //                     //     is_elephant,
    //                     //     world.minutes,
    //                     //     self.position.0,
    //                     // ));

    //                     None
    //                 } else {
    //                     self.goal = Some(Goal::OpenValve);
    //                     self.goal_time += 1;
    //                     // hist.push(HistoryItem::SetOpenValveGoal(
    //                     //     is_elephant,
    //                     //     world.minutes,
    //                     //     self.position.0,
    //                     // ));
    //                     Some(self.position)
    //                 }
    //             }
    //             Goal::OpenValve => {
    //                 let goal_cave = caves.get(self.position.0).unwrap();
    //                 world.open_valve(self.position, goal_cave.flow_rate);
    //                 // hist.push(HistoryItem::ValveOpened(
    //                 //     is_elephant,
    //                 //     world.minutes,
    //                 //     self.position.0,
    //                 // ));
    //                 self.goal = None;
    //                 None
    //             }
    //             Goal::Idle => {
    //                 // hist.push(HistoryItem::WentToIdle(
    //                 //     is_elephant,
    //                 //     world.minutes,
    //                 //     self.position.0,
    //                 // ));
    //                 panic!("Idle should always be set with a goal_time that'll never be reached")
    //             }
    //         }
    //     } else {
    //         None
    //     }

    //     // false

    //     // Set goal time?
    // }
}

// trait CavePath {
// fn append_futures<T>(&self, cave_system: &CaveSystem, queue: &mut Vec<T>);
// }

impl Path {
    // /// Advances the world to the point where new goals need to be assigned
    // fn resolve_actions(&mut self, cave_system: &CaveSystem, max_cave_time: u32) {
    //     let next_action_time = self.next_action_time();
    //     if next_action_time > max_cave_time {
    //         return;
    //     }

    //     self.world.advance_time_to(next_action_time);
    //     let current_time = self.world.minutes;

    //     // println!("------------------");
    //     // println!(
    //     //     "{:?} {:?} {}",
    //     //     self.me.goal, self.me.position, self.me.goal_time
    //     // );
    //     // println!(
    //     //     "{:?} {:?} {}",
    //     //     self.elephant.goal, self.elephant.position, self.elephant.goal_time
    //     // );

    //     let mut inhibit_valve = false;
    //     if self.me.is_action_time(current_time)
    //         && self.elephant.is_action_time(current_time)
    //         && (self.me.goal.is_some() && self.elephant.goal.is_some())
    //     {
    //         match self.me.goal.as_ref().unwrap() {
    //             Goal::MoveTo(pos) => match self.elephant.goal.as_ref().unwrap() {
    //                 Goal::MoveTo(_) => (),
    //                 Goal::OpenValve => {
    //                     if self.elephant.position == *pos {
    //                         inhibit_valve = true
    //                     }
    //                 }
    //                 Goal::Idle => (),
    //             },
    //             Goal::OpenValve => (),
    //             Goal::Idle => (),
    //         }
    //     }

    //     // let mut reserved_valve = None;

    //     if self.me.is_action_time(current_time) {
    //         self.me.act(
    //             &cave_system.caves,
    //             &mut self.world,
    //             // &mut self.history,
    //             false,
    //             inhibit_valve,
    //         );
    //     }

    //     // let inhibit_valve: bool = if reserved_valve.is_some() {
    //     //     match &self.elephant.goal {
    //     //         Some(goal) => match goal {
    //     //             Goal::MoveTo(_) => self.elephant.position == reserved_valve.unwrap(),
    //     //             Goal::OpenValve => false,
    //     //             Goal::Idle => false,
    //     //         },
    //     //         None => false,
    //     //     }
    //     // } else {
    //     //     false
    //     // };

    //     // inhibit_valve.then(|| println!("Valve inhibit")); // No HL3

    //     if self.elephant.is_action_time(current_time) {
    //         // let ele_reserved_valve =
    //         self.elephant.act(
    //             &cave_system.caves,
    //             &mut self.world,
    //             // &mut self.history,
    //             true,
    //             false,
    //         );
    //     }

    //     // if let Some(pos) = ele_reserved_valve {
    //     //     // println!("{:?}", self);
    //     //     if pos == self.me.position
    //     //         && self.me.goal.as_ref().is_some_and(|g| *g == Goal::OpenValve)
    //     //     {
    //     //         // println!("Resetting goal!");
    //     //         // self.me.goal = None;
    //     //     }
    //     // }
    //     // }

    //     // if self.me.position == self.elephant.position {
    //     //     if (self.me.goal == Some(Goal::OpenValve)) {
    //     //         if (self.elephant.goal == Some(Goal::OpenValve)) {
    //     //             println!("equal pos");
    //     //         }
    //     //     }
    //     // }

    //     if self.elephant.goal.is_some()
    //         && self.me.goal.is_some()
    //         && (self.me.position == self.elephant.position)
    //         && (self.me.goal == Some(Goal::OpenValve)
    //             && self.elephant.goal == Some(Goal::OpenValve))
    //     {
    //         // println!("Goal conflict!");

    //         // println!("{:?}", self.me);
    //         // println!("{:?}", self.elephant);

    //         // self.history
    //         // .push(HistoryItem::GoalConflictReset(self.world.minutes));

    //         // self.debug = 1;
    //         self.elephant.goal = None;
    //         self.elephant.goal_time -= 1;
    //     }
    // }

    fn futures(
        &mut self,
        cave_system: &CaveSystem,
        queue: &mut Vec<Path>,
        max_cave_time: u32,
        left_options: &mut Vec<Goal>,
        right_options: &mut Vec<Goal>,
    ) -> u32 {
        let time = self.world.minutes;
        if time > max_cave_time || (self.me.goal == Goal::Idle && self.elephant.goal == Goal::Idle)
        {
            return self.world.pressure_at_time(max_cave_time);
        } else {
            // queue.push(self.clone());
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
                return self.world.pressure_at_time(max_cave_time);
            }

            let me_cave = cave_system.caves.get(self.me.position.0).unwrap();
            left_options.push(Goal::Idle);
            left_options.extend(
                self.world
                    .closed_valves(cave_system)
                    .filter(|cave| {
                        let effect_time = me_cave.paths.get(cave.0).unwrap() + 1;
                        self.world.minutes + effect_time <= max_cave_time
                    })
                    .map(|cave| {
                        let effect_time = me_cave.paths.get(cave.0).unwrap() + 1;
                        let rate = cave_system.caves.get(cave.0).unwrap().flow_rate;
                        Goal::MoveTo(*cave, self.world.minutes + effect_time, rate)
                    }),
            );
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
                return self.world.pressure_at_time(max_cave_time);
            }

            let ele_cave = cave_system.caves.get(self.elephant.position.0).unwrap();
            right_options.push(Goal::Idle);
            right_options.extend(
                self.world
                    .closed_valves(cave_system)
                    .filter(|cave| {
                        let effect_time = ele_cave.paths.get(cave.0).unwrap() + 1;
                        self.world.minutes + effect_time <= max_cave_time
                    })
                    .map(|cave| {
                        let effect_time = ele_cave.paths.get(cave.0).unwrap() + 1;
                        let rate = cave_system.caves.get(cave.0).unwrap().flow_rate;
                        Goal::MoveTo(*cave, self.world.minutes + effect_time, rate)
                    }),
            );

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

        self.world.pressure_at_time(max_cave_time)

        // if self.me.goal.is_none() {
        //     let mut p = self.clone();
        //     p.me.set_idling();
        //     queue.push(p);

        //     self.me.append_futures(
        //         cave_system,
        //         queue,
        //         self,
        //         max_cave_time,
        //         false,
        //         self.world.minutes,
        //     );
        // }

        // if self.elephant.goal.is_none() {
        //     let mut p = self.clone();
        //     p.elephant.set_idling();
        //     queue.push(p);

        //     self.elephant.append_futures(
        //         cave_system,
        //         queue,
        //         self,
        //         max_cave_time,
        //         true,
        //         self.world.minutes,
        //     );
        // }

        // if self.elephant.goal.is_none() && self.me.goal.is_none() {
        //     self.reassign_both(cave_system, queue, max_cave_time);
        // }

        // panic!("Shouldn't happen");
    }

    // fn reassign_both(&self, cave_system: &CaveSystem, queue: &mut Vec<Path>, max_cave_time: u32) {
    //     // println!("Reassign-both");
    //     let me_cave = cave_system.caves.get(self.me.position.0).unwrap();
    //     let elephant_cave = cave_system.caves.get(self.elephant.position.0).unwrap();

    //     let me_option_count = cave_system
    //         .caves_with_working_valve
    //         .iter()
    //         .filter(|cave_id| {
    //             !self.world.is_valve_open(**cave_id)
    //                 && !(self.me.position == **cave_id)
    //                 && self.world.minutes + me_cave.paths.get(cave_id.0).unwrap() < max_cave_time
    //         })
    //         .count();
    //     let ele_option_count = cave_system
    //         .caves_with_working_valve
    //         .iter()
    //         .filter(|cave_id| {
    //             !self.world.is_valve_open(**cave_id)
    //                 && !(self.elephant.position == **cave_id)
    //                 && self.world.minutes + elephant_cave.paths.get(cave_id.0).unwrap()
    //                     < max_cave_time
    //         })
    //         .count();

    //     if me_option_count == 0 && ele_option_count == 0 {
    //         let mut p = self.clone();
    //         p.me.set_idling();
    //         p.elephant.set_idling();
    //         queue.push(p);
    //         return;
    //     }

    //     if me_option_count > 0 && ele_option_count > 0 {
    //         cave_system
    //             .caves_with_working_valve
    //             .iter()
    //             .filter(|cave_id| {
    //                 !self.world.is_valve_open(**cave_id)
    //                     && **cave_id != self.me.position
    //                     && self.world.minutes + me_cave.paths.get(cave_id.0).unwrap()
    //                         < max_cave_time
    //             })
    //             .for_each(|me_cave_target_id| {
    //                 let me_travel_time = me_cave.paths.get(me_cave_target_id.0).unwrap();

    //                 cave_system
    //                     .caves_with_working_valve
    //                     .iter()
    //                     .filter(|cave_id| {
    //                         !self.world.is_valve_open(**cave_id)
    //                             && **cave_id != self.elephant.position
    //                             && self.world.minutes + elephant_cave.paths.get(cave_id.0).unwrap()
    //                                 < max_cave_time
    //                     })
    //                     .for_each(|elephant_cave_target_id| {
    //                         let elephant_travel_time =
    //                             elephant_cave.paths.get(elephant_cave_target_id.0).unwrap();

    //                         let mut p = self.clone();
    //                         p.me.goal = Some(Goal::MoveTo(*me_cave_target_id));
    //                         p.me.goal_time = self.world.minutes + me_travel_time;
    //                         p.elephant.goal = Some(Goal::MoveTo(*elephant_cave_target_id));
    //                         p.elephant.goal_time = self.world.minutes + elephant_travel_time;
    //                         queue.push(p);
    //                     });
    //             });
    //         return;
    //     }

    //     if me_option_count > 0 {
    //         cave_system
    //             .caves_with_working_valve
    //             .iter()
    //             .filter(|cave_id| {
    //                 !self.world.is_valve_open(**cave_id)
    //                     && **cave_id != self.me.position
    //                     && self.world.minutes + me_cave.paths.get(cave_id.0).unwrap()
    //                         < max_cave_time
    //             })
    //             .for_each(|me_cave_target_id| {
    //                 let me_travel_time = me_cave.paths.get(me_cave_target_id.0).unwrap();
    //                 let mut p = self.clone();
    //                 p.me.goal = Some(Goal::MoveTo(*me_cave_target_id));
    //                 p.me.goal_time = self.world.minutes + me_travel_time;
    //                 p.elephant.set_idling();
    //                 queue.push(p);
    //             });
    //         // let p = self.clone();
    //         return;
    //     }

    //     if ele_option_count > 0 {
    //         // Every variant of elephant with me idling
    //         cave_system
    //             .caves_with_working_valve
    //             .iter()
    //             .filter(|cave_id| {
    //                 !self.world.is_valve_open(**cave_id)
    //                     && **cave_id != self.elephant.position
    //                     && self.world.minutes + elephant_cave.paths.get(cave_id.0).unwrap()
    //                         < max_cave_time
    //             })
    //             .for_each(|elephant_cave_target_id| {
    //                 let elephant_travel_time =
    //                     elephant_cave.paths.get(elephant_cave_target_id.0).unwrap();

    //                 let mut p = self.clone();
    //                 p.me.set_idling();
    //                 p.elephant.goal = Some(Goal::MoveTo(*elephant_cave_target_id));
    //                 p.elephant.goal_time = self.world.minutes + elephant_travel_time;
    //                 queue.push(p);
    //             });
    //         // let p = self.clone();
    //         return;
    //     }

    //     unreachable!();

    //     cave_system
    //         .caves_with_working_valve
    //         .iter()
    //         .filter(|cave_id| {
    //             !self.world.is_valve_open(**cave_id)
    //                 && **cave_id != self.me.position
    //                 && self.world.minutes + me_cave.paths.get(cave_id.0).unwrap() < max_cave_time
    //         })
    //         .for_each(|me_cave_target_id| {
    //             // let p = self.clone();

    //             let me_travel_time = me_cave.paths.get(me_cave_target_id.0).unwrap();
    //             // let set_me_goal = self.world.minutes + me_travel_time < max_cave_time
    //             // && !self.world.is_valve_open(*me_cave_target_id);

    //             // Every combination of non-idle goals
    //             cave_system
    //                 .caves_with_working_valve
    //                 .iter()
    //                 .filter(|cave_id| {
    //                     !self.world.is_valve_open(**cave_id)
    //                         && **cave_id != self.elephant.position
    //                         && self.world.minutes + elephant_cave.paths.get(cave_id.0).unwrap()
    //                             < max_cave_time
    //                 })
    //                 .for_each(|elephant_cave_target_id| {
    //                     let elephant_travel_time =
    //                         elephant_cave.paths.get(elephant_cave_target_id.0).unwrap();

    //                     let mut p = self.clone();
    //                     p.me.goal = Some(Goal::MoveTo(*me_cave_target_id));
    //                     p.me.goal_time = sself.world.minutes + me_travel_time;
    //                     p.elephant.goal = Some(Goal::MoveTo(*elephant_cave_target_id));
    //                     p.elephant.goal_time = self.world.minutes + elephant_travel_time;
    //                     queue.push(p);
    //                 });

    //             // Every combination with elephant idling
    //             let mut p = self.clone();
    //             p.me.goal = Some(Goal::MoveTo(*me_cave_target_id));
    //             p.me.goal_time = self.world.minutes + me_travel_time;
    //             p.elephant.set_idling();

    //             queue.push(p);

    //             // let mut p = self.clone();
    //             // p.me.goal = Some(Goal::MoveTo(*me_cave_target_id));
    //             // p.me.goal_time = self.world.minutes + me_travel_time;
    //             // p.elephant.set_idling();
    //             // queue.push(p);

    //             // .for_each(|elephant_cave_target_id| {
    //             //     if *me_cave_target_id == *elephant_cave_target_id {
    //             //         // return; // Do not allow setting the same goal on both travelers
    //             //     }

    //             //     let elephant_travel_time =
    //             //         elephant_cave.paths.get(elephant_cave_target_id.0).unwrap();

    //             //     let set_elephant_goal = self.world.minutes + elephant_travel_time
    //             //         < max_cave_time
    //             //         && !self.world.is_valve_open(*elephant_cave_target_id);

    //             //     let mut p = self.clone();

    //             //     if set_me_goal {
    //             //         p.me.goal = Some(Goal::MoveTo(*me_cave_target_id));
    //             //         p.me.goal_time = p.world.minutes + me_travel_time;
    //             //     } else {
    //             //         p.me.set_idling();
    //             //     }

    //             //     if set_elephant_goal {
    //             //         p.elephant.goal = Some(Goal::MoveTo(*elephant_cave_target_id));
    //             //         p.elephant.goal_time = p.world.minutes + elephant_travel_time;
    //             //     } else {
    //             //         p.elephant.set_idling();
    //             //     }
    //             //     if set_me_goal || set_elephant_goal {
    //             //         queue.push(p);
    //             //     }
    //             // });
    //         });
    // }

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
    let mut iter = 0;

    let mut left = vec![];
    let mut right = vec![];

    while let Some(mut path) = queue.pop() {
        // path.world.advance_time_to(path.next_action_time());
        // biggest_release = biggest_release.max(path.world.pressure_at_time(30));
        path.world.advance_time_to(path.next_action_time(30));
        // biggest_release = pressure.max(biggest_release);
        let pressure = path.futures(cave_system, &mut queue, 30, &mut left, &mut right);
        biggest_release = pressure.max(biggest_release);
        // biggest_release = biggest_release.max(path.world.pressure_at_time(30));
        iter += 1;
        if iter == 10_000 {
            println!("{}", queue.len());
            iter = 0;
        }
    }

    biggest_release
}

fn find_biggest_release_with_elephant(cave_system: &CaveSystem) -> u32 {
    let start_cave_id = cave_system
        .cave_by_name(START_CAVE)
        .expect("start cave should be present in cave_system");

    println!("Part 2 start");

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

    // let path = queue.remove(0);
    // path.futures(cave_system, &mut queue, 26, &mut left, &mut right);

    let mut biggest_release: u32 = 0;
    let mut iter = 0;

    while let Some(mut path) = queue.pop() {
        // path.resolve_actions(cave_system, 26);
        // biggest_release = biggest_release.max(path.world.pressure_at_time(26));
        path.world.advance_time_to(path.next_action_time(26));
        let pressure = path.futures(cave_system, &mut queue, 26, &mut left, &mut right);
        biggest_release = biggest_release.max(pressure);
        iter += 1;
        if iter == 20_000000 {
            println!("{}", queue.len());
            iter = 0;
        }
    }

    biggest_release
}

// https://adventofcode.com/2022/day/16
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let caves = CaveSystem::from_str(input);

    println!("{}", caves);
    let pressure = find_biggest_release(&caves);
    let p2 = find_biggest_release_with_elephant(&caves);

    // println!("{},{}", pressure, p2);
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
