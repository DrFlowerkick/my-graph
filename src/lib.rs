//!lib.rs

use std::any::Any;
use std::marker::PhantomData;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub trait MetaData {
    fn new() -> Rc<RefCell<Self>>;
    fn unique_id(&mut self) -> usize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}

pub trait MetaNodeCache<V: 'static, E: Edge, M: MetaData + 'static>: MetaData {
    fn cache_node(&mut self, node: Rc<impl Node<V, E, M>>);
    fn get_node_cache(&self) -> Vec<Rc<dyn Any>>;
}

pub trait MetaEdgeCache: MetaData {
    fn cache_edge(&mut self, edge: Rc<impl Edge>);
    fn get_edge_cache(&self) -> Vec<Rc<dyn Any>>;
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

pub trait Edge: Sized + 'static {
    fn cast_edge(edge: Rc<dyn Any>) -> Rc<Self>;
    // REQUIRED?
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_id(&self) -> usize;
    fn get_self(&self) -> Rc<Self>;
    // if node with node_id points in this edge toward a linked node, return linked node (head)
    fn try_head_node(&self, node_id: usize) -> Option<Rc<dyn Any>>;
    // if a node points in this edge toward node with node_id, return linked node (tail)
    fn try_tail_node(&self, node_id: usize) -> Option<Rc<dyn Any>>;
    fn edge_end(&self, node_id: usize) -> EdgePos;
}

// Directed edge
pub trait EdgeArrow<V: 'static, E: Edge, M: MetaData + 'static>: Edge {
    fn try_arrow(tail: Rc<impl Node<V, E, M>>, head: Rc<impl Node<V, E, M>>) -> Option<Rc<Self>> {
        // first check of both nodes share same metadata
        if ptr::eq(tail.get_meta().as_ref(), head.get_meta().as_ref()) {
            return Some(Self::new_arrow(tail, head));
        }
        None
    }
    // use metadata of Node to generate unique ID for new arrow
    fn new_arrow(tail: Rc<impl Node<V, E, M>>, head: Rc<impl Node<V, E, M>>) -> Rc<Self>;
    fn head_node(&self) -> Rc<dyn Any>;
    fn tail_node(&self) -> Rc<dyn Any>;
}

// Undirected edge
pub trait EdgeLink<V: 'static, E: Edge, M: MetaData + 'static>: Edge {
    fn try_link(tail: Rc<impl Node<V, E, M>>, head: Rc<impl Node<V, E, M>>) -> Option<Rc<Self>> {
        // first check of both nodes share same metadata
        if ptr::eq(tail.get_meta().as_ref(), head.get_meta().as_ref()) {
            return Some(Self::new_link(tail, head));
        }
        None
    }
    // use metadata of Node to generate unique ID for new link
    fn new_link(left: Rc<impl Node<V, E, M>>, right: Rc<impl Node<V, E, M>>) -> Rc<Self>;
    fn left_node(&self) -> Rc<dyn Any>;
    fn right_node(&self) -> Rc<dyn Any>;
}

// Looping Edge
pub trait EdgeLoop<V: 'static, E: Edge, M: MetaData + 'static>: Edge {
    // use metadata of Node to generate unique ID for new loop
    fn new_loop(node: Rc<impl Node<V, E, M>>) -> Rc<Self>;
    fn loop_node(&self) -> Rc<dyn Any>;
}

// Edge with a value
pub trait EdgeWeighted<W>: Edge {
    fn set_weight(&mut self, value: W) -> &W;
    fn get_weight(&self) -> &W;
}

pub trait EdgeCache<M: MetaEdgeCache> {
    fn cache_edge(&self, meta: Rc<RefCell<M>>);
}

pub trait Node<V: 'static, E: Edge, M: MetaData + 'static>: Sized + 'static {
    fn cast_node(node: Rc<dyn Any>) -> Rc<Self>;
    // REQUIRED?
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn alpha_node(value: V) -> Rc<Self> {
        let meta = M::new();
        Self::new(value, meta)
    }
    fn new(value: V, meta: Rc<RefCell<M>>) -> Rc<Self>;
    fn get_value(&self) -> std::cell::Ref<'_, V>;
    fn get_value_mut(&self) -> std::cell::RefMut<'_, V>;
    fn get_id(&self) -> usize;
    fn get_meta(&self) -> Rc<RefCell<M>>;
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
                .filter_map(|e| e.try_head_node(self.get_id()))
                .map(|n| Self::cast_node(n)),
        )
    }
    fn iter_tails(&self) -> Box<dyn Iterator<Item = Rc<Self>> + '_> {
        Box::new(
            self.iter_edges()
                .filter_map(|e| e.try_tail_node(self.get_id()))
                .map(|n| Self::cast_node(n)),
        )
    }
}

pub trait NodeCache<V: 'static, E: Edge, M: MetaData + MetaNodeCache<V, E, M> + 'static>:
    Node<V, E, M>
{
    fn cache_node(&self, meta: Rc<RefCell<M>>);
    fn get_node_cache(&self, meta: Rc<RefCell<M>>) -> Vec<Rc<Self>>;
}

pub trait NodeEdgeCache<M: MetaEdgeCache> {
    fn get_edge_cache(&self, meta: Rc<RefCell<M>>) -> Vec<Rc<dyn Any>>;
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

impl<V: 'static, E: Edge + 'static, M: MetaData + 'static> Node<V, E, M> for BaseNode<V, E, M> {
    fn cast_node(node: Rc<dyn Any>) -> Rc<Self> {
        match node.downcast::<Self>() {
            Ok(node) => node,
            Err(_) => panic!("node is not of type BaseNode"),
        }
    }
    fn new(value: V, meta: Rc<RefCell<M>>) -> Rc<BaseNode<V, E, M>> {
        let node = Rc::new(BaseNode {
            value: RefCell::new(value),
            id: meta.borrow_mut().unique_id(),
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
    fn get_meta(&self) -> Rc<RefCell<M>> {
        self.meta.clone()
    }
    fn get_self(&self) -> Option<Rc<BaseNode<V, E, M>>> {
        match self.selfie.borrow().upgrade() {
            Some(ref node) => Some(node.clone()),
            None => None,
        }
    }
    fn add_edge(&self, edge: Rc<E>) -> Rc<E> {
        self.edges.borrow_mut().push(edge.clone());
        edge
    }
    fn len_edges(&self) -> usize {
        self.edges.borrow().len()
    }
}

pub struct IterEdges<N: Node<V, E, M>, V: 'static, E: Edge, M: MetaData + 'static> {
    node: Rc<N>,
    edge_index: usize,
    _v: PhantomData<V>,
    _e: PhantomData<E>,
    _m: PhantomData<M>,
    finished: bool, // true if iterator finished
}

impl<'a, N: Node<V, E, M>, V, E: Edge, M: MetaData> IterEdges<N, V, E, M> {
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

impl<'a, N: Node<V, E, M>, V, E: Edge, M: MetaData> Iterator for IterEdges<N, V, E, M> {
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

impl<'a, N: Node<V, E, M>, V, E: Edge, M: MetaData> ExactSizeIterator for IterEdges<N, V, E, M> {
    fn len(&self) -> usize {
        self.node.len_edges()
    }
}
