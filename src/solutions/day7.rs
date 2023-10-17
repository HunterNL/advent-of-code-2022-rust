use core::fmt;
use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{self, Rc},
};

use super::{DayOutput, LogicError, PartResult};

enum FolderItem {
    Folder,
    File(String, i32),
}

#[derive(Debug)]
struct FsNode {
    items: RefCell<HashMap<String, FsNode>>,
    size: Cell<Option<i32>>,
    is_dir: bool,
}

impl FsNode {
    fn new(is_dir: bool, size: Option<i32>) -> Self {
        Self {
            items: RefCell::new(HashMap::new()),
            size: Cell::new(size),
            is_dir,
        }
    }

    // fn add_sub_item<'b>(&mut self, n: FsNode, path: &str) {
    //     self.items.insert(path.to_owned(), n);
    // }
}

struct VecGraph {
    vec: Vec<VecNode>,
}

struct Node {
    children: RefCell<HashMap<String, NodeRef>>,
    size: Cell<Option<i32>>,
    parent: Option<NodeRef>,
    is_dir: bool,
}

type NodeRef = Rc<Node>;

fn set_dir_sizes(node: NodeRef) -> i32 {
    if node.size.get().is_some() {
        return node.size.get().unwrap();
    }

    let size: i32 = node
        .children
        .borrow()
        .iter()
        .map(|f| set_dir_sizes(Rc::clone(f.1)))
        .sum();

    node.size.set(Some(size));

    size
}

impl Node {
    fn new(size: Option<i32>, parent: Option<NodeRef>, isDir: bool) -> Self {
        Self {
            children: RefCell::new(HashMap::new()),
            size: Cell::new(size),
            parent,
            is_dir: isDir,
        }
    }

