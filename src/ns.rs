use derivative::Derivative;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub type RF = Rc<dyn Fn(usize, usize)>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Name(
    RefCell<Rc<Cell<usize>>>,
    #[derivative(Debug = "ignore")] Option<RF>,
);

impl Name {
    pub fn id(&self) -> usize {
        self.0.borrow().get()
    }

    pub fn unify(&self, other: &Name) {
        if let Some(ref f) = self.1 {
            f(self.0.borrow().get(), other.0.borrow().get());
        }
        *self.0.borrow_mut() = other.0.borrow().clone();
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct LinearNamespace {
    next: usize,
    grants: Vec<Weak<Cell<usize>>>,
    #[derivative(Debug = "ignore")]
    reorder_fn: Option<RF>,
}

impl LinearNamespace {
    pub fn new() -> LinearNamespace {
        LinearNamespace {
            next: 0,
            grants: Vec::new(),
            reorder_fn: None,
        }
    }

    pub fn set_reorder_fn(&mut self, reorder_fn: Option<RF>) {
        self.reorder_fn = reorder_fn;
    }

    pub fn next(&mut self) -> Name {
        let nm = Rc::new(Cell::new(self.next));
        self.next += 1;
        self.grants.push(Rc::downgrade(&nm));
        Name(RefCell::new(nm), self.reorder_fn.clone())
    }

    pub fn linearize(&mut self) -> usize {
        let new_grants = self
            .grants
            .iter()
            .map(Weak::upgrade)
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect::<Vec<_>>();
        for (idx, nm) in new_grants.iter().enumerate() {
            if let Some(ref f) = self.reorder_fn {
                f(nm.get(), idx);
            }
            nm.set(idx);
        }
        self.grants = new_grants.iter().map(Rc::downgrade).collect();
        self.next = self.grants.len();
        self.next
    }

    pub fn names(&self) -> Vec<Name> {
        self.grants
            .iter()
            .map(Weak::upgrade)
            .filter(Option::is_some)
            .map(Option::unwrap)
            .map(|x| Name(RefCell::new(x), self.reorder_fn.clone()))
            .collect()
    }
}
