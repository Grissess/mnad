use super::*;

use std::iter;

use rlapack::ll::{__CLPK_integer, __CLPK_real, __CLPK_doublereal};
use libc::{c_int, c_char};

#[derive(Debug,Clone)]
pub struct MatrixBuilder<S: Scalar> {
    nodes: usize,
    stride: usize,
    matrix: Vec<S>,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum MatrixError {
    Overflow,
    Singular{idx: usize},
    BadArg{idx: usize},
}

impl<S: Scalar> MatrixBuilder<S> {
    pub fn new(nodes: usize, sources: usize) -> Result<MatrixBuilder<S>, MatrixError> {
        let size = nodes + sources;
        Ok(MatrixBuilder {
            nodes: nodes,
            stride: size,
            matrix: iter::repeat(S::zero()).take(
                size.checked_mul(size).ok_or(MatrixError::Overflow)?
            ).collect::<Vec<S>>(),
        })
    }

    pub fn nodes(&self) -> usize { self.nodes }
    pub fn sources(&self) -> usize { self.stride - self.nodes }
    pub fn size(&self) -> usize { self.stride }
    pub fn matrix(&self) -> Vec<S> { self.matrix.clone() }

    pub fn add_conductance(&mut self, a: usize, b: Option<usize>, c: S) {
        self.matrix[a * self.stride + a] += c;
        if let Some(n) = b {
            self.matrix[n * self.stride + n] += c;
            self.matrix[n * self.stride + a] -= c;
            self.matrix[a * self.stride + n] -= c;
        }
    }

    pub fn add_vs_con(&mut self, src: usize, pos: Option<usize>, neg: Option<usize>) {
        if let Some(p) = pos {
            self.matrix[(self.nodes + src) * self.stride + p] = S::one();
            self.matrix[p * self.stride + self.nodes + src] = S::one();
        }
        if let Some(n) = neg {
            self.matrix[(self.nodes + src) * self.stride + n] = -S::one();
            self.matrix[n * self.stride + self.nodes + src] = -S::one();
        }
    }

    pub fn remove_vs_con(&mut self, src: usize, a: Option<usize>, b: Option<usize>) {
        if let Some(p) = a {
            self.matrix[(self.nodes + src) * self.stride + p] = S::zero();
            self.matrix[p * self.stride + self.nodes + src] = S::zero();
        }
        if let Some(n) = b {
            self.matrix[(self.nodes + src) * self.stride + n] = S::zero();
            self.matrix[n * self.stride + self.nodes + src] = S::zero();
        }
    }

    pub fn build(mut self) -> Result<MatrixEvaluator<S>, MatrixError> {
        let mut m: c_int = self.stride as c_int;
        let mut n: c_int = self.stride as c_int;
        let mut lda: c_int = self.stride as c_int;
        let mut piv: Vec<c_int> = iter::repeat(0).take(self.stride).collect();
        let mut info: c_int = 0;

        unsafe { 
            match S::precision() {
                Precision::Single => {
                    rlapack::ll::sgetrf_(
                        &mut m as *mut __CLPK_integer,
                        &mut n as *mut __CLPK_integer,
                        self.matrix.as_mut_ptr() as *mut __CLPK_real,
                        &mut lda as *mut __CLPK_integer,
                        piv.as_mut_ptr() as *mut __CLPK_integer,
                        &mut info as *mut __CLPK_integer
                    );
                },
                Precision::Double => {
                    rlapack::ll::dgetrf_(
                        &mut m as *mut __CLPK_integer,
                        &mut n as *mut __CLPK_integer,
                        self.matrix.as_mut_ptr() as *mut __CLPK_doublereal,
                        &mut lda as *mut __CLPK_integer,
                        piv.as_mut_ptr() as *mut __CLPK_integer,
                        &mut info as *mut __CLPK_integer
                    );
                },
            }
        }

        if info < 0 {
            return Err(MatrixError::BadArg{idx: (-info) as usize});
        }
        if info > 0 {
            return Err(MatrixError::Singular{idx: (info - 1) as usize});
        }

        Ok(MatrixEvaluator {
            dirty: true,
            nodes: self.nodes,
            stride: self.stride,
            matrix: self.matrix,
            piv: piv,
            known: iter::repeat(S::zero()).take(self.stride).collect(),
            out: iter::repeat(S::zero()).take(self.stride).collect(),
        })
    }
}

pub struct MatrixEvaluator<S: Scalar> {
    dirty: bool,
    nodes: usize,
    stride: usize,
    matrix: Vec<S>,
    piv: Vec<c_int>,
    known: Vec<S>,
    out: Vec<S>,
}

impl<S: Scalar> MatrixEvaluator<S> {
    pub fn nodes(&self) -> usize { self.nodes }
    pub fn sources(&self) -> usize { self.stride - self.nodes }

