use std::iter::Iterator;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::hash::{Hash, Hasher, SipHasher};

struct HashTracker<T> {
    hash_set: HashSet<u64>,
    phantom: PhantomData<T>,
}

impl<T> HashTracker<T> where T: Hash {
    fn new() -> HashTracker<T> {
        HashTracker {
            hash_set: HashSet::new(),
            phantom: PhantomData
        }
    }

    fn contains(&mut self, item: &T) -> bool {
        let mut hasher = SipHasher::new();
        item.hash(&mut hasher);
        let hash = hasher.finish();
        let contains = self.hash_set.contains(&hash);
        if !contains {
            self.hash_set.insert(hash);
        }
        contains
    }
}

pub trait SearchGoal<T> {
    fn is_goal(&self, state: &T) -> bool;
}

impl<T> SearchGoal<T> for T where T: PartialEq {
    fn is_goal(&self, state: &T) -> bool {
        *self == *state
    }
}

pub trait SearchSpace {
    type State: Hash;
    type Action;

    fn expand(&self, state: &Self::State)
    -> Box<Iterator<Item=(Self::Action, Self::State)>>;

    fn dfs<G>(&self, start: Self::State, goal: G) -> Option<Vec<Self::Action>>
    where G: SearchGoal<Self::State> {
        if goal.is_goal(&start) {
            return Some(vec![]);
        }

        let mut path = Vec::new();
        let mut visited = HashTracker::new();
        let mut frontier = vec![self.expand(&start)];

        loop {
            let result = match frontier.last_mut() {
                None => {
                    return None
                },
                Some(&mut ref mut iter) => {
                    match iter.next() {
                        None => {
                            path.pop();
                            None
                        },
                        Some((action, state)) => {
                            if visited.contains(&state) {
                                continue;
                            }
                            path.push(action);
                            if goal.is_goal(&state) {
                                return Some(path);
                            }
                            Some(self.expand(&state))
                        }
                    }
                }
            };
            match result {
                None => {
                    frontier.pop();
                }
                Some(iter) => {
                    frontier.push(iter);
                }
            };
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::SearchSpace;

    #[test]
    pub fn test_dfs() {
        #[derive(Debug, PartialEq)]
        enum Dir { Left, Right }

        struct TestSearch;

        impl SearchSpace for TestSearch {
            type State = i32;
            type Action = Dir;

            fn expand(&self, state: &Self::State)
            -> Box<Iterator<Item=(Self::Action, Self::State)>> {
                Box::new(match *state {
                    0 => vec![(Dir::Left, 1), (Dir::Right, 2)],
                    1 => vec![(Dir::Left, 3), (Dir::Right, 4)],
                    2 => vec![(Dir::Left, 2)],
                    _ => vec![],
                }.into_iter())
            }
        }

        let ts = TestSearch;

        assert_eq!(ts.dfs(0, 0).unwrap(), vec![]);
        assert_eq!(ts.dfs(0, 1).unwrap(), vec![Dir::Left]);
        assert_eq!(ts.dfs(0, 2).unwrap(), vec![Dir::Right]);
        assert_eq!(ts.dfs(0, 3).unwrap(), vec![Dir::Left, Dir::Left]);
        assert_eq!(ts.dfs(0, 4).unwrap(), vec![Dir::Left, Dir::Right]);
        assert_eq!(ts.dfs(2, 2).unwrap(), vec![]);
        assert!(ts.dfs(2, 0).is_none());
        assert!(ts.dfs(5, 0).is_none());
    }
}
