//!baseedge.rs

use crate::core::{Edge, EdgeArrow, EdgeLink, EdgeLoop, EdgeType, EdgeWeighted, Head, Node};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

// BaseEdge implements all types of edge, which has some side effects:
// 1. Nodes have references on edges and edges have references on nodes.
// Therefore it is very easy to create

enum EdgeAlpha<N> {
    None,
    Tail(Weak<N>),
    Left(Rc<N>),
}

enum EdgeOmega<N> {
    Head(Rc<N>),
    Right(Rc<N>),
    Loop(Weak<N>),
}

pub struct BaseEdge<N: Node<V, Self, H>, V: 'static, H: Head, W: Default + 'static> {
    weight: RefCell<W>,
    id: usize,
    selfie: RefCell<Weak<BaseEdge<N, V, H, W>>>,
    alpha: EdgeAlpha<N>,
    omega: EdgeOmega<N>,
}

impl<N: Node<V, Self, H>, V: 'static, H: Head, W: Default + 'static> Edge<N, V, H>
    for BaseEdge<N, V, H, W>
{
    fn get_id(&self) -> usize {
        self.id
    }
    fn get_self(&self) -> Option<Rc<Self>> {
        self.selfie.borrow().upgrade().as_ref().cloned()
    }
    fn get_edge_type(&self) -> EdgeType {
        match (&self.alpha, &self.omega) {
            (EdgeAlpha::Tail(_), EdgeOmega::Head(_)) => EdgeType::Arrow,
            (EdgeAlpha::Left(_), EdgeOmega::Right(_)) => EdgeType::Link,
            (EdgeAlpha::None, EdgeOmega::Loop(_)) => EdgeType::Loop,
            _ => panic!("unknown edge end configuration"),
        }
    }
    // if node with node_id points inside this edge toward another node, return node pointed to (head)
    fn try_head_node(&self, node_id: usize) -> Option<Rc<N>> {
        match &self.alpha {
            EdgeAlpha::Tail(weak) => {
                if let Some(node) = weak.upgrade() {
                    if node.get_id() == node_id {
                        if let EdgeOmega::Head(head) = &self.omega {
                            return Some(head.clone());
                        }
                    }
                }
            }
            EdgeAlpha::Left(node) => {
                if node.get_id() == node_id {
                    if let EdgeOmega::Right(head) = &self.omega {
                        return Some(head.clone());
                    }
                }
            }
            EdgeAlpha::None => (),
        }
        match &self.omega {
            EdgeOmega::Right(node) => {
                if node.get_id() == node_id {
                    if let EdgeAlpha::Left(head) = &self.alpha {
                        return Some(head.clone());
                    }
                }
            }
            EdgeOmega::Loop(weak) => {
                if let Some(node) = weak.upgrade() {
                    if node.get_id() == node_id {
                        return Some(node.clone());
                    }
                }
            }
            EdgeOmega::Head(_) => (),
        }
        None
    }
    // if a node of this edge points toward node with node_id, return node pointed from (tail)
    fn try_tail_node(&self, node_id: usize) -> Option<Rc<N>> {
        match &self.omega {
            EdgeOmega::Head(node) => {
                if node.get_id() == node_id {
                    if let EdgeAlpha::Tail(weak) = &self.alpha {
                        if let Some(tail) = weak.upgrade() {
                            return Some(tail.clone());
                        }
                    }
                }
            }
            EdgeOmega::Right(node) => {
                if node.get_id() == node_id {
                    if let EdgeAlpha::Left(tail) = &self.alpha {
                        return Some(tail.clone());
                    }
                }
            }
            EdgeOmega::Loop(weak) => {
                if let Some(node) = weak.upgrade() {
                    if node.get_id() == node_id {
                        return Some(node.clone());
                    }
                }
            }
        }
        if let EdgeAlpha::Left(node) = &self.alpha {
            if node.get_id() == node_id {
                if let EdgeOmega::Right(tail) = &self.omega {
                    return Some(tail.clone());
                }
            }
        }
        None
    }
}

impl<N: Node<V, Self, H>, V: 'static, H: Head, W: Default> EdgeArrow<N, V, H>
    for BaseEdge<N, V, H, W>
{
    fn new_arrow(tail: Rc<N>, head: Rc<N>, meta: Rc<H>) -> Rc<Self> {
        let arrow = Rc::new(Self {
            weight: RefCell::new(W::default()),
            id: meta.new_edge_id(),
            selfie: RefCell::new(Weak::new()),
            alpha: EdgeAlpha::Tail(Rc::downgrade(&tail)),
            omega: EdgeOmega::Head(head),
        });
        let selfie = Rc::downgrade(&arrow);
        *arrow.selfie.borrow_mut() = selfie;
        arrow
    }
    fn head_node(&self) -> Option<Rc<N>> {
        match &self.omega {
            EdgeOmega::Head(node) => Some(node.clone()),
            _ => None,
        }
    }
    fn tail_node(&self) -> Option<Rc<N>> {
        match &self.alpha {
            EdgeAlpha::Tail(node) => node.upgrade().as_ref().cloned(),
            _ => None,
        }
    }
}

impl<N: Node<V, Self, H>, V: 'static, H: Head, W: Default> EdgeLink<N, V, H>
    for BaseEdge<N, V, H, W>
{
    fn new_link(left: Rc<N>, right: Rc<N>, meta: Rc<H>) -> Rc<Self> {
        let link = Rc::new(Self {
            weight: RefCell::new(W::default()),
            id: meta.new_edge_id(),
            selfie: RefCell::new(Weak::new()),
            alpha: EdgeAlpha::Left(left),
            omega: EdgeOmega::Right(right),
        });
        let selfie = Rc::downgrade(&link);
        *link.selfie.borrow_mut() = selfie;
        link
    }
    fn right_node(&self) -> Option<Rc<N>> {
        match &self.omega {
            EdgeOmega::Right(node) => Some(node.clone()),
            _ => None,
        }
    }
    fn left_node(&self) -> Option<Rc<N>> {
        match &self.alpha {
            EdgeAlpha::Left(node) => Some(node.clone()),
            _ => None,
        }
    }
}

impl<N: Node<V, Self, H>, V: 'static, H: Head, W: Default> EdgeLoop<N, V, H>
    for BaseEdge<N, V, H, W>
{
    fn new_loop(node: Rc<N>, meta: Rc<H>) -> Rc<Self> {
        let loop_node = Rc::new(Self {
            weight: RefCell::new(W::default()),
            id: meta.new_edge_id(),
            selfie: RefCell::new(Weak::new()),
            alpha: EdgeAlpha::None,
            omega: EdgeOmega::Loop(Rc::downgrade(&node)),
        });
        let selfie = Rc::downgrade(&loop_node);
        *loop_node.selfie.borrow_mut() = selfie;
        loop_node
    }
    fn loop_node(&self) -> Option<Rc<N>> {
        match &self.omega {
            EdgeOmega::Loop(node) => node.upgrade().as_ref().cloned(),
            _ => None,
        }
    }
}

impl<N: Node<V, Self, H>, V: 'static, H: Head, W: Default + 'static> EdgeWeighted<N, V, H, W>
    for BaseEdge<N, V, H, W>
{
    fn get_weight(&self) -> std::cell::Ref<'_, W> {
        self.weight.borrow()
    }
    fn get_weight_mut(&self) -> std::cell::RefMut<'_, W> {
        self.weight.borrow_mut()
    }
}
