use std::rc::{Weak, Rc};
use std::cell::Cell;

pub trait ReorderFn: FnMut(usize, usize) {}
pub type ReorderFnP = fn(&mut self, usize, usize);
pub type LinearNamespaceFnP = LinearNamespace<ReorderFnP>;
pub type NameFnP = Name<ReorderFnP>;

#[derive(Debug)]
pub struct LinearNamespace<R: ReorderFn> {
    next: usize,
    grants: Vec<Weak<Cell<usize>>>,
    reorder_fn: Option<R>,
}

#[derive(Debug)]
pub struct Name<R: ReorderFn>(Rc<Cell<usize>>, Option<R>);

impl<R: ReorderFn> LinearNamespace<R> {
    pub fn new() -> LinearNamespace<R> {
        LinearNamespace {
            next: 0,
            grants: Vec::new(),
            reorder_fn: None,
        }
    }

    pub fn set_reorder_fn(&mut self, reorder_fn: Option<R>) {
        self.reorder_fn = reorder_fn;
    }

    pub fn next(&mut self) -> Name<R> {
        let nm = Rc::new(Cell::new(self.next));
        self.next += 1;
        self.grants.push(Rc::downgrade(&nm));
        Name(nm, self.reorder_fn)
    }

    pub fn linearize(&mut self) {
        let new_grants = self.grants.iter().map(Weak::upgrade).filter(Option::is_some).map(Option::unwrap).collect::<Vec<_>>();
        for (idx, nm) in new_grants.iter().enumerate() {
            if let Some(f) = self.reorder_fn {
                f(nm.get(), idx);
            }
            nm.set(idx);
        }
        self.grants = new_grants.iter().map(Rc::downgrade).collect();
    }

    pub fn names(&self) -> Vec<Name<R>> {
        self.grants.iter().map(Weak::upgrade).filter(Option::is_some).map(Option::unwrap).map(|x| Name(x, self.reorder_fn)).collect()
    }
}

impl<R: ReorderFn> Name<R> {
    pub fn id(&self) -> usize {
        self.0.get()
    }

    pub fn unify(&mut self, other: &Name<R>) {
        if let Some(f) = self.1 {
            f(self.0.get(), other.0.get());
        }
        self.0 = Rc::clone(&other.0);
    }
}
