//!lib.rs

// BIG TODO NOW: seperate code in different modules to clean up and make code better readable

use private_meta::*;
use std::marker::PhantomData;
use std::ptr;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

mod private_meta {
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub enum MetaUniqueIDAction {
        Load,
        Increment,
    }

    pub trait MetaUniqueID {
        fn unique_node_id(&self, action: MetaUniqueIDAction) -> usize {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            match action {
                MetaUniqueIDAction::Load => COUNTER.load(Ordering::Relaxed),
                MetaUniqueIDAction::Increment => COUNTER.fetch_add(1, Ordering::Relaxed),
            }
        }
        fn unique_edge_id(&self, action: MetaUniqueIDAction) -> usize {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            match action {
                MetaUniqueIDAction::Load => COUNTER.load(Ordering::Relaxed),
                MetaUniqueIDAction::Increment => COUNTER.fetch_add(1, Ordering::Relaxed),
            }
        }
    }
}

pub trait MetaData: Sized + 'static + MetaUniqueID {
    fn new() -> Rc<Self>;
    fn new_node_id(&self) -> usize {
        self.unique_node_id(MetaUniqueIDAction::Increment)
    }
    fn last_node_id(&self) -> usize {
        self.unique_node_id(MetaUniqueIDAction::Load)
    }
    fn new_edge_id(&self) -> usize {
        self.unique_edge_id(MetaUniqueIDAction::Increment)
    }
    fn last_edge_id(&self) -> usize {
        self.unique_edge_id(MetaUniqueIDAction::Load)
    }
}

pub trait MetaNodeCache<N: Node<V, E, Self>, V: 'static, E: Edge<N, V, Self>>: MetaData {
    fn cache_node(&self, node: Rc<N>);
    fn get_node_cache(&self) -> Vec<Rc<N>>;
}

pub trait MetaEdgeCache<N: Node<V, E, Self>, V: 'static, E: Edge<N, V, Self>>:
    MetaNodeCache<N, V, E>
{
    fn cache_edge(&self, edge: Rc<E>);
    fn get_edge_cache(&self) -> Vec<Rc<E>>;
}

// EdgePos describes position of a Node (identified by it's id) in Edge
pub enum EdgePos {
    // Node is at Tail position: Edge points from Node to it's target
    Tail,
    // Node is at Head position: Edge points from it's source to Node
    Head,
    // undirected Edge: Node points to linked node and vice versa
    Link,
    // looped Edge: Node points to itself
    Loop,
    // Node is not part of Edge
    None,
}

pub trait Edge<N: Node<V, Self, M>, V: 'static, M: MetaData>: Sized + 'static {
    fn get_id(&self) -> usize;
    fn get_self(&self) -> Option<Rc<Self>>;
    // if node with node_id points in this edge toward a linked node, return linked node (head)
    fn try_head_node(&self, node_id: usize) -> Option<Rc<N>>;
    // if a node points in this edge toward node with node_id, return linked node (tail)
    fn try_tail_node(&self, node_id: usize) -> Option<Rc<N>>;
    fn edge_end(&self, node_id: usize) -> EdgePos;
}

// Directed edge
pub trait EdgeArrow<N: Node<V, Self, M>, V: 'static, M: MetaData>: Edge<N, V, M> {
    fn try_arrow(tail: Rc<N>, head: Rc<N>) -> Option<Rc<Self>> {
        // first check of both nodes share same metadata
        if ptr::eq(tail.get_meta().as_ref(), head.get_meta().as_ref()) {
            return Some(Self::new_arrow(tail, head));
        }
        None
    }
    // use metadata of Node to generate unique ID for new arrow
    fn new_arrow(tail: Rc<N>, head: Rc<N>) -> Rc<Self>;
    fn head_node(&self) -> Rc<N>;
    fn tail_node(&self) -> Rc<N>;
}

// Undirected edge
pub trait EdgeLink<N: Node<V, Self, M>, V: 'static, M: MetaData>: Edge<N, V, M> {
    fn try_link(tail: Rc<N>, head: Rc<N>) -> Option<Rc<Self>> {
        // first check of both nodes share same metadata
        if ptr::eq(tail.get_meta().as_ref(), head.get_meta().as_ref()) {
            return Some(Self::new_link(tail, head));
        }
        None
    }
    // use metadata of Node to generate unique ID for new link
    fn new_link(left: Rc<N>, right: Rc<N>) -> Rc<Self>;
    fn left_node(&self) -> Rc<N>;
    fn right_node(&self) -> Rc<N>;
}

// Looping Edge
pub trait EdgeLoop<N: Node<V, Self, M>, V: 'static, M: MetaData>: Edge<N, V, M> {
    // use metadata of Node to generate unique ID for new loop
    fn new_loop(node: Rc<N>) -> Rc<Self>;
    fn loop_node(&self) -> Rc<N>;
}

