//!mod.rs (core)

pub mod iterators;

use iterators::IterLinks;
use std::rc::Rc;

pub trait LinkBuilder<L, N, W: Default> {
    fn new() -> Self;
    fn set_weight(self, weight: W) -> Self;
    fn set_alpha_weak(self, node: Rc<N>) -> Self;
    fn set_alpha_strong(self, node: Rc<N>) -> Self;
    fn set_omega_weak(self, node: Rc<N>) -> Self;
    fn set_omega_strong(self, node: Rc<N>) -> Self;
    fn build(self) -> Rc<L>;
}

pub trait Link<N: Node<V, Self, W>, V: 'static, W: Default + 'static>: Sized + 'static {
    fn get_id(&self) -> usize;
    fn get_self(&self) -> Option<Rc<Self>>;
    // if node with node_id points inside this link toward another node, return node pointed to
    fn try_node(&self, node_id: usize) -> Option<Rc<N>>;
    // if node with node_id points inside this link toward a strong node, return strong node pointed to
    fn try_strong_node(&self, node_id: usize) -> Option<Rc<N>>;
    // if node with node_id points inside this link toward a weak node, return weak node pointed to
    fn try_weak_node(&self, node_id: usize) -> Option<Rc<N>>;
    fn get_weight(&self) -> std::cell::Ref<'_, W>;
    fn get_weight_mut(&self) -> std::cell::RefMut<'_, W>;
}

pub trait Node<V: 'static, L: Link<Self, V, W>, W: Default + 'static>: Sized + 'static {
    fn new(value: V) -> Rc<Self>;
    fn get_value(&self) -> std::cell::Ref<'_, V>;
    fn get_value_mut(&self) -> std::cell::RefMut<'_, V>;
    fn get_id(&self) -> usize;
    fn get_self(&self) -> Option<Rc<Self>>;
    fn add_link(&self, link: Rc<L>) -> Rc<L>;
    fn len_links(&self) -> usize;
    fn get_link(&self, index: usize) -> Option<Rc<L>> {
        self.iter_links().nth(index)
    }
    fn get_link_by_id(&self, id: usize) -> Option<Rc<L>> {
        self.iter_links().find(|e| e.get_id() == id)
    }
    fn iter_links(&self) -> Box<dyn Iterator<Item = Rc<L>>> {
        Box::new(IterLinks::<Self, V, L, W>::new(self.get_self().unwrap()))
    }
    fn iter_nodes(&self) -> Box<dyn Iterator<Item = Rc<Self>> + '_> {
        Box::new(self.iter_links().filter_map(|e| e.try_node(self.get_id())))
    }
    fn iter_strong_nodes(&self) -> Box<dyn Iterator<Item = Rc<Self>> + '_> {
        Box::new(
            self.iter_links()
                .filter_map(|e| e.try_strong_node(self.get_id())),
        )
    }
    fn iter_weak_nodes(&self) -> Box<dyn Iterator<Item = Rc<Self>> + '_> {
        Box::new(
            self.iter_links()
                .filter_map(|e| e.try_weak_node(self.get_id())),
        )
    }
}
