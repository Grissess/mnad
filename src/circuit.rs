use self::ns::*;
use self::solver::*;
use super::*;

use std::cell::{RefCell, Ref, RefMut};
use std::rc::{Rc, Weak};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitError {
    MatrixError(MatrixError),
    CircuitDead,
}

impl From<MatrixError> for CircuitError {
    fn from(v: MatrixError) -> CircuitError {
        CircuitError::MatrixError(v)
    }
}

#[derive(Debug, Clone)]
pub enum BipoleKind<S: Scalar> {
    Resistor(S),
    VoltageSource(S),
    CurrentSource(S),
}

#[derive(Debug)]
pub struct Pin(Option<Name>);

impl Pin {
    pub fn ground() -> Pin {
        Pin(None)
    }
    pub fn is_ground(&self) -> bool {
        self.0.is_none()
    }

    pub fn id(&self) -> Option<usize> {
        self.0.as_ref().map(Name::id)
    }

    pub fn connect(&mut self, other: &mut Pin) {
        match (&mut self.0, &other.0) {
            (Some(a), Some(b)) => a.unify(b),
            (Some(_), None) => self.0 = None,
            (None, Some(_)) => other.0 = None,
            (None, None) => (),
        }
    }
}

#[derive(Debug)]
pub struct Bipole<S: Scalar> {
    pos: Pin,
    neg: Pin,
    vsid: Option<Name>,
    kind: BipoleKind<S>,
    circuit: Weak<RefCell<Circuit<S>>>,
}

impl<S: Scalar> Bipole<S> {
    pub fn pos(&self) -> &Pin {
        &self.pos
    }
    pub fn neg(&self) -> &Pin {
        &self.neg
    }
    pub fn vsid(&self) -> Option<&Name> {
        self.vsid.as_ref()
    }
    pub fn kind(&self) -> &BipoleKind<S> {
        &self.kind
    }
    pub fn circuit(&self) -> Option<Rc<RefCell<Circuit<S>>>> {
        self.circuit.upgrade()
    }

    pub fn set_kind(&mut self, kind: BipoleKind<S>) -> Result<(), CircuitError> {
        let circuit_cell: Rc<RefCell<_>> = self.circuit().ok_or(CircuitError::CircuitDead)?;

        let mut circuit = circuit_cell.borrow_mut();

        circuit.repeal_effect(&self);

        if let BipoleKind::VoltageSource(_) = self.kind {
            match kind {
                BipoleKind::VoltageSource(_) => (),
                _ => self.vsid = None,
            }
        }

        self.kind = kind;

        if let BipoleKind::VoltageSource(_) = self.kind {
            match self.vsid {
                Some(_) => (),
                None => self.vsid = Some(circuit.alloc_vsid()),
            }
        }

        circuit.apply_effect(&self);

        Ok(())
    }
}

pub struct BipoleRef<S: Scalar>(pub Rc<RefCell<Bipole<S>>>);

impl<S: Scalar> BipoleRef<S> {
    pub fn borrow(&self) -> Ref<Bipole<S>> { self.0.borrow() }

    pub fn borrow_mut(&self) -> RefMut<Bipole<S>> { self.0.borrow_mut() }
}

#[derive(Debug)]
pub struct Circuit<S: Scalar> {
    bipoles: Vec<Rc<RefCell<Bipole<S>>>>,
    myself: Option<Rc<RefCell<Circuit<S>>>>,
    vsns: LinearNamespace,
    ndns: LinearNamespace,
    builder: MatrixBuilder<S>,
    eval: MatrixEvaluator<S>,
    need_lin: bool,
    need_build: bool,
}

#[derive(Debug)]
pub struct CircuitRef<S: Scalar>(pub Rc<RefCell<Circuit<S>>>);

impl<S: Scalar> CircuitRef<S> {
    pub fn borrow(&self) -> Ref<Circuit<S>> { self.0.borrow() }

    pub fn borrow_mut(&self) -> RefMut<Circuit<S>> { self.0.borrow_mut() }
}

