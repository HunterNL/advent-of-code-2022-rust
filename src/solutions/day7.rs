use std::{
    cell::{Cell, OnceCell},
    collections::HashMap,
    str::FromStr,
};

use super::{DayOutput, LogicError, PartResult};

enum Node {
    File {
        size: i32,
    },
    Folder {
        size: OnceCell<i32>,
        children: HashMap<String, Node>,
    },
}
// Pops a directory from the end of the vector and move it into the new last entry in the vector
fn pop_and_restore_dir(dirs: &mut Vec<(String, Node)>) {
    let entry = dirs.pop().expect("dirs to have an entry");
    dirs.last_mut()
        .expect("dirs to have a last")
        .1
        .add_child(entry.0, entry.1);
}

impl FromStr for Node {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let root = Self::Folder {
            size: OnceCell::new(),
            children: HashMap::new(),
        };

        // Vector of all opened folders
        // "Open" folders are removed from their original parent and put here while accessed
        // It is important to "close" and return folders back to their original parent when popped
        // Use `pop_and_restore_dir` for this
        let mut dirs: Vec<(String, Self)> = vec![(String::new(), root)];

        s.lines().map(str::parse::<Line>).for_each(|entry| {
            let cmd = entry.expect("Succesfull parse");
            match cmd {
                Line::Command(cmd) => match cmd {
                    Command::ChRoot => {
                        while dirs.len() > 1 {
                            pop_and_restore_dir(&mut dirs);
                        }
                    }
                    Command::ChUp => pop_and_restore_dir(&mut dirs),
                    Command::ChDir(dir_name) => {
                        let child = dirs
                            .last_mut()
                            .expect("Dirs to contain an item")
                            .1
                            .remove_child(dir_name);
                        dirs.push(child);
                    }
                    Command::Ls => (),
                },
                Line::DirEntry(dir_entry) => match dir_entry {
                    DirEntry::File(name, size) => dirs
                        .last_mut()
                        .expect("Dirs to contain an item")
                        .1
                        .add_child(name, Self::new(&NodeKind::File, size)),
                    DirEntry::Dir(name) => {
                        dirs.last_mut()
                            .expect("Dirs to contain an item")
                            .1
                            .add_child(name, Self::new(&NodeKind::Folder, 0));
                    }
                },
            }
        });

        // Important to ensure all opened dirs are back into their proper place
        while dirs.len() > 1 {
            pop_and_restore_dir(&mut dirs);
        }

        Ok(dirs.remove(0).1)
    }
}

enum NodeKind {
    File,
    Folder,
}

impl Node {
    fn new(kind: &NodeKind, size: i32) -> Self {
        match kind {
            NodeKind::File => Self::File { size },
            NodeKind::Folder => Self::Folder {
                size: OnceCell::new(), // Note ignoring the argument, unlike files, folder size is not known at creation. calc_size can figure that out
                children: HashMap::new(),
            },
        }
    }

    fn add_child(&mut self, path: impl Into<String>, n: Self) {
        match self {
            Self::File { .. } => panic!("Cannot add child to a file"),
            Self::Folder { children, .. } => {
                children.insert(path.into(), n);
            }
        }
    }

    // Get own size or resursively get (and cache) children's size
    fn calc_size(&self) -> i32 {
        match self {
            Self::File { size, .. } => *size,
            Self::Folder { size, children, .. } => *size.get_or_init(|| {
                children
                    .iter()
                    .map(|(_, noderef)| noderef.calc_size())
                    .sum()
            }),
        }
    }

    fn remove_child(&mut self, path: impl Into<String>) -> (String, Self) {
        match self {
            Self::File { .. } => panic!("File doesn't have children"),
            Self::Folder { children, .. } => children
                .remove_entry(&path.into())
                .expect("map to contain given child"),
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
    let fs: Node = input.parse().expect("Succesfull parse");
    let total_size = fs.calc_size();

    let countcell = Cell::new(0);
    sum_size(&fs, &countcell);

    let del_size = find_dir_to_delete(&fs, total_size);

    Ok(DayOutput {
        part1: Some(PartResult::Int(countcell.get())),
        part2: Some(PartResult::Int(del_size)),
    })
}

fn sum_size(fs: &Node, count: &Cell<i32>) {
    match fs {
        Node::File { .. } => (),
        Node::Folder { size, children, .. } => {
            let size = *size.get().expect("Size should be known at this point, if not NodeRef::calc_size should have been called first");

            if size <= 100_000 {
                count.set(count.get() + size);
            };
            children
                .iter()
                .for_each(|(_, noderef)| sum_size(noderef, count));
        }
    }
}

fn collect_fs_to_vec(fs: &Node, v: &mut Vec<i32>) {
    match fs {
        Node::File { .. } => (),
        Node::Folder { size, children, .. } => {
            v.push(*size.get().expect("size to exist"));
            children.iter().for_each(|f| collect_fs_to_vec(f.1, v));
        }
    }
}

fn find_dir_to_delete(fs: &Node, occupied_space: i32) -> i32 {
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

        let fs: Node = input.parse().expect("Succesfull parse");
        let size = fs.calc_size();

        assert_eq!(size, 48_381_165);

        let countcell = Cell::new(0);
        sum_size(&fs, &countcell);

        assert_eq!(countcell.get(), 95437);

        Ok(())
    }
}