    pub fn add_current(&mut self, node: usize, i: S) {
        self.known[node] += i;
        self.dirty = true;
    }

    pub fn node_currents(&mut self) -> &mut [S] {
        &mut self.known[..self.nodes]
    }

    pub fn add_potential(&mut self, src: usize, p: S) {
        self.known[self.nodes + src] += p;
        self.dirty = true;
    }

    pub fn src_potentials(&mut self) -> &mut [S] {
        &mut self.known[self.nodes..]
    }

    pub fn get_potential(&mut self, node: usize) -> Result<S, MatrixError> {
        if self.dirty { self.solve()?; }
        Ok(self.out[node])
    }

    pub fn node_potentials(&mut self) -> Result<&mut [S], MatrixError> {
        if self.dirty { self.solve()?; }
        Ok(&mut self.out[..self.nodes])
    }

    pub fn get_current(&mut self, src: usize) -> Result<S, MatrixError> {
        if self.dirty { self.solve()?; }
        Ok(self.out[self.nodes + src])
    }

    pub fn src_currents(&mut self) -> Result<&mut [S], MatrixError> {
        if self.dirty { self.solve()?; }
        Ok(&mut self.out[self.nodes..])
    }

    pub fn solve(&mut self) -> Result<(), MatrixError> {
        let mut trans: c_char = 'N' as c_char;
        let mut n: c_int = self.stride as c_int;
        let mut nrhs: c_int = 1;
        let mut lda: c_int = self.stride as c_int;
        let mut ldb: c_int = self.stride as c_int;
        let mut info: c_int = 0;
        self.out = self.known.clone();

        unsafe {
            match S::precision() {
                Precision::Single => {
                    rlapack::ll::sgetrs_(
                        &mut trans as *mut c_char,
                        &mut n as *mut __CLPK_integer,
                        &mut nrhs as *mut __CLPK_integer,
                        self.matrix.as_mut_ptr() as *mut __CLPK_real,
                        &mut lda as *mut __CLPK_integer,
                        self.piv.as_mut_ptr() as *mut __CLPK_integer,
                        self.out.as_mut_ptr() as *mut __CLPK_real,
                        &mut ldb as *mut __CLPK_integer,
                        &mut info as *mut __CLPK_integer,
                    );
                },
                Precision::Double => {
                    rlapack::ll::dgetrs_(
                        &mut trans as *mut c_char,
                        &mut n as *mut __CLPK_integer,
                        &mut nrhs as *mut __CLPK_integer,
                        self.matrix.as_mut_ptr() as *mut __CLPK_doublereal,
                        &mut lda as *mut __CLPK_integer,
                        self.piv.as_mut_ptr() as *mut __CLPK_integer,
                        self.out.as_mut_ptr() as *mut __CLPK_doublereal,
                        &mut ldb as *mut __CLPK_integer,
                        &mut info as *mut __CLPK_integer,
                    );
                },
            }
        }

        if info < 0 {
            return Err(MatrixError::BadArg{idx: (-info) as usize});
        }

        Ok(())
    }
}