    fn calc_size(&self) -> i32 {
        {
            if self.size.get().is_some() {
                return self.size.get().unwrap();
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

    fn add_child(&mut self, n: Rc<Node>, path: impl Into<String>) {
        self.children.borrow_mut().insert(path.into(), n);
    }
}

fn get_and_calc(vg: &mut VecGraph, index: usize) -> i32 {
    todo!();
    let child = vg.vec.get_mut(index).unwrap();
    todo!();

    // calc_sizes(vg, child);
    child.size.unwrap()
}

// fn calc_sizes(vg: &mut VecGraph, node: &mut VecNode) {
//     todo!();
//     if node.size.is_some() {
//         return;
//     }

//     let size: i32 = node
//         .children
//         .iter()
//         .map(|kv| kv.1)
//         .map(|index| {
//             let child = vg.vec.get_mut(*index).unwrap();
//             let c1 = vg.vec.get_mut(*index).unwrap();

//             // println!("{:?}{:?}", c0, c1);
//             calc_sizes(vg, child);
//             return node.size.expect("valid size");
//         })
//         .sum();

//     node.size.replace(size);

//     // self.size.replace(
//     //     self.children
//     //         .iter()
//     //         .map(|key_value| {
//     //             let child_index = *key_value.1;
//     //             let child: &mut VecNode;
//     //             {
//     //                 child = (v).get_mut(child_index).unwrap();
//     //             }
//     //             child.calc_size(v)
//     //         })
//     //         .sum(),
//     // );
// }

struct VecNode {
    size: Option<i32>,
    is_dir: bool,
    children: HashMap<String, usize>,
    parent: Option<usize>,
}

// impl VecNode {
//     fn new(size: Option<i32>, is_dir: bool, parent: usize) -> Self {
//         Self {
//             size,
//             is_dir,
//             children: HashMap::new(),
//             parent: Some(parent),
//         }
//     }

//     fn calc_size<'a>(mut self, v: &'a mut &Vec<Self>) -> i32 {
//         if self.size.is_some() {
//             return self.size.unwrap();
//         }

//         self.size.replace(
//             self.children
//                 .iter()
//                 .map(|key_value| {
//                     let child_index = *key_value.1;
//                     let mut child: &mut VecNode;
//                     {
//                         child = (v).get_mut(child_index).unwrap();
//                     }
//                     child.calc_size(v)
//                 })
//                 .sum(),
//         );

//         self.size.unwrap()
//     }
// }

enum Command {
    ChRoot,
    ChUp,
    ChDir,
    Ls,
}
fn reset_stack(s: &mut Vec<&mut FsNode>) {
    while s.len() > 1 {
        s.pop();
    }
    println!("path is now {:?}", s)
}

fn add_vec_node(v: &mut Vec<VecNode>, n: VecNode, path: impl Into<String>) {
    let parent_index = n.parent.unwrap();
    v.push(n);

    let new_index = v.len();

    v.get_mut(parent_index)
        .unwrap()
        .children
        .insert(path.into(), new_index - 1);
}

fn printnode(node: NodeRef, indent: usize) {
    let leftSpace = " ".repeat(indent * 4);
    let size = node.size.get().unwrap_or(-1);

    println!("{leftSpace}, {size}");

    for child in node.children.borrow().iter() {
        printnode(Rc::clone(child.1), indent + 1)
    }
}

// https://adventofcode.com/2022/day/7
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let fs = fun(&input);
    set_dir_sizes(Rc::clone(&fs));

    let countcell = Cell::new(0);
    sum_size(Rc::clone(&fs), &countcell);

    // let mut fs = HashMap::new();

    // let root = VecNode {
    //     size: None,
    //     is_dir: true,
    //     children: HashMap::new(),
    //     parent: None,
    // };

    // fun(input);

    printnode(Rc::clone(&fs), 0);

    let del_size = find_dir_to_delete(Rc::clone(&fs));

    Ok(DayOutput {
        part1: Some(PartResult::Int(countcell.get())),
        part2: Some(PartResult::Int(del_size)),
    })
}

fn sum_size(fs: Rc<Node>, count: &Cell<i32>) {
    if (!fs.is_dir) {
        return;
    };

    fs.size.get().and_then(|f: i32| {
        if f <= 100000 {
            let c = count.get();
            println!("Adding {f}");
            count.set(c + f);
            let curcount = count.get();
            println!("Count now {curcount}");
        };

        None::<Option<i32>>
    });

    for entry in fs.children.borrow().iter() {
        sum_size(Rc::clone(entry.1), count)
    }
}

fn collect_fs_to_vec(fs: Rc<Node>, v: &mut Vec<i32>) {
    if (!fs.is_dir) {
        return;
    }

    v.push(fs.size.get().unwrap());

    for pair in fs.children.borrow().iter() {
        collect_fs_to_vec(Rc::clone(pair.1), v);
    }
}

fn find_dir_to_delete(fs: Rc<Node>) -> i32 {
    let occupied_space = fs.size.get().unwrap();
    let storage_size = 70000000;
    let current_free_space = storage_size - occupied_space;
    let min_space_to_free = 30000000 - current_free_space;

    let mut dirs = vec![];

    collect_fs_to_vec(fs, &mut dirs);

    dirs.sort();

    *dirs.iter().find(|i| **i > min_space_to_free).unwrap()
}

fn fun(input: &str) -> NodeRef {
    let root: NodeRef = Rc::new(Node::new(None, None, true));

    let mut current_node = Rc::clone(&root);

    println!("{}", input);

    // let mut v: Vec<String> = vec![];

    // let mut path_stack: Vec<&mut FsNode> = vec![&mut root];

    // let mut current_node = &root;

    // let mut current_dir: &mut &FsNode;

    // let mut current_folder: &FsNode = &root;

    input.lines().for_each(|f| {
        println!("{f}");
        if f.as_bytes()[0] == b'$' {
            // println!("is cmd");
            match f {
                "$ cd /" => current_node = Rc::clone(&root),
                "$ ls" => (),
                "$ cd .." => {
                    current_node =
                        Rc::clone(&current_node.parent.clone().expect("node to have a parent"))
                }
                _ => {
                    // cd $dirname
                    let (_, dirname) = f.split_at(5);
                    let c2: NodeRef;
                    {
                        let child = current_node.children.borrow();
                        c2 = Rc::clone(child.get(dirname).expect("dir to contain child"));
                    }

                    current_node = c2

                    // let current_directory: &mut &mut &mut FsNode = path_stack.last().unwrap();
                    // let new_dir: &mut FsNode = current_directory.items.get_mut(dirname).unwrap();
                    // let n2 = current_node;

                    // v.push(dirname.to_owned());
                    // current_node =

                    // let childref = v.last().expect("v to contain last").children.borrow();
                    // let c2 = childref.get(dirname).expect("dir");

                    // // let c2 = child_node.get(dirname).expect("dirname to be found");

                    // v.push(c2)
                }
            }
        } else {
            // println!("isnt cmd");
            let (left, right) = f.split_once(' ').expect("line to split into two");

            if left == "dir" {
                let newnode = Node::new(None, Some(Rc::clone(&current_node)), true);
                // let current_node = v.last().expect("tehre to be a last node");

                current_node
                    .children
                    .borrow_mut()
                    .insert(right.to_owned(), Rc::new(newnode));

                // let kv = current_node.borrow_mut();

                // current_node
                //     .borrow()
                //     .children
                //     .borrow_mut()
                //     .insert(right.to_owned(), newnode);
                // v.last_mut().unwrap().add_child(Node::new(None), right);
                // add_vec_node(&mut v, VecNode::new(None, true, current_dir), right)
            } else {
                //File
                let size: i32 = left.parse().unwrap();
                let newnode = Node::new(Some(size), None, false);
                // let current_node = *v.last().expect("tehre to be a last node");

                current_node
                    .children
                    .borrow_mut()
                    .insert(right.to_owned(), Rc::new(newnode));
                // current_node
                //     .get_mut()
                //     .add_child(Node::new(Some(size), Some(newnode)), right)
                // v.last_mut().unwrap().add_child(Node::new(None), right);
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
        set_dir_sizes(Rc::clone(&fs));

        assert_eq!(fs.size.get().unwrap_or_default(), 48381165);

        set_dir_sizes(Rc::clone(&fs));

        let countcell = Cell::new(0);
        sum_size(Rc::clone(&fs), &countcell);

        assert_eq!(countcell.get(), 95437);

        Ok(())
    }
}
