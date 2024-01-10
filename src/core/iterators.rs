//!iterators.rs

use super::{Edge, Head, Node};
use std::marker::PhantomData;
use std::rc::Rc;

pub struct IterEdges<N: Node<V, E, H>, V: 'static, E: Edge<N, V, H>, H: Head> {
    node: Rc<N>,
    edge_index: usize,
    _v: PhantomData<V>,
    _e: PhantomData<E>,
    _m: PhantomData<H>,
    finished: bool, // true if iterator finished
}

impl<N: Node<V, E, H>, V, E: Edge<N, V, H>, H: Head> IterEdges<N, V, E, H> {
    pub fn new(node: Rc<N>) -> Self {
        IterEdges {
            node,
            edge_index: 0,
            _v: PhantomData,
            _e: PhantomData,
            _m: PhantomData,
            finished: false,
        }
    }
}

impl<N: Node<V, E, H>, V, E: Edge<N, V, H>, H: Head> Iterator for IterEdges<N, V, E, H> {
    type Item = Rc<E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        match self.node.get_edge(self.edge_index) {
            Some(node) => {
                self.edge_index += 1;
                Some(node)
            }
            None => {
                self.finished = true;
                None
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.node.len_edges()))
    }
}

impl<N: Node<V, E, H>, V, E: Edge<N, V, H>, H: Head> ExactSizeIterator for IterEdges<N, V, E, H> {
    fn len(&self) -> usize {
        self.node.len_edges()
    }
}
