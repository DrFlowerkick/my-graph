//!baseedge.rs

use crate::core::{Node, Edge, Head};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};


pub struct BaseEdge<N: Node<V, Self, H>, V: 'static, H: Head> {
    id: usize,
    selfie: RefCell<Weak<BaseEdge<N, V, H>>>,
    alpha: Option<Rc<N>>, // Tail or Left
    omega: Rc<N>, // Head or Right or Loop
}

impl<N: Node<V, Self, H>, V: 'static, H: Head> Edge<N, V, H> for BaseEdge<N, V, H> {
    fn get_id(&self) -> usize {
        self.id
    }
    fn get_self(&self) -> Option<Rc<Self>> {
        self.selfie.borrow().upgrade().as_ref().cloned()
    }
    fn try_head_node(&self, node_id: usize) -> Option<Rc<N>> {
        if self.omega.get_id() == node_id {
            Some(self.omega.clone())
        } else {
            None
        }
    }
    fn try_tail_node(&self, node_id: usize) -> Option<Rc<N>> {
        if let Some(node) = &self.alpha {
            if node.get_id() == node_id {
                return Some(node.clone());
            }
        }
        None
    }
}