//!baseedge.rs

use crate::core::{
    Edge, EdgeArrow, EdgeLink, EdgeLoop, EdgeType, EdgeWeighted, Head, HeadNodeCache, Node,
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

// BaseEdge implements all types of edge, which has some side effects:
// 1. Nodes have references on edges and edges have references on nodes.
// Therefore it is very easy to create reference cycles, if "strong"
// Rc smart pointers are used. I decided to redesign the structure
// of edges. Edges will be Links. Meta will be removed.

enum EdgeEnd<N> {
    None,
    Weak(Weak<N>),
    Strong(Rc<N>),
}

impl<N> EdgeEnd<N> {
    fn try_node(&self) -> Option<Rc<N>> {
        match self {
            EdgeEnd::None => None,
            EdgeEnd::Weak(weak) => weak.upgrade().as_ref().cloned(),
            EdgeEnd::Strong(node) => Some(node.clone()),
        }
    }
}

pub struct BaseEdge<N: Node<V, Self, H>, V: 'static, H: Head, W: Default + 'static> {
    weight: RefCell<W>,
    edge_type: EdgeType,
    id: usize,
    selfie: RefCell<Weak<BaseEdge<N, V, H, W>>>,
    alpha: EdgeEnd<N>,
    omega: EdgeEnd<N>,
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
        self.edge_type
    }
    // if node with node_id points inside this edge toward another node, return node pointed to (head)
    fn try_head_node(&self, node_id: usize) -> Option<Rc<N>> {
        match self.edge_type {
            EdgeType::Arrow => {
                if let Some(node) = &self.alpha.try_node() {
                    if node.get_id() == node_id {
                        return self.omega.try_node();
                    }
                }
            }
            EdgeType::Link => {
                if let Some(node) = &self.alpha.try_node() {
                    if node.get_id() == node_id {
                        return self.omega.try_node();
                    }
                }
                if let Some(node) = &self.omega.try_node() {
                    if node.get_id() == node_id {
                        return self.alpha.try_node();
                    }
                }
            }
            EdgeType::Loop => {
                if let Some(node) = &self.omega.try_node() {
                    if node.get_id() == node_id {
                        return self.alpha.try_node();
                    }
                }
            }
        }
        None
    }
    // if a node of this edge points toward node with node_id, return node pointed from (tail)
    fn try_tail_node(&self, node_id: usize) -> Option<Rc<N>> {
        match self.edge_type {
            EdgeType::Arrow => {
                if let Some(node) = &self.omega.try_node() {
                    if node.get_id() == node_id {
                        return self.alpha.try_node();
                    }
                }
            }
            EdgeType::Link => {
                if let Some(node) = &self.alpha.try_node() {
                    if node.get_id() == node_id {
                        return self.omega.try_node();
                    }
                }
                if let Some(node) = &self.omega.try_node() {
                    if node.get_id() == node_id {
                        return self.alpha.try_node();
                    }
                }
            }
            EdgeType::Loop => {
                if let Some(node) = &self.omega.try_node() {
                    if node.get_id() == node_id {
                        return self.alpha.try_node();
                    }
                }
            }
        }
        None
    }
}


// Thinking more and more about circular ownership. With graphs, were node contain the Information about thier edges,
// circular ownership or references cannot be prevented, of strong (normal) Rc smart pointers are used.
// You have to use weak pointers. But than you always have to cache nodes, because every node inside a weak edge ref would
// be dropped, the moment the node goes out of scope.+

impl<N: Node<V, Self, H>, V: 'static, H: Head, W: Default> EdgeArrow<N, V, H>
    for BaseEdge<N, V, H, W>
{
    fn new_arrow(tail: Rc<N>, head: Rc<N>, meta: Rc<H>) -> Rc<Self> {
        let arrow = Rc::new(Self {
            weight: RefCell::new(W::default()),
            edge_type: EdgeType::Arrow,
            id: meta.new_edge_id(),
            selfie: RefCell::new(Weak::new()),
            alpha: EdgeEnd::Weak(Rc::downgrade(&tail)),
            omega: EdgeEnd::Strong(head),
        });
        let selfie = Rc::downgrade(&arrow);
        *arrow.selfie.borrow_mut() = selfie;
        arrow
    }
    fn head_node(&self) -> Option<Rc<N>> {
        match self.edge_type {
            EdgeType::Arrow => self.omega.try_node(),
            _ => None,
        }
    }
    fn tail_node(&self) -> Option<Rc<N>> {
        match self.edge_type {
            EdgeType::Arrow => self.alpha.try_node(),
            _ => None,
        }
    }
}


// In a Link edge (undirected edge) no node has ownership over the other
// To prevent reference cycles (see https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)
// new_link must use weak links
impl<N: Node<V, Self, H>, V: 'static, H: HeadNodeCache<N, V, Self>, W: Default> EdgeLink<N, V, H>
    for BaseEdge<N, V, H, W>
{
    fn new_link(left: Rc<N>, right: Rc<N>, meta: Rc<H>) -> Rc<Self> {
        let link = Rc::new(Self {
            weight: RefCell::new(W::default()),
            edge_type: EdgeType::Link,
            id: meta.new_edge_id(),
            selfie: RefCell::new(Weak::new()),
            alpha: EdgeEnd::Weak(Rc::downgrade(&left)),
            omega: EdgeEnd::Weak(Rc::downgrade(&right)),
        });
        let selfie = Rc::downgrade(&link);
        *link.selfie.borrow_mut() = selfie;
        link
    }
    fn right_node(&self) -> Option<Rc<N>> {
        match self.edge_type {
            EdgeType::Link => self.omega.try_node(),
            _ => None,
        }
    }
    fn left_node(&self) -> Option<Rc<N>> {
        match self.edge_type {
            EdgeType::Link => self.alpha.try_node(),
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
            edge_type: EdgeType::Loop,
            id: meta.new_edge_id(),
            selfie: RefCell::new(Weak::new()),
            alpha: EdgeEnd::None,
            omega: EdgeEnd::Weak(Rc::downgrade(&node)),
        });
        let selfie = Rc::downgrade(&loop_node);
        *loop_node.selfie.borrow_mut() = selfie;
        loop_node
    }
    fn loop_node(&self) -> Option<Rc<N>> {
        match self.edge_type {
            EdgeType::Link => self.omega.try_node(),
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
