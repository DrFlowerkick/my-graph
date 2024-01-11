//!mod.rs (core)

pub mod iterators;

use iterators::IterEdges;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub enum HeadUniqueIDAction {
    Load,
    Increment,
}

pub trait Head: Sized + 'static {
    fn new() -> Rc<Self>;
    fn new_node_id(&self) -> usize {
        self.unique_node_id(HeadUniqueIDAction::Increment)
    }
    fn last_node_id(&self) -> usize {
        self.unique_node_id(HeadUniqueIDAction::Load)
    }
    fn new_edge_id(&self) -> usize {
        self.unique_edge_id(HeadUniqueIDAction::Increment)
    }
    fn last_edge_id(&self) -> usize {
        self.unique_edge_id(HeadUniqueIDAction::Load)
    }
    fn unique_node_id(&self, action: HeadUniqueIDAction) -> usize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        match action {
            HeadUniqueIDAction::Load => COUNTER.load(Ordering::Relaxed),
            HeadUniqueIDAction::Increment => COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
    fn unique_edge_id(&self, action: HeadUniqueIDAction) -> usize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        match action {
            HeadUniqueIDAction::Load => COUNTER.load(Ordering::Relaxed),
            HeadUniqueIDAction::Increment => COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

pub trait HeadNodeCache<N: Node<V, E, Self>, V: 'static, E: Edge<N, V, Self>>: Head {
    fn cache_node(&self, node: Rc<N>);
    fn get_node_cache(&self) -> std::cell::Ref<'_, Vec<Rc<N>>>;
}

pub trait HeadEdgeCache<N: Node<V, E, Self>, V: 'static, E: Edge<N, V, Self>>:
    HeadNodeCache<N, V, E>
{
    fn cache_edge(&self, edge: Rc<E>);
    fn get_edge_cache(&self) -> std::cell::Ref<'_, Vec<Rc<E>>>;
}

#[derive(Debug, Clone, Copy)]
pub enum EdgeType {
    Arrow,
    Link,
    Loop,
}

pub trait Edge<N: Node<V, Self, H>, V: 'static, H: Head>: Sized + 'static {
    fn get_id(&self) -> usize;
    fn get_self(&self) -> Option<Rc<Self>>;
    fn get_edge_type(&self) -> EdgeType;
    // if node with node_id points inside this edge toward another node, return node pointed to (head)
    fn try_head_node(&self, node_id: usize) -> Option<Rc<N>>;
    // if a node of this edge points toward node with node_id, return node pointed from (tail)
    fn try_tail_node(&self, node_id: usize) -> Option<Rc<N>>;
}

// Directed edge
pub trait EdgeArrow<N: Node<V, Self, H>, V: 'static, H: Head>: Edge<N, V, H> {
    fn new_arrow(tail: Rc<N>, head: Rc<N>, meta: Rc<H>) -> Rc<Self>;
    fn head_node(&self) -> Option<Rc<N>>;
    fn tail_node(&self) -> Option<Rc<N>>;
}

// Undirected edge
pub trait EdgeLink<N: Node<V, Self, H>, V: 'static, H: HeadNodeCache<N, V, Self>>:
    Edge<N, V, H>
{
    fn new_link(left: Rc<N>, right: Rc<N>, meta: Rc<H>) -> Rc<Self>;
    fn left_node(&self) -> Option<Rc<N>>;
    fn right_node(&self) -> Option<Rc<N>>;
}

// Looping Edge
pub trait EdgeLoop<N: Node<V, Self, H>, V: 'static, H: Head>: Edge<N, V, H> {
    fn new_loop(node: Rc<N>, meta: Rc<H>) -> Rc<Self>;
    fn loop_node(&self) -> Option<Rc<N>>;
}

// Edge with a value
pub trait EdgeWeighted<N: Node<V, Self, H>, V: 'static, H: Head, W: Default + 'static>:
    Edge<N, V, H>
{
    fn get_weight(&self) -> std::cell::Ref<'_, W>;
    fn get_weight_mut(&self) -> std::cell::RefMut<'_, W>;
}

pub trait Node<V: 'static, E: Edge<Self, V, H>, H: Head>: Sized + 'static {
    fn new(value: V, meta: Rc<H>) -> Rc<Self>;
    fn get_value(&self) -> std::cell::Ref<'_, V>;
    fn get_value_mut(&self) -> std::cell::RefMut<'_, V>;
    fn get_id(&self) -> usize;
    fn get_self(&self) -> Option<Rc<Self>>;
    fn add_edge(&self, edge: Rc<E>) -> Rc<E>;
    fn len_edges(&self) -> usize;
    fn get_edge(&self, index: usize) -> Option<Rc<E>> {
        self.iter_edges().nth(index)
    }
    fn get_edge_by_id(&self, id: usize) -> Option<Rc<E>> {
        self.iter_edges().find(|e| e.get_id() == id)
    }
    fn iter_edges(&self) -> Box<dyn Iterator<Item = Rc<E>>> {
        Box::new(IterEdges::<Self, V, E, H>::new(self.get_self().unwrap()))
    }
    fn iter_heads(&self) -> Box<dyn Iterator<Item = Rc<Self>> + '_> {
        Box::new(
            self.iter_edges()
                .filter_map(|e| e.try_head_node(self.get_id())),
        )
    }
    fn iter_tails(&self) -> Box<dyn Iterator<Item = Rc<Self>> + '_> {
        Box::new(
            self.iter_edges()
                .filter_map(|e| e.try_tail_node(self.get_id())),
        )
    }
}
