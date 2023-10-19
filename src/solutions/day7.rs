use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use super::{DayOutput, LogicError, PartResult};

enum Node {
    File {
        size: i32,
    },
    Folder {
        size: Cell<Option<i32>>,
        children: RefCell<HashMap<String, NodeRef>>,
    },
}

struct NodeRef(Rc<Node>);

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
                size: Cell::new(None),
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
                children
                    .borrow_mut()
                    .insert(path.into(), NodeRef::new(child));
            }
        }
    }

    // Get own size or resursively get (and cache) children's size
    fn calc_size(&self) -> i32 {
        match self.0.as_ref() {
            Node::File { size, .. } => *size,
            Node::Folder { size, children, .. } => match size.get() {
                Some(a) => a,
                None => {
                    let newsize = children.borrow().iter().map(|f| f.1.calc_size()).sum();
                    size.set(Some(newsize)); // Bit aw
                    newsize
                }
            },
        }
    }

    fn get_child(&self, path: impl Into<String>) -> NodeRef {
        match self.0.as_ref() {
            Node::File { .. } => panic!("File doesn't have children"),
            Node::Folder { children, .. } => children.borrow().get(&path.into()).unwrap().clone(),
        }
    }
}

enum Command {
    ChRoot,
    ChUp,
    ChDir,
    Ls,
}

// https://adventofcode.com/2022/day/7
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let fs = fun(input);
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
        Node::Folder { size, children, .. } => match (size).get() {
            Some(c) => {
                if c <= 100_000 {
                    count.set(count.get() + c);
                };
                children.borrow().iter().for_each(|f| sum_size(f.1, count));
            }
            None => panic!("Size should be known by now"),
        },
    }
}

fn collect_fs_to_vec(fs: &NodeRef, v: &mut Vec<i32>) {
    match fs.0.as_ref() {
        Node::File { .. } => (),
        Node::Folder { size, children, .. } => {
            v.push(size.get().expect("size to exist"));
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

fn fun(input: &str) -> NodeRef {
    let node = Node::Folder {
        size: Cell::new(None),
        children: RefCell::new(HashMap::new()),
    };

    let mut dirs = vec![NodeRef::new(node)];

    input.lines().for_each(|f| {
        if f.as_bytes()[0] == b'$' {
            // println!("is cmd");
            match f {
                "$ cd /" => {
                    dirs.drain(1..);
                }
                "$ ls" => (),
                "$ cd .." => {
                    dirs.pop();
                }
                _ => {
                    let (_, dirname) = f.split_at(5);
                    let child = dirs.last().unwrap().get_child(dirname).clone();
                    dirs.push(child);
                }
            }
        } else {
            let (left, right) = f.split_once(' ').expect("line to split into two");

            if left == "dir" {
                dirs.last().unwrap().add_child(right, None, true);
            } else {
                let size: i32 = left.parse().expect("left side to parse into int");
                dirs.last().unwrap().add_child(right, Some(size), false);
            }
        }
    });

    dirs.remove(0)
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

        let fs = fun(input.as_ref());
        let size = fs.calc_size();

        assert_eq!(size, 48_381_165);

        let countcell = Cell::new(0);
        sum_size(&fs, &countcell);

        assert_eq!(countcell.get(), 95437);

        Ok(())
    }
}
