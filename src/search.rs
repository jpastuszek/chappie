use std::iter::Iterator;
use std::collections::HashSet;
use std::hash::Hash;
use std::borrow::Borrow;


struct Visited<T> {
    hash_set: HashSet<T>
}

impl<T> Visited<T> where T: Hash + Eq {
    fn new() -> Visited<T> {
        Visited {
            hash_set: HashSet::new()
        }
    }

    fn is_visited(&self, value: &T) -> bool {
        self.hash_set.contains(value)
    }

    fn insert(&mut self, value: T) -> bool {
        self.hash_set.insert(value)
    }
}

pub trait SearchGoal<T> {
    fn is_goal(&self, state: &T) -> bool;
}

impl<T> SearchGoal<T> for T where T: PartialEq {
    fn is_goal(&self, state: &T) -> bool {
        self == state
    }
}

pub trait SearchSpace<'a> {
    type State;
    type Action;

    type BState: Borrow<Self::State> + Hash + Eq;
    type BAction: Borrow<Self::Action>;

    fn expand(&'a self, state: &Self::State) -> Box<Iterator<Item=(Self::BAction, Self::BState)> + 'a>;

    fn dfs<G>(&'a self, start: Self::BState, goal: &G) -> Option<Vec<Self::BAction>>
    where G: SearchGoal<Self::State> {
        if goal.is_goal(start.borrow()) {
            return Some(vec![]);
        }

        let mut visited = Visited::new();
        let mut stack = vec![(self.expand(start.borrow()), None)];

        loop {
            let next = match stack.last_mut() {
                None => return None,
                Some(&mut (ref mut iter, _)) => iter.next()
            };
            if let Some((action, state)) = next {
                if visited.is_visited(&state) {
                    continue;
                }
                if goal.is_goal(state.borrow()) {
                    return Some(
                        stack.into_iter()
                             .filter_map(|(_, a)| a)
                             .chain(Some(action).into_iter())
                             .collect()
                    )
                }
                stack.push((self.expand(state.borrow()), Some(action)));
                visited.insert(state);
            } else {
                stack.pop();
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::{SearchSpace, SearchGoal};
    use std::vec::IntoIter;
    use std::slice::Iter;
    use std::iter::Map;
    use rand::chacha::ChaChaRng;
    use rand::Rng;
    use std::iter::Enumerate;
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::marker::PhantomData;

    /*
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

    impl SearchSpace for RandomGraph {
        type State = usize;
        type Action = usize;
        type Iterator = Enumerate<IntoIter<usize>>;

        fn expand(&self, state: &usize) -> Self::Iterator {
            if *state < self.nodes.len() {
                self.nodes[*state].clone()
            } else {
                vec![]
            }.into_iter().enumerate()
        }
    }

    struct Observer<T> {
        goal: T,
        visited: RefCell<Vec<T>>,
    }

    impl<T> Observer<T> where T: Clone {
        pub fn new(goal: T) -> Observer<T> {
            Observer {
                goal: goal,
                visited: RefCell::new(vec![])
            }
        }

        pub fn visited(&self) -> Vec<T> {
            self.visited.borrow().clone()
        }
    }

    impl<T> SearchGoal<T> for Observer<T> where T: PartialEq + Clone {
        fn is_goal(&self, state: &T) -> bool {
            self.visited.borrow_mut().push(state.clone());
            self.goal == *state
        }
    }
    */

    #[test]
    pub fn test_dfs_simple() {
        struct TestSearch;

        #[derive(Debug, PartialEq)]
        enum Dir { Left, Right }

        impl<'a> SearchSpace<'a> for TestSearch {
            type State = i32;
            type Action = Dir;

            type BState = i32;
            type BAction = Dir;

            fn expand(&'a self, state: &Self::State) -> Box<Iterator<Item=(Self::BAction, Self::BState)> + 'a> {
                Box::new(
                    match *state {
                        0 => vec![(Dir::Left, 1), (Dir::Right, 2)],
                        1 => vec![(Dir::Left, 3), (Dir::Right, 4)],
                        2 => vec![(Dir::Left, 2)],
                        _ => vec![]
                    }.into_iter()
                )
            }
        }

        let ts = TestSearch;

        assert_eq!(ts.dfs(0, &0).unwrap(), Vec::<Dir>::new());
        assert_eq!(ts.dfs(0, &1).unwrap(), vec![Dir::Left]);
        assert_eq!(ts.dfs(0, &2).unwrap(), vec![Dir::Right]);
        assert_eq!(ts.dfs(0, &3).unwrap(), vec![Dir::Left, Dir::Left]);
        assert_eq!(ts.dfs(0, &4).unwrap(), vec![Dir::Left, Dir::Right]);
        assert_eq!(ts.dfs(2, &2).unwrap(), Vec::<Dir>::new());
        assert!(ts.dfs(2, &0).is_none());
    }

    #[test]
    pub fn test_dfs_simple_by_ref() {
        #[derive(Debug, PartialEq, Clone)]
        enum Dir { Left, Right }

        struct TestSearch<'a, T: 'a> {
            nodes: Vec<Vec<(Dir, usize)>>,
            phantom: PhantomData<&'a T>
        }

        impl<'a, T> SearchSpace<'a> for TestSearch<'a, T> {
            type State = usize;
            type Action = Dir;

            // HKT needed to leave lifetime unspecified here and specify it in fn expand as self
            // lifetime
            type BState = &'a usize;
            type BAction = &'a Dir;

            fn expand(&'a self, state: &Self::State) -> Box<Iterator<Item=(Self::BAction, Self::BState)> + 'a> {
                Box::new(
                    self.nodes.iter().nth(*state).expect(format!("no state: {}", *state).trim()).iter()
                    .map(|&(ref a, ref s)| (a, s))
                )
            }
        }

        let ts: TestSearch<usize> = TestSearch {
            nodes: vec![
                vec![(Dir::Left, 1), (Dir::Right, 2)],  // 0
                vec![(Dir::Left, 3), (Dir::Right, 4)],  // 1
                vec![(Dir::Left, 2)],                   // 2
                vec![],                                 // 3
                vec![]                                  // 4
            ],
            phantom: PhantomData
        };

        assert_eq!(ts.dfs(&0, &0).unwrap(), Vec::<&Dir>::new());
        assert_eq!(ts.dfs(&0, &1).unwrap(), vec![&Dir::Left]);
        assert_eq!(ts.dfs(&0, &2).unwrap(), vec![&Dir::Right]);
        assert_eq!(ts.dfs(&0, &3).unwrap(), vec![&Dir::Left, &Dir::Left]);
        assert_eq!(ts.dfs(&0, &4).unwrap(), vec![&Dir::Left, &Dir::Right]);
        assert_eq!(ts.dfs(&2, &2).unwrap(), Vec::<&Dir>::new());
        assert!(ts.dfs(&2, &0).is_none());
    }

/*
    #[test]
    pub fn test_dfs_random() {
        const N_NODES: usize = 48;
        const MAX_EDGES: usize = 6;

        let g = RandomGraph::new(N_NODES, MAX_EDGES);

        assert!(g.dfs(&N_NODES, &0).is_none());
        assert!(g.dfs(&0, &N_NODES).is_none());

        for start in (0..N_NODES) {
            for goal in (0..N_NODES) {
                let observer = Observer::new(goal);
                if let Some(path) = g.dfs(&start, &observer) {
                    let mut state = start;
                    for action in path {
                        state = g.expand(&state).skip(action).next().unwrap().1;
                    }
                    assert_eq!(state, goal);
                } else {
                    let visited: HashSet<_> = observer.visited().iter().cloned().collect();
                    for state in observer.visited() {
                        assert!(!observer.is_goal(&state));
                        for (_, next_state) in g.expand(&state) {
                            assert!(visited.contains(&next_state));
                        }
                    }
                }
            }
        }
    }
*/
}
