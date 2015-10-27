use std::iter::Iterator;
use std::collections::HashSet;
use std::hash::Hash;

pub trait SearchSpace<'a> {
    type State: Hash + Eq + Clone;
    type Action;
    type Iterator: Iterator<Item=(Self::Action, Self::State)>;

    fn expand(&'a self, state: &Self::State) -> Self::Iterator;

    fn dfs<P>(&'a self, start: Self::State, predicate: P) -> Option<Vec<Self::Action>>
    where P: Fn(&Self::State) -> bool
    {
        if predicate(&start) {
            return Some(vec![]);
        }

        let mut visited = HashSet::new();
        let mut stack = vec![(self.expand(&start), None)];

        loop {
            let next = match stack.last_mut() {
                None => return None,
                Some(&mut (ref mut iter, _)) => iter.next()
            };
            if let Some((action, state)) = next {
                if !visited.insert(state.clone()) {
                    continue;
                }
                if predicate(&state) {
                    return Some(
                        stack.into_iter()
                             .filter_map(|(_, a)| a)
                             .chain(Some(action).into_iter())
                             .collect()
                    )
                }
                stack.push((self.expand(&state), Some(action)));
            } else {
                stack.pop();
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::SearchSpace;
    use std::vec::IntoIter;
    use rand::chacha::ChaChaRng;
    use rand::Rng;
    use std::iter::Enumerate;
    use std::cell::RefCell;
    use std::collections::HashSet;

    struct RandomGraph {
        nodes: Vec<Vec<usize>>
    }

    impl RandomGraph {
        fn new(n_nodes: usize, max_edges: usize) -> RandomGraph {
            let mut rng_e = ChaChaRng::new_unseeded();
            let mut rng_n = ChaChaRng::new_unseeded();

            RandomGraph {
                nodes: rng_e.gen_iter::<usize>().take(n_nodes)
                    .map(|e| rng_n.gen_iter::<usize>().take(e % max_edges)
                    .map(|n| n % n_nodes).collect()).collect()
            }
        }

        #[allow(dead_code)]
        fn to_graphviz(&self) -> String {
            let mut buf = "digraph G {".to_owned();
            for (node_from, nodes) in self.nodes.iter().enumerate() {
                for (action, node_to) in nodes.iter().enumerate() {
                    buf.push_str(format!("\n\t{} -> {} [ label=\"{}\" ];",
                                         node_from, node_to, action).as_ref());
                }
            }
            buf.push_str("\n}");
            buf
        }
    }

    impl<'a> SearchSpace<'a> for RandomGraph {
        type State = usize;
        type Action = usize;
        type Iterator = Enumerate<IntoIter<usize>>;

        fn expand(&self, state: &Self::State) -> Self::Iterator {
            if *state < self.nodes.len() {
                self.nodes[*state].clone()
            } else {
                vec![]
            }.into_iter().enumerate()
        }
    }

    #[test]
    pub fn test_dfs_simple() {
        struct TestSearch;

        #[derive(Debug, PartialEq)]
        enum Dir { Left, Right }

        impl<'a> SearchSpace<'a> for TestSearch {
            type State = i32;
            type Action = Dir;
            type Iterator = IntoIter<(Self::Action, Self::State)>;

            fn expand(&'a self, state: &Self::State) -> Self::Iterator {
                match *state {
                    0 => vec![(Dir::Left, 1), (Dir::Right, 2)],
                    1 => vec![(Dir::Left, 3), (Dir::Right, 4)],
                    2 => vec![(Dir::Left, 2)],
                    _ => vec![]
                }.into_iter()
            }
        }

        let ts = TestSearch;

        assert_eq!(ts.dfs(0, |&s| s == 0).unwrap(), Vec::<Dir>::new());
        assert_eq!(ts.dfs(0, |&s| s == 1).unwrap(), vec![Dir::Left]);
        assert_eq!(ts.dfs(0, |&s| s == 2).unwrap(), vec![Dir::Right]);
        assert_eq!(ts.dfs(0, |&s| s == 3).unwrap(), vec![Dir::Left, Dir::Left]);
        assert_eq!(ts.dfs(0, |&s| s == 4).unwrap(), vec![Dir::Left, Dir::Right]);
        assert_eq!(ts.dfs(2, |&s| s == 2).unwrap(), Vec::<Dir>::new());
        assert!(ts.dfs(2, |&s| s == 0).is_none());
    }

    #[test]
    pub fn test_dfs_simple_by_ref() {
        #[derive(Debug, PartialEq)]
        enum Dir { Left, Right }

        struct TestSearch {
            nodes: Vec<Vec<(Dir, u32)>>
        }

        impl<'a> SearchSpace<'a> for TestSearch {
            type State = &'a u32;
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

        let ts: TestSearch = TestSearch {
            nodes: vec![
                vec![(Dir::Left, 1), (Dir::Right, 2)],
                vec![(Dir::Left, 3), (Dir::Right, 4)],
                vec![(Dir::Left, 2)],
                vec![],
                vec![]
            ]
        };

        assert_eq!(ts.dfs(&0, |&s| *s == 0).unwrap(), Vec::<&Dir>::new());
        assert_eq!(ts.dfs(&0, |&s| *s == 1).unwrap(), vec![&Dir::Left]);
        assert_eq!(ts.dfs(&0, |&s| *s == 2).unwrap(), vec![&Dir::Right]);
        assert_eq!(ts.dfs(&0, |&s| *s == 3).unwrap(), vec![&Dir::Left, &Dir::Left]);
        assert_eq!(ts.dfs(&0, |&s| *s == 4).unwrap(), vec![&Dir::Left, &Dir::Right]);
        assert_eq!(ts.dfs(&2, |&s| *s == 2).unwrap(), Vec::<&Dir>::new());
        assert!(ts.dfs(&2, |&s| *s == 0).is_none());
    }

    #[test]
    pub fn test_dfs_random() {
        const N_NODES: usize = 48;
        const MAX_EDGES: usize = 6;

        let g = RandomGraph::new(N_NODES, MAX_EDGES);

        assert!(g.dfs(N_NODES, |&s| s == 0).is_none());
        assert!(g.dfs(0, |&s| s == N_NODES).is_none());

        for start in 0..N_NODES {
            for goal in 0..N_NODES {
                let visited = RefCell::new(HashSet::new());

                if let Some(path) = g.dfs(start, |&s| {
                    visited.borrow_mut().insert(s);
                    s == goal
                }) {
                    let mut state = start;
                    for action in path {
                        state = g.expand(&state).skip(action).next().unwrap().1;
                    }
                    assert_eq!(state, goal);
                } else {
                    for state in visited.borrow().iter() {
                        assert!(*state != goal);
                        for (_, next_state) in g.expand(&state) {
                            assert!(visited.borrow().contains(&next_state));
                        }
                    }
                }
            }
        }
    }
}
