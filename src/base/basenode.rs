//!basenode.rs

use crate::core::{Link, Node};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

// BaseNodes are always used inside a Rc Ref
// selfie is used to provide a proper link on the node itself
pub struct BaseNode<V: 'static, L: Link<Self, V, W>, W: Default + 'static> {
    value: RefCell<V>,
    id: usize,
    selfie: RefCell<Weak<BaseNode<V, L, W>>>,
    links: RefCell<Vec<Rc<L>>>,
}

impl<V: 'static, L: Link<Self, V, W>, W: Default + 'static> Node<V, L, W> for BaseNode<V, L, W> {
    fn new(value: V) -> Rc<BaseNode<V, L, W>> {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let node = Rc::new(BaseNode {
            value: RefCell::new(value),
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
            selfie: RefCell::new(Weak::new()),
            links: RefCell::new(Vec::new()),
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
    fn get_self(&self) -> Option<Rc<BaseNode<V, L, W>>> {
        self.selfie.borrow().upgrade().as_ref().cloned()
    }
    fn add_link(&self, edge: Rc<L>) -> Rc<L> {
        self.links.borrow_mut().push(edge.clone());
        edge
    }
    fn len_links(&self) -> usize {
        self.links.borrow().len()
    }
}
