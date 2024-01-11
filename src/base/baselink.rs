//!baseedge.rs

use crate::core::{Link, LinkBuilder, Node};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

enum LinkEnd<N> {
    None,
    Weak(Weak<N>),
    Strong(Rc<N>),
}

impl<N> LinkEnd<N> {
    fn try_node(&self) -> Option<Rc<N>> {
        match self {
            LinkEnd::None => None,
            LinkEnd::Weak(weak) => weak.upgrade().as_ref().cloned(),
            LinkEnd::Strong(node) => Some(node.clone()),
        }
    }
    fn weak_node(&self) -> Option<Rc<N>> {
        match self {
            LinkEnd::Weak(weak) => weak.upgrade().as_ref().cloned(),
            _ => None,
        }
    }
    fn strong_node(&self) -> Option<Rc<N>> {
        match self {
            LinkEnd::Strong(node) => Some(node.clone()),
            _ => None,
        }
    }
}

pub struct BaseLinkBuilder<N, W: Default> {
    weight: Option<W>,
    id: usize,
    alpha: LinkEnd<N>,
    omega: LinkEnd<N>,
}

impl<N: Node<V, BaseLink<N, V, W>, W>, V, W: Default> LinkBuilder<BaseLink<N, V, W>, N, W>
    for BaseLinkBuilder<N, W>
{
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self {
            weight: None,
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
            alpha: LinkEnd::None,
            omega: LinkEnd::None,
        }
    }
    fn set_alpha_strong(mut self, node: Rc<N>) -> Self {
        self.alpha = LinkEnd::Strong(node);
        self
    }
    fn set_alpha_weak(mut self, node: Rc<N>) -> Self {
        self.alpha = LinkEnd::Weak(Rc::downgrade(&node));
        self
    }
    fn set_omega_strong(mut self, node: Rc<N>) -> Self {
        self.omega = LinkEnd::Strong(node);
        self
    }
    fn set_omega_weak(mut self, node: Rc<N>) -> Self {
        self.omega = LinkEnd::Weak(Rc::downgrade(&node));
        self
    }
    fn set_weight(mut self, weight: W) -> Self {
        self.weight = Some(weight);
        self
    }
    fn build(self) -> Rc<BaseLink<N, V, W>> {
        let link = Rc::new(BaseLink {
            weight: RefCell::new(self.weight.unwrap_or_default()),
            id: self.id,
            selfie: RefCell::new(Weak::new()),
            alpha: self.alpha,
            omega: self.omega,
        });
        let selfie = Rc::downgrade(&link);
        *link.selfie.borrow_mut() = selfie;
        link
    }
}

pub struct BaseLink<N: Node<V, Self, W>, V: 'static, W: Default + 'static> {
    weight: RefCell<W>,
    id: usize,
    selfie: RefCell<Weak<BaseLink<N, V, W>>>,
    alpha: LinkEnd<N>,
    omega: LinkEnd<N>,
}

impl<N: Node<V, Self, W>, V: 'static, W: Default + 'static> Link<N, V, W> for BaseLink<N, V, W> {
    fn get_id(&self) -> usize {
        self.id
    }
    fn get_self(&self) -> Option<Rc<Self>> {
        self.selfie.borrow().upgrade().as_ref().cloned()
    }
    // if node with node_id points inside this link toward another node, return node pointed to
    fn try_node(&self, node_id: usize) -> Option<Rc<N>> {
        if let Some(node) = self.alpha.try_node() {
            if node.get_id() == node_id {
                return self.omega.try_node();
            }
        }
        if let Some(node) = self.omega.try_node() {
            if node.get_id() == node_id {
                return self.alpha.try_node();
            }
        }
        None
    }
    // if node with node_id points inside this link toward a strong node, return strong node pointed to
    fn try_strong_node(&self, node_id: usize) -> Option<Rc<N>> {
        if let Some(node) = self.alpha.try_node() {
            if node.get_id() == node_id {
                return self.omega.strong_node();
            }
        }
        if let Some(node) = self.omega.try_node() {
            if node.get_id() == node_id {
                return self.alpha.strong_node();
            }
        }
        None
    }
    // if node with node_id points inside this link toward a weak node, return weak node pointed to
    fn try_weak_node(&self, node_id: usize) -> Option<Rc<N>> {
        if let Some(node) = self.alpha.try_node() {
            if node.get_id() == node_id {
                return self.omega.weak_node();
            }
        }
        if let Some(node) = self.omega.try_node() {
            if node.get_id() == node_id {
                return self.alpha.weak_node();
            }
        }
        None
    }
    fn get_weight(&self) -> std::cell::Ref<'_, W> {
        self.weight.borrow()
    }
    fn get_weight_mut(&self) -> std::cell::RefMut<'_, W> {
        self.weight.borrow_mut()
    }
}
