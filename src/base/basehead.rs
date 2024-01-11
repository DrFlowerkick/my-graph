//!basehead.rs

use crate::core::{Edge, Head, HeadEdgeCache, HeadNodeCache, Node};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct BaseHead<N: Node<V, E, Self>, V: 'static, E: Edge<N, V, Self>> {
    node_cache: RefCell<Vec<Rc<N>>>,
    edge_cache: RefCell<Vec<Rc<E>>>,
    _v: PhantomData<V>,
}

impl<N: Node<V, E, Self>, V: 'static, E: Edge<N, V, Self>> Head for BaseHead<N, V, E> {
    fn new() -> Rc<Self> {
        Rc::new(BaseHead {
            node_cache: RefCell::new(Vec::new()),
            edge_cache: RefCell::new(Vec::new()),
            _v: PhantomData,
        })
    }
}

impl<N: Node<V, E, Self>, V: 'static, E: Edge<N, V, Self>> HeadNodeCache<N, V, E>
    for BaseHead<N, V, E>
{
    fn cache_node(&self, node: Rc<N>) {
        self.node_cache.borrow_mut().push(node);
    }
    fn get_node_cache(&self) -> std::cell::Ref<'_, Vec<Rc<N>>> {
        self.node_cache.borrow()
    }
}

impl<N: Node<V, E, Self>, V: 'static, E: Edge<N, V, Self>> HeadEdgeCache<N, V, E>
    for BaseHead<N, V, E>
{
    fn cache_edge(&self, edge: Rc<E>) {
        self.edge_cache.borrow_mut().push(edge);
    }
    fn get_edge_cache(&self) -> std::cell::Ref<'_, Vec<Rc<E>>> {
        self.edge_cache.borrow()
    }
}