impl<S: Scalar> Circuit<S> {
    pub fn new() -> Result<CircuitRef<S>, CircuitError> {
        let builder = MatrixBuilder::new(0, 0)?;

        let circuit = Rc::new(RefCell::new(Circuit {
            bipoles: Vec::new(),
            myself: None,
            vsns: LinearNamespace::new(),
            ndns: LinearNamespace::new(),
            builder: builder.clone(),
            eval: builder.clone().build()?,
            need_lin: false,
            need_build: false,
        }));

        let circuit2 = circuit.clone();
        (*circuit.borrow_mut()).myself = Some(circuit2);
        Ok(CircuitRef(circuit))
    }

    pub fn myself(&self) -> CircuitRef<S> { CircuitRef(self.myself.as_ref().expect("found a circuit with invalid `myself`").clone()) }

    pub fn add(&mut self, kind: BipoleKind<S>) -> BipoleRef<S> {
        let bp = Rc::new(RefCell::new(Bipole {
            pos: self.alloc_pin(),
            neg: self.alloc_pin(),
            vsid: if let BipoleKind::VoltageSource(_) = kind { Some(self.alloc_vsid()) } else { None },
            kind: kind,
            circuit: Rc::downgrade(&self.myself().0),
        }));
        self.bipoles.push(bp.clone());
        BipoleRef(bp)
    }

    fn need_lin(&mut self) {
        self.need_lin = true;
        self.need_build = true;
    }

    fn need_build(&mut self) {
        self.need_build = true;
    }

    fn update(&mut self) -> Result<(), CircuitError> {
        if self.need_lin {
            let sources = self.vsns.linearize();
            let nodes = self.ndns.linearize();
            self.builder = MatrixBuilder::new(nodes, sources)?;
            for bp in &self.bipoles {
                self.apply_effect(&*bp.borrow());
            }
        }

        if self.need_build {
            self.eval = self.builder.clone().build()?;
        }

        Ok(())
    }

    fn alloc_vsid(&mut self) -> Name {
        self.need_lin();
        self.vsns.next()
    }

    fn alloc_pin(&mut self) -> Pin {
        self.need_lin();
        Pin(Some(self.ndns.next()))
    }

    fn apply_effect(&self, bp: &Bipole<S>) {
        let myself = self.myself();
        let mut me = myself.borrow_mut();
        match bp.kind() {
            &BipoleKind::Resistor(r) => {
                me.need_build();
                match (bp.pos().id(), bp.neg().id()) {
                    (Some(p), Some(n)) => me.builder.add_conductance(p, Some(n), r.recip()),
                    (Some(p), None) => me.builder.add_conductance(p, None, r.recip()),
                    (None, Some(n)) => me.builder.add_conductance(n, None, r.recip()),
                    (None, None) => (),
                }
            }
            &BipoleKind::VoltageSource(v) => {
                me.update();
                if let Some(vsid) = bp.vsid().map(Name::id) {
                    me.eval.add_potential(vsid, v);
                }
            }
            &BipoleKind::CurrentSource(i) => {
                me.update();
                if let Some(p) = bp.pos().id() {
                    me.eval.add_current(p, i);
                }
                if let Some(n) = bp.neg().id() {
                    me.eval.add_current(n, -i);
                }
            }
        }
    }

    fn repeal_effect(&self, bp: &Bipole<S>) {
        let myself = self.myself();
        let mut me = myself.borrow_mut();
        match bp.kind() {
            &BipoleKind::Resistor(r) => {
                me.need_build();
                match (bp.pos().id(), bp.neg.id()) {
                    (Some(p), Some(n)) => me.builder.add_conductance(p, Some(n), -r.recip()),
                    (Some(p), None) => me.builder.add_conductance(p, None, -r.recip()),
                    (None, Some(n)) => me.builder.add_conductance(n, None, -r.recip()),
                    (None, None) => (),
                }
            }
            &BipoleKind::VoltageSource(v) => {
                me.update();
                if let Some(vsid) = bp.vsid().map(Name::id) {
                    me.eval.add_potential(vsid, -v);
                }
            }
            &BipoleKind::CurrentSource(i) => {
                me.update();
                if let Some(p) = bp.pos().id() {
                    me.eval.add_current(p, -i);
                }
                if let Some(n) = bp.neg().id() {
                    me.eval.add_current(n, i);
                }
            }
        }
    }
}