// Edge with a value
pub trait EdgeWeighted<N: Node<V, Self, M>, V: 'static, M: MetaData, W>: Edge<N, V, M> {
    fn set_weight(&mut self, value: W) -> &W;
    fn get_weight(&self) -> &W;
}

pub trait EdgeCache<N: Node<V, Self, M>, V: 'static, M: MetaEdgeCache<N, V, Self>>:
    Edge<N, V, M>
{
    fn cache_edge(&self, meta: Rc<M>) {
        meta.cache_edge(self.get_self().unwrap());
    }
    fn get_edge_cache(&self, meta: Rc<M>) -> Vec<Rc<Self>> {
        meta.get_edge_cache()
    }
}

pub trait Node<V: 'static, E: Edge<Self, V, M>, M: MetaData>: Sized + 'static {
    fn alpha_node(value: V) -> Rc<Self> {
        let meta = M::new();
        Self::new(value, meta)
    }
    fn new(value: V, meta: Rc<M>) -> Rc<Self>;
    fn get_value(&self) -> std::cell::Ref<'_, V>;
    fn get_value_mut(&self) -> std::cell::RefMut<'_, V>;
    fn get_id(&self) -> usize;
    fn get_meta(&self) -> Rc<M>;
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
        Box::new(IterEdges::<Self, V, E, M>::new(self.get_self().unwrap()))
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

pub trait NodeCache<V: 'static, E: Edge<Self, V, M>, M: MetaNodeCache<Self, V, E>>:
    Node<V, E, M>
{
    fn cache_node(&self, meta: Rc<M>) {
        meta.cache_node(self.get_self().unwrap());
    }
    fn get_node_cache(&self, meta: Rc<M>) -> Vec<Rc<Self>> {
        meta.get_node_cache()
    }
}

pub trait NodeEdgeCache<V: 'static, E: Edge<Self, V, M>, M: MetaEdgeCache<Self, V, E>>:
    NodeCache<V, E, M>
{
    fn get_edge_cache(&self, meta: Rc<M>) -> Vec<Rc<E>>;
}

// BaseNodes are always used inside a Rc Ref
// node is used to provide a proper link on the node itself
pub struct BaseNode<V: 'static, E: Edge<Self, V, M>, M: MetaData> {
    value: RefCell<V>,
    id: usize,
    selfie: RefCell<Weak<BaseNode<V, E, M>>>,
    meta: Rc<M>,
    edges: RefCell<Vec<Rc<E>>>,
}

impl<V: 'static, E: Edge<Self, V, M>, M: MetaData> Node<V, E, M> for BaseNode<V, E, M> {
    fn new(value: V, meta: Rc<M>) -> Rc<BaseNode<V, E, M>> {
        let node = Rc::new(BaseNode {
            value: RefCell::new(value),
            id: meta.new_node_id(),
            selfie: RefCell::new(Weak::new()),
            meta: meta.clone(),
            edges: RefCell::new(Vec::new()),
        });
        let selfie = Rc::downgrade(&node);
        *node.selfie.borrow_mut() = selfie;
        node
    }
    fn get_value(&self) -> std::cell::Ref<'_, V> {
        self.value.borrow()
    }
    fn get_value_mut(&self) -> std::cell::RefMut<'_, V> {
        self.value.borrow_mut()
    }
    fn get_id(&self) -> usize {
        self.id
    }
    fn get_meta(&self) -> Rc<M> {
        self.meta.clone()
    }
    fn get_self(&self) -> Option<Rc<BaseNode<V, E, M>>> {
        self.selfie.borrow().upgrade().as_ref().cloned()
    }
    fn add_edge(&self, edge: Rc<E>) -> Rc<E> {
        self.edges.borrow_mut().push(edge.clone());
        edge
    }
    fn len_edges(&self) -> usize {
        self.edges.borrow().len()
    }
}

pub struct IterEdges<N: Node<V, E, M>, V: 'static, E: Edge<N, V, M>, M: MetaData> {
    node: Rc<N>,
    edge_index: usize,
    _v: PhantomData<V>,
    _e: PhantomData<E>,
    _m: PhantomData<M>,
    finished: bool, // true if iterator finished
}

impl<N: Node<V, E, M>, V, E: Edge<N, V, M>, M: MetaData> IterEdges<N, V, E, M> {
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

impl<N: Node<V, E, M>, V, E: Edge<N, V, M>, M: MetaData> Iterator for IterEdges<N, V, E, M> {
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

impl<N: Node<V, E, M>, V, E: Edge<N, V, M>, M: MetaData> ExactSizeIterator
    for IterEdges<N, V, E, M>
{
    fn len(&self) -> usize {
        self.node.len_edges()
    }
}
