use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use super::{DayOutput, LogicError, PartResult};

struct Node {
    children: RefCell<HashMap<String, NodeRef>>,
    size: Cell<Option<i32>>,
    parent: Option<NodeRef>,
    is_dir: bool,
}

struct NodeRef(Rc<Node>);

impl Clone for NodeRef {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl NodeRef {
    fn add_child(&self, path: impl Into<String>, size: Option<i32>, is_dir: bool) {
        let self_cloned = Rc::clone(&self.0);
        let child = Node::new(size, Some(Self(Rc::clone(&self_cloned))), is_dir);

        self.0
            .children
            .borrow_mut()
            .insert(path.into(), Self(Rc::new(child)));
    }

    fn calc_size(&self) -> i32 {
        if let Some(size) = self.0.size.get() {
            return size;
        }

        let size: i32 = self
            .0
            .children
            .borrow()
            .iter()
            .map(|f| f.1.calc_size())
            .sum();

        self.0.size.set(Some(size));

        size
    }
}

impl Node {
    fn new(size: Option<i32>, parent: Option<NodeRef>, is_dir: bool) -> Self {
        Self {
            children: RefCell::new(HashMap::new()),
            size: Cell::new(size),
            parent,
            is_dir,
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
    fs.calc_size();

    let countcell = Cell::new(0);
    sum_size(&fs, &countcell);

    let del_size = find_dir_to_delete(&fs);

    Ok(DayOutput {
        part1: Some(PartResult::Int(countcell.get())),
        part2: Some(PartResult::Int(del_size)),
    })
}

fn sum_size(fs: &NodeRef, count: &Cell<i32>) {
    if !fs.0.is_dir {
        return;
    };

    fs.0.size.get().and_then(|f: i32| {
        if f <= 100_000 {
            count.set(count.get() + f);
        };

        None::<Option<i32>>
    });

    for entry in &*fs.0.children.borrow() {
        sum_size(entry.1, count);
    }
}

fn collect_fs_to_vec(fs: &NodeRef, v: &mut Vec<i32>) {
    if !fs.0.is_dir {
        return;
    }

    v.push(fs.0.size.get().expect("size to be Some"));

    for pair in &*fs.0.children.borrow() {
        collect_fs_to_vec(pair.1, v);
    }
}

fn find_dir_to_delete(fs: &NodeRef) -> i32 {
    let occupied_space = fs.0.size.get().expect("node to have size");
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
    let root: NodeRef = NodeRef(Rc::new(Node::new(None, None, true)));

    let mut current_node = NodeRef(Rc::clone(&root.0));

    input.lines().for_each(|f| {
        if f.as_bytes()[0] == b'$' {
            // println!("is cmd");
            match f {
                "$ cd /" => current_node = NodeRef(Rc::clone(&root.0)),
                "$ ls" => (),
                "$ cd .." => {
                    let parent_node = current_node
                        .0
                        .parent
                        .clone()
                        .expect("node to have a parent");
                    current_node = parent_node;
                }
                _ => {
                    // cd $dirname
                    let (_, dirname) = f.split_at(5);

                    current_node = current_node
                        .clone()
                        .0
                        .children
                        .borrow()
                        .get(dirname)
                        .expect("dir to have child")
                        .clone();
                }
            }
        } else {
            let (left, right) = f.split_once(' ').expect("line to split into two");

            if left == "dir" {
                current_node.add_child(right, None, true);
            } else {
                let size: i32 = left.parse().expect("left side to parse into int");
                current_node.add_child(right, Some(size), false);
            }
        }
    });

    root
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
        fs.calc_size();

        assert_eq!(fs.0.size.get().unwrap_or_default(), 48381165);

        let countcell = Cell::new(0);
        sum_size(&fs, &countcell);

        assert_eq!(countcell.get(), 95437);

        Ok(())
    }
}
