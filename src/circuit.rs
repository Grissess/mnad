use super::*;
use self::ns::*;
use self::solver::*;

use std::rc::{Weak, Rc};
use std::cell::RefCell;

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum CircuitError {
    MatrixError(MatrixError),
    CircuitDead,
}

impl From<MatrixError> for CircuitError {
    fn from(v: MatrixError) -> CircuitError { CircuitError::MatrixError(v) }
}

#[derive(Debug,Clone)]
pub enum BipoleKind<S: Scalar> {
    Resistor(S),
    VoltageSource(S),
    CurrentSource(S),
}

#[derive(Debug)]
pub struct Pin(Option<NameFnP>);

impl Pin {
    pub fn ground() -> Pin { Pin(None) }
    pub fn is_ground(&self) -> bool { self.0.is_none() }

    pub fn id(&self) -> Option<usize> { self.0.map(Name::id) }

    pub fn connect(&mut self, other: &mut Pin) {
        match (self.0, other.0) {
            (Some(ref mut a), Some(ref b)) => a.unify(b),
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
    vsid: Option<NameFnP>,
    kind: BipoleKind<S>,
    circuit: Weak<RefCell<Circuit<S>>>,
}

impl<S: Scalar> Bipole<S> {
    pub fn pos(&mut self) -> &mut Pin { &mut self.pos }
    pub fn neg(&mut self) -> &mut Pin { &mut self.neg }
    pub fn vsid(&self) -> Option<&NameFnP> { (&self.vsid).as_ref() }
    pub fn kind(&self) -> &BipoleKind<S> { &self.kind }
    pub fn circuit(&self) -> Option<Rc<RefCell<Circuit<S>>>> { self.circuit.upgrade() }

    pub fn set_kind(&mut self, kind: BipoleKind<S>) -> Result<(), CircuitError>{
        let mut circuit = self.circuit().ok_or(CircuitError::CircuitDead)?.borrow_mut();

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

#[derive(Debug)]
pub struct Circuit<S: Scalar> {
    bipoles: Vec<Bipole<S>>,
    vsns: LinearNamespaceFnP,
    ndns: LinearNamespaceFnP,
    builder: MatrixBuilder<S>,
    eval: MatrixEvaluator<S>,
    need_lin: bool,
    need_build: bool,
}

impl<S: Scalar> Circuit<S> {
    fn need_lin(&mut self) {
        self.need_lin = true;
        self.need_build = true;
    }

    fn need_build(&mut self) {
        self.need_build = true;
    }

    fn update(&mut self) -> Result<(), CircuitError> {
        if self.need_lin {
            self.vsns.linearize();
            self.ndns.linearize();
        }

        if self.need_build {
            self.eval = self.builder.clone().build()?;
        }

        Ok(())
    }

    fn alloc_vsid(&mut self) -> NameFnP {
        self.need_lin();
        self.vsns.next()
    }

    fn alloc_pin(&mut self) -> Pin {
        self.need_lin();
        Pin(Some(self.ndns.next()))
    }

    fn apply_effect(&mut self, bp: &Bipole<S>) {
        match bp.kind() {
            &BipoleKind::Resistor(r) => {
                self.need_build();
                match (bp.pos().id(), bp.neg().id()) {
                    (Some(p), Some(n)) => self.builder.add_conductance(p, Some(n), r.recip()),
                    (Some(p), None) => self.builder.add_conductance(p, None, r.recip()),
                    (None, Some(n)) => self.builder.add_conductance(n, None, r.recip()),
                    (None, None) => (),
                }
            },
            &BipoleKind::VoltageSource(v) => {
                self.update();
                if let Some(vsid) = bp.vsid().map(Name::id) {
                    self.eval.add_potential(vsid, v);
                }
            },
            &BipoleKind::CurrentSource(i) => {
                self.update();
                if let Some(p) = bp.pos().id() {
                    self.eval.add_current(p, i);
                }
                if let Some(n) = bp.neg().id() {
                    self.eval.add_current(n, -i);
                }
            },
        }
    }

    fn repeal_effect(&mut self, bp: &Bipole<S>) {
        match bp.kind() {
            &BipoleKind::Resistor(r) => {
                self.need_build();
                match (bp.pos().id(), bp.neg.id()) {
                    (Some(p), Some(n)) => self.builder.add_conductance(p, Some(n), -r.recip()),
                    (Some(p), None) => self.builder.add_conductance(p, None, -r.recip()),
                    (None, Some(n)) => self.builder.add_conductance(n, None, -r.recip()),
                    (None, None) => (),
                }
            },
            &BipoleKind::VoltageSource(v) => {
                self.update();
                if let Some(vsid) = bp.vsid().map(Name::id) {
                    self.eval.add_potential(vsid, -v);
                }
            },
            &BipoleKind::CurrentSource(i) => {
                self.update();
                if let Some(p) = bp.pos().id() {
                    self.eval.add_current(p, -i);
                }
                if let Some(n) = bp.neg().id() {
                    self.eval.add_current(n, i);
                }
            },
        }
    }
}
