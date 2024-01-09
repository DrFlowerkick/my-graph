//!baseedge.rs

use crate::core::{Node, Edge, EdgeType, EdgeArrow, Head, EdgeLink};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};


// BaseEdge implements all types of edge, which has some side effects:
// 1. Nodes have references on edges and edges have references on nodes.
// Therefore it is very easy to create 

pub struct BaseEdge<N: Node<V, Self, H>, V: 'static, H: Head> {
    id: usize,
    etype: EdgeType,
    selfie: RefCell<Weak<BaseEdge<N, V, H>>>,
    nodes: RefCell<Vec<Rc<N>>>,
}

impl<N: Node<V, Self, H>, V: 'static, H: Head> Edge<N, V, H> for BaseEdge<N, V, H> {
    fn get_id(&self) -> usize {
        self.id
    }
    fn get_self(&self) -> Option<Rc<Self>> {
        self.selfie.borrow().upgrade().as_ref().cloned()
    }
    fn get_edge_type(&self) -> EdgeType {
        self.etype
    }
    fn try_head_node(&self, node_id: usize) -> Option<Rc<N>> {
        match self.etype {
            EdgeType::Arrow => if self.nodes.borrow()[1].get_id() == node_id {
                return Some(self.nodes.borrow()[1].clone());
            }
            EdgeType::Link => if let Some(pos) = self.nodes.borrow().iter().position(|n| n.get_id() == node_id) {
                return Some(self.nodes.borrow()[pos].clone())
            },
            EdgeType::Loop => if self.nodes.borrow()[0].get_id() == node_id {
                return Some(self.nodes.borrow()[0].clone());
            },
        }
        None
    }
    fn try_tail_node(&self, node_id: usize) -> Option<Rc<N>> {
        match self.etype {
            EdgeType::Arrow => if self.nodes.borrow()[0].get_id() == node_id {
                return Some(self.nodes.borrow()[0].clone());
            }
            EdgeType::Link => if let Some(pos) = self.nodes.borrow().iter().position(|n| n.get_id() == node_id) {
                return Some(self.nodes.borrow()[pos].clone())
            },
            EdgeType::Loop => if self.nodes.borrow()[0].get_id() == node_id {
                return Some(self.nodes.borrow()[0].clone());
            },
        }
        None
    }
}

impl<N: Node<V, Self, H>, V: 'static, H: Head> EdgeArrow<N, V, H> for BaseEdge<N, V, H> {
    fn new_arrow(tail: Rc<N>, head: Rc<N>, meta: Rc<H>) -> Rc<Self> {
        let arrow = Rc::new(Self {
            id: meta.new_edge_id(),
            etype: EdgeType::Arrow,
            selfie: RefCell::new(Weak::new()),
            nodes: RefCell::new(Vec::new()),
        });
        let selfie = Rc::downgrade(&arrow);
        *arrow.selfie.borrow_mut() = selfie;
        // tail is always index 0
        arrow.nodes.borrow_mut().push(tail);
        // head is always index 1
        arrow.nodes.borrow_mut().push(head);
        arrow
    }
    fn head_node(&self) -> Option<Rc<N>> {
        match self.etype {
            EdgeType::Arrow => Some(self.nodes.borrow()[1].clone()),
            _ => None,
        }
    }
    fn tail_node(&self) -> Option<Rc<N>> {
        match self.etype {
            EdgeType::Arrow => Some(self.nodes.borrow()[0].clone()),
            _ => None,
        }
    }
}

impl<N: Node<V, Self, H>, V: 'static, H: Head> EdgeLink<N, V, H> for BaseEdge<N, V, H> {
    fn new_link(left: Rc<N>, right: Rc<N>, meta: Rc<H>) -> Rc<Self> {
        let link = Rc::new(Self {
            id: meta.new_edge_id(),
            etype: EdgeType::Link,
            selfie: RefCell::new(Weak::new()),
            nodes: RefCell::new(Vec::new()),
        });
        let selfie = Rc::downgrade(&link);
        *link.selfie.borrow_mut() = selfie;
        // left is always index 0
        link.nodes.borrow_mut().push(left);
        // right is always index 1
        link.nodes.borrow_mut().push(right);
        link
    }
    fn right_node(&self) -> Option<Rc<N>> {
        match self.etype {
            EdgeType::Arrow => Some(self.nodes.borrow()[1].clone()),
            _ => None,
        }
    }
    fn left_node(&self) -> Option<Rc<N>> {
        match self.etype {
            EdgeType::Arrow => Some(self.nodes.borrow()[0].clone()),
            _ => None,
        }
    }
}

