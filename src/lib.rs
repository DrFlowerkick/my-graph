//!lib.rs

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub trait MetaData {
    fn init() -> Self;
    fn next_node_id(&mut self) -> usize;
}

pub trait NodeCache: MetaData {
    fn cache_node<V, E: Edge, M: MetaData>(&mut self, node: Rc<BaseNode<V, E, M>>);
    fn get_node_cache<V, E: Edge, M: MetaData>(&self) -> Vec<Rc<BaseNode<V, E, M>>>;
}

// At the moment, rustc 1.74.1, impl Trait as result of trait fn() is not possible
// As soon as this is possible, this feature will be added
/*pub trait EdgeCache: MetaData {
    fn cache_edge(&mut self, edge: impl Edge);
    fn get_edge_cache(&self) -> Vec<impl Edge>;
}*/

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

pub trait Edge {
    // if node with node_id points in this edge toward a linked node, return linked node (head)
    fn head_node<V, E: Edge, M: MetaData>(&self, node_id: usize)
        -> Option<Rc<BaseNode<V, E, M>>>;
    // if a node points in this edge toward node with node_id, return linked node (tail)
    fn tail_node<V, E: Edge, M: MetaData>(&self, node_id: usize)
        -> Option<Rc<BaseNode<V, E, M>>>;
    fn edge_end(&self, node_id: usize) -> EdgePos;
}

pub trait DirectedEdge: Edge {
    fn new_arrow<V, E: Edge, M: MetaData>(
        tail: Rc<BaseNode<V, E, M>>,
        head: Rc<BaseNode<V, E, M>>,
    ) -> Rc<Self>;
}

pub trait UndirectedEdge: Edge {
    fn new_link<V, E: Edge, M: MetaData>(
        left: Rc<BaseNode<V, E, M>>,
        right: Rc<BaseNode<V, E, M>>,
    ) -> Rc<Self>;
}

pub trait LoopEdge: Edge {
    fn new_loop<V, E: Edge, M: MetaData>(node: Rc<BaseNode<V, E, M>>) -> Rc<Self>;
}

pub trait WeightedEdge<W>: Edge {
    fn set_weight(&mut self, value: W) -> &W;
    fn get_weight(&self) -> &W;
}

// BaseNodes are always used inside a Rc Ref
// node is used to provide a proper link on the node itself
pub struct BaseNode<V, E: Edge, M: MetaData> {
    value: RefCell<V>,
    id: usize,
    selfie: RefCell<Weak<BaseNode<V, E, M>>>,
    meta: Rc<RefCell<M>>,
    edges: RefCell<Vec<Rc<E>>>,
}

impl<V, E: Edge, M: MetaData> BaseNode<V, E, M> {
    pub fn alpha_node(value: V) -> Rc<BaseNode<V, E, M>> {
        let meta = Rc::new(RefCell::new(M::init()));
        Self::new(value, meta)
    }
    pub fn new(value: V, meta: Rc<RefCell<M>>) -> Rc<BaseNode<V, E, M>> {
        let node = Rc::new(BaseNode {
            value: RefCell::new(value),
            id: meta.borrow_mut().next_node_id(),
            selfie: RefCell::new(Weak::new()),
            meta: meta.clone(),
            edges: RefCell::new(Vec::new()),
        });
        let selfie = Rc::downgrade(&node);
        *node.selfie.borrow_mut() = selfie;
        node
    }
    pub fn get_value(&self) -> std::cell::Ref<'_, V> {
        self.value.borrow()
    }
    pub fn get_id(&self) -> usize {
        self.id
    }
    pub fn get_self(&self) -> Option<Rc<BaseNode<V, E, M>>> {
        match self.selfie.borrow().upgrade() {
            Some(ref node) => Some(node.clone()),
            None => None,
        }
    }
    pub fn add_edge(&self, edge: Rc<E>) {
        self.edges.borrow_mut().push(edge.clone());
    }
    pub fn get_heads(&self) -> Vec<Rc<BaseNode<V, E, M>>> {
        self.edges
            .borrow()
            .iter()
            .filter_map(|e| e.head_node(self.id))
            .collect::<Vec<Rc<BaseNode<V, E, M>>>>()
    }
    pub fn get_tails(&self) -> Vec<Rc<BaseNode<V, E, M>>> {
        self.edges
            .borrow()
            .iter()
            .filter_map(|e| e.tail_node(self.id))
            .collect::<Vec<Rc<BaseNode<V, E, M>>>>()
    }
}
