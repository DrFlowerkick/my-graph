//!basenode.rs


use crate::core::{Node, Edge, Head};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};


// BaseNodes are always used inside a Rc Ref
// selfie is used to provide a proper link on the node itself
pub struct BaseNode<V: 'static, E: Edge<Self, V, H>, H: Head> {
    value: RefCell<V>,
    id: usize,
    selfie: RefCell<Weak<BaseNode<V, E, H>>>,
    edges: RefCell<Vec<Rc<E>>>,
}

impl<V: 'static, E: Edge<Self, V, H>, H: Head> Node<V, E, H> for BaseNode<V, E, H> {
    fn new(value: V, meta: Rc<H>) -> Rc<BaseNode<V, E, H>> {
        let node = Rc::new(BaseNode {
            value: RefCell::new(value),
            id: meta.new_node_id(),
            selfie: RefCell::new(Weak::new()),
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
    fn get_self(&self) -> Option<Rc<BaseNode<V, E, H>>> {
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

