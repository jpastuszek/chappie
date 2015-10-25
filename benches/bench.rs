#![feature(test)]

extern crate chappie;
extern crate test;

use chappie::search::SearchSpace;
use test::{Bencher, black_box};
use std::vec::IntoIter;

enum Dir { Left, Right}

struct BinaryTree;

const MAX_DEPTH: u64 = 16;
const MAX_OFFSET: u64 = 1 << (MAX_DEPTH + 1);

impl<'a> SearchSpace<'a> for BinaryTree {
    type State = u64;
    type Action = Dir;
    type Iterator = IntoIter<(Self::Action, Self::State)>;

    fn expand(&'a self, state: &Self::State) -> Self::Iterator {
        let offset = (*state + 2).next_power_of_two();
        if offset >= MAX_OFFSET {
            return vec![].into_iter();
        }
        let right = *state + offset;
        let left = *state + offset / 2;
        vec![(Dir::Left, left), (Dir::Right, right)].into_iter()
    }
}

struct BinaryTreeByRef {
    nodes: Vec<Vec<(Dir, u64)>>
}

impl BinaryTreeByRef {
    fn new() -> BinaryTreeByRef {
        let mut nodes = vec![];
        let tree = BinaryTree;
        let max_nodes: u64 = 2u64.pow((MAX_DEPTH + 1) as u32) - 1;

        for node in 0..max_nodes {
            nodes.push(tree.expand(&node).collect());
        }

        BinaryTreeByRef { nodes: nodes }
    }
}

impl<'a> SearchSpace<'a> for BinaryTreeByRef {
    type State = &'a u64;
    type Action = &'a Dir;
    type Iterator = IntoIter<(Self::Action, Self::State)>;

    fn expand(&'a self, state: &Self::State) -> Self::Iterator {
        self.nodes
            .iter()
            .nth(**state as usize).unwrap().iter()
            .map(|&(ref a, ref s)| (a, s))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[bench]
fn dfs(b: &mut Bencher) {
    let tree = BinaryTree;
    b.iter(|| { black_box(tree.dfs(0, |&s| s == 2)) });
}

#[bench]
fn dfs_by_ref(b: &mut Bencher) {
    let tree = BinaryTreeByRef::new();
    let start = 0;
    b.iter(|| { black_box(tree.dfs(&start, |&s| *s == 2)) });
}
