use ahash::{AHashMap, AHashSet};
use std::collections::VecDeque;

#[derive(Default, Clone)]
pub struct Graph<Key: Default + Copy + Eq + std::hash::Hash> {
    adjacenty_list: AHashMap<Key, AHashSet<Key>>,
    queue: VecDeque<Key>,
    visited: AHashSet<Key>,
}

impl<Key: Default + Copy + Eq + std::hash::Hash> Graph<Key> {
    /// Add edge from `src` to `dst`.
    pub fn add_edge(&mut self, src: Key, dst: Key) {
        self.adjacenty_list
            .entry(src)
            .or_insert_with(AHashSet::default)
            .insert(dst);

        self.adjacenty_list
            .entry(dst)
            .or_insert_with(AHashSet::default);
    }

    pub fn clear(&mut self) {
        self.adjacenty_list.clear();
        self.queue.clear();
        self.visited.clear();
    }

    /// A BFS based function to check whether `dst` is reachable from `src`.
    pub fn reachable(&mut self, src: Key, dst: Key) -> bool {
        // Base case
        if src == dst {
            return true;
        }

        // Mark all the vertices as not visited
        self.visited.clear();

        // Create a queue for BFS
        self.queue.clear();

        // Mark the current node as visited and enqueue it
        self.visited.insert(src);
        self.queue.push_back(src);

        // Dequeue a vertex from queue and print it
        while let Some(n) = self.queue.pop_front() {
            // Get all adjacent vertices of the dequeued vertex s
            // If a adjacent has not been visited, then mark it visited and enqueue it
            for &adjacent_node in self.adjacenty_list[&n].iter() {
                // If this adjacent node is the destination node, then return true
                if adjacent_node == dst {
                    return true;
                }

                // Else, continue to do BFS
                if !self.visited.contains(&adjacent_node) {
                    self.visited.insert(adjacent_node);
                    self.queue.push_back(adjacent_node);
                }
            }
        }

        // If BFS is complete without visiting d
        false
    }

    // prints all not yet visited vertices reachable from s
    pub fn _dfs<E>(&self, s: Key, mut handle: impl FnMut(Key) -> Result<(), E>) -> Result<(), E> {
        // Initially mark all verices as not visited
        let mut visited: AHashSet<Key> = AHashSet::default();

        // Create a stack for DFS and push the current source node.
        let mut stack = vec![s];

        // Pop a vertex from stack and print it
        while let Some(s) = stack.pop() {
            // Stack may contain same vertex twice.
            // So we need to print the popped item only if it is not visited.
            if !visited.contains(&s) {
                handle(s)?;
                visited.insert(s);
            }

            // Get all adjacent vertices of the popped vertex s
            // If a adjacent has not been visited, then push it
            // to the stack.
            if let Some(list) = self.adjacenty_list.get(&s) {
                for item in list.iter() {
                    if !visited.contains(item) {
                        stack.push(*item);
                    }
                }
            }
        }
        Ok(())
    }
}

#[test]
fn test_detect_cyclic() {
    // Driver program to test methods of graph class
    // Create a graph given in the above diagram
    let mut g = Graph::<i8>::default();

    g.add_edge(0, 1);
    g.add_edge(0, 2);
    g.add_edge(1, 2);
    g.add_edge(2, 0);
    g.add_edge(2, 3);
    g.add_edge(3, 3);

    assert!(g.reachable(1, 3));
    assert!(!g.reachable(3, 1));
    assert!(g.reachable(3, 3));
}
