use std::{
    cell::{Cell, OnceCell, RefCell},
    collections::HashMap,
    rc::Rc,
    str::FromStr,
};

use super::{DayOutput, LogicError, PartResult};

enum Node {
    File {
        size: i32,
    },
    Folder {
        size: OnceCell<i32>,
        children: RefCell<HashMap<String, NodeRef>>,
    },
}

struct NodeRef(Rc<Node>);

impl FromStr for NodeRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let node = Node::Folder {
            size: OnceCell::new(),
            children: RefCell::new(HashMap::new()),
        };

        let mut dirs = vec![Self::new(node)];

        s.lines().map(str::parse::<Line>).for_each(|entry| {
            let cmd = entry.expect("Succesfull parse");
            match cmd {
                Line::Command(cmd) => match cmd {
                    Command::ChRoot => {
                        dirs.drain(1..);
                    }
                    Command::ChUp => {
                        dirs.pop();
                    }
                    Command::ChDir(dir_name) => {
                        let child = dirs
                            .last()
                            .expect("Dirs to contain an item")
                            .get_child(dir_name);
                        dirs.push(child);
                    }
                    Command::Ls => (),
                },
                Line::DirEntry(dir_entry) => match dir_entry {
                    DirEntry::File(name, size) => {
                        dirs.last().expect("Dirs to contain an item").add_child(
                            name,
                            Some(size),
                            false,
                        );
                    }
                    DirEntry::Dir(name) => {
                        dirs.last()
                            .expect("Dirs to contain an item")
                            .add_child(name, None, true);
                    }
                },
            }
        });

        Ok(dirs.remove(0))
    }
}

impl NodeRef {
    fn new(n: Node) -> Self {
        Self(Rc::new(n))
    }
}

impl Clone for NodeRef {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl NodeRef {
    fn add_child(&self, path: impl Into<String>, size: Option<i32>, is_dir: bool) {
        let child: Node = if is_dir {
            Node::Folder {
                size: OnceCell::new(),
                children: RefCell::new(HashMap::new()),
            }
        } else {
            Node::File {
                size: size.expect("File must have size provided"),
            }
        };

        match self.0.as_ref() {
            Node::File { .. } => panic!("Cannot add child to a file"),
            Node::Folder { children, .. } => {
                children.borrow_mut().insert(path.into(), Self::new(child));
            }
        }
    }

    // Get own size or resursively get (and cache) children's size
    fn calc_size(&self) -> i32 {
        match self.0.as_ref() {
            Node::File { size, .. } => *size,
            Node::Folder { size, children, .. } => *size.get_or_init(|| {
                children
                    .borrow()
                    .iter()
                    .map(|(_, noderef)| noderef.calc_size())
                    .sum()
            }),
        }
    }

    fn get_child(&self, path: impl Into<String>) -> Self {
        match self.0.as_ref() {
            Node::File { .. } => panic!("File doesn't have children"),
            Node::Folder { children, .. } => children
                .borrow()
                .get(&path.into())
                .expect("map to contain given child")
                .clone(),
        }
    }
}

enum Command {
    ChRoot,
    ChUp,
    ChDir(String),
    Ls,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "$ cd /" => Self::ChRoot,

            "$ ls" => Self::Ls,
            "$ cd .." => Self::ChUp,
            _ => {
                let (_, dirname) = s.split_at(5);
                Self::ChDir(dirname.into())
            }
        })
    }
}

enum DirEntry {
    File(String, i32),
    Dir(String),
}

impl FromStr for DirEntry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s.split_once(' ').expect("line to split into two");

        if left == "dir" {
            Ok(Self::Dir(right.into()))
        } else {
            let size: i32 = left.parse().expect("left side to parse into int");
            Ok(Self::File(right.into(), size))
        }
    }
}

enum Line {
    Command(Command),
    DirEntry(DirEntry),
}

impl FromStr for Line {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.as_bytes()[0] == b'$' {
            Ok(Self::Command(s.parse::<Command>()?))
        } else {
            Ok(Self::DirEntry(s.parse::<DirEntry>()?))
        }
    }
}

// https://adventofcode.com/2022/day/7
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let fs: NodeRef = input.parse().expect("Succesfull parse");
    let total_size = fs.calc_size();

    let countcell = Cell::new(0);
    sum_size(&fs, &countcell);

    let del_size = find_dir_to_delete(&fs, total_size);

    Ok(DayOutput {
        part1: Some(PartResult::Int(countcell.get())),
        part2: Some(PartResult::Int(del_size)),
    })
}

fn sum_size(fs: &NodeRef, count: &Cell<i32>) {
    match fs.0.as_ref() {
        Node::File { .. } => (),
        Node::Folder { size, children, .. } => {
            let size = *size.get().expect("Size should be known at this point, if not NodeRef::calc_size should have been called first");

            if size <= 100_000 {
                count.set(count.get() + size);
            };
            children
                .borrow()
                .iter()
                .for_each(|(_, noderef)| sum_size(noderef, count));
        }
    }
}

fn collect_fs_to_vec(fs: &NodeRef, v: &mut Vec<i32>) {
    match fs.0.as_ref() {
        Node::File { .. } => (),
        Node::Folder { size, children, .. } => {
            v.push(*size.get().expect("size to exist"));
            children
                .borrow()
                .iter()
                .for_each(|f| collect_fs_to_vec(f.1, v));
        }
    }
}

fn find_dir_to_delete(fs: &NodeRef, occupied_space: i32) -> i32 {
    let storage_size = 70_000_000;
    let current_free_space = storage_size - occupied_space;
    let min_space_to_free = 30_000_000 - current_free_space;

    let mut dirs = vec![];

    collect_fs_to_vec(fs, &mut dirs);

    dirs.sort_unstable();

    *dirs
        .iter()
        .find(|i| **i > min_space_to_free)
        .expect("find to succeed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(7, super::solve)
    }

    #[test]
    fn example() -> Result<(), String> {
        let input: String = vec![
            "$ cd /",
            "$ ls",
            "dir a",
            "14848514 b.txt",
            "8504156 c.dat",
            "dir d",
            "$ cd a",
            "$ ls",
            "dir e",
            "29116 f",
            "2557 g",
            "62596 h.lst",
            "$ cd e",
            "$ ls",
            "584 i",
            "$ cd ..",
            "$ cd ..",
            "$ cd d",
            "$ ls",
            "4060174 j",
            "8033020 d.log",
            "5626152 d.ext",
            "7214296 k",
        ]
        .join("\n");

        let fs: NodeRef = input.parse().expect("Succesfull parse");
        let size = fs.calc_size();

        assert_eq!(size, 48_381_165);

        let countcell = Cell::new(0);
        sum_size(&fs, &countcell);

        assert_eq!(countcell.get(), 95437);

        Ok(())
    }
}
