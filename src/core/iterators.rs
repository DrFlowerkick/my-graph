//!iterators.rs

use super::{Link, Node};
use std::marker::PhantomData;
use std::rc::Rc;

pub struct IterLinks<N: Node<V, L, W>, V: 'static, L: Link<N, V, W>, W: Default + 'static> {
    node: Rc<N>,
    link_index: usize,
    _v: PhantomData<V>,
    _e: PhantomData<L>,
    _m: PhantomData<W>,
    finished: bool, // true if iterator finished
}

impl<N: Node<V, L, W>, V, L: Link<N, V, W>, W: Default> IterLinks<N, V, L, W> {
    pub fn new(node: Rc<N>) -> Self {
        IterLinks {
            node,
            link_index: 0,
            _v: PhantomData,
            _e: PhantomData,
            _m: PhantomData,
            finished: false,
        }
    }
}

impl<N: Node<V, L, W>, V, L: Link<N, V, W>, W: Default> Iterator for IterLinks<N, V, L, W> {
    type Item = Rc<L>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        match self.node.get_link(self.link_index) {
            Some(node) => {
                self.link_index += 1;
                Some(node)
            }
            None => {
                self.finished = true;
                None
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.node.len_links()))
    }
}

impl<N: Node<V, L, W>, V, L: Link<N, V, W>, W: Default> ExactSizeIterator
    for IterLinks<N, V, L, W>
{
    fn len(&self) -> usize {
        self.node.len_links()
    }
}
