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

type NodeRef = Rc<Node>;

fn set_dir_sizes(node: &NodeRef) -> i32 {
    if let Some(size) = node.size.get() {
        return size;
    }

    let size: i32 = node
        .children
        .borrow()
        .iter()
        .map(|f| set_dir_sizes(f.1))
        .sum();

    node.size.set(Some(size));

    size
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

    fn calc_size(&self) -> i32 {
        {
            if let Some(size) = self.size.get() {
                return size;
            }
        }

        // let size = self.size.borrow_mut().unwrap();
        let size: i32 = {
            self.children
                .borrow_mut()
                .iter()
                .map(|child| child.1.calc_size())
                .sum()
        };

        self.size.set(Some(size));

        size
    }

    fn add_child(&mut self, n: Rc<Self>, path: impl Into<String>) {
        self.children.borrow_mut().insert(path.into(), n);
    }
}

enum Command {
    ChRoot,
    ChUp,
    ChDir,
    Ls,
}

fn printnode(node: &NodeRef, indent: usize) {
    let left_space = " ".repeat(indent * 4);
    let size = node.size.get().unwrap_or(-1);

    println!("{left_space}, {size}");

    for child in &*node.children.borrow() {
        printnode(child.1, indent + 1);
    }
}

// https://adventofcode.com/2022/day/7
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let fs = fun(input);
    set_dir_sizes(&fs);

    let countcell = Cell::new(0);
    sum_size(&fs, &countcell);

    // let mut fs = HashMap::new();

    // let root = VecNode {
    //     size: None,
    //     is_dir: true,
    //     children: HashMap::new(),
    //     parent: None,
    // };

    // fun(input);

    // printnode(&fs, 0);

    let del_size = find_dir_to_delete(&fs);

    Ok(DayOutput {
        part1: Some(PartResult::Int(countcell.get())),
        part2: Some(PartResult::Int(del_size)),
    })
}

fn sum_size(fs: &NodeRef, count: &Cell<i32>) {
    if !fs.is_dir {
        return;
    };

    fs.size.get().and_then(|f: i32| {
        if f <= 100_000 {
            count.set(count.get() + f);
        };

        None::<Option<i32>>
    });

    for entry in &*fs.children.borrow() {
        sum_size(entry.1, count);
    }
}

fn collect_fs_to_vec(fs: &NodeRef, v: &mut Vec<i32>) {
    if !fs.is_dir {
        return;
    }

    v.push(fs.size.get().expect("size to be Some"));

    for pair in &*fs.children.borrow() {
        collect_fs_to_vec(pair.1, v);
    }
}

fn find_dir_to_delete(fs: &NodeRef) -> i32 {
    let occupied_space = fs.size.get().expect("node to have size");
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
    let root: NodeRef = Rc::new(Node::new(None, None, true));

    let mut current_node = Rc::clone(&root);

    input.lines().for_each(|f| {
        if f.as_bytes()[0] == b'$' {
            // println!("is cmd");
            match f {
                "$ cd /" => current_node = Rc::clone(&root),
                "$ ls" => (),
                "$ cd .." => {
                    current_node =
                        Rc::clone(&current_node.parent.clone().expect("node to have a parent"));
                }
                _ => {
                    // cd $dirname
                    let (_, dirname) = f.split_at(5);
                    let c2: NodeRef;
                    {
                        let child = current_node.children.borrow();
                        c2 = Rc::clone(child.get(dirname).expect("dir to contain child"));
                    }

                    current_node = c2;
                }
            }
        } else {
            let (left, right) = f.split_once(' ').expect("line to split into two");

            if left == "dir" {
                // Directory
                let newnode = Node::new(None, Some(Rc::clone(&current_node)), true);

                current_node
                    .children
                    .borrow_mut()
                    .insert(right.to_owned(), Rc::new(newnode));
            } else {
                //File
                let size: i32 = left.parse().expect("left side to parse into int");
                let newnode = Node::new(Some(size), None, false);

                current_node
                    .children
                    .borrow_mut()
                    .insert(right.to_owned(), Rc::new(newnode));
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
        set_dir_sizes(&fs);

        assert_eq!(fs.size.get().unwrap_or_default(), 48381165);

        set_dir_sizes(&fs);

        let countcell = Cell::new(0);
        sum_size(&fs, &countcell);

        assert_eq!(countcell.get(), 95437);

        Ok(())
    }
}
