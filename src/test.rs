use self::solver::*;
use super::*;

use std::time;

fn make_simple_circuit<S: Scalar>(r: S) -> Result<MatrixEvaluator<S>, MatrixError> {
    let mut builder = MatrixBuilder::<S>::new(1, 1)?;
    builder.add_conductance(0, None, r.recip());
    builder.add_vs_con(0, Some(0), None);
    builder.build()
}

fn ohms_law<S: Scalar>() -> Result<(), MatrixError> {
    for r in 1..3 {
        let mut cir = make_simple_circuit(S::from_f64(r as f64))?;
        for v in 1..3 {
            cir.src_potentials()[0] = S::from_f64(v as f64);
            assert_eq!(cir.get_potential(0)?, S::from_f64(v as f64));
            assert_eq!(cir.get_current(0)?, S::from_f64(-(v as f64) / (r as f64)));
        }
    }
    Ok(())
}

#[test]
fn ohms_law_f32() -> Result<(), MatrixError> {
    ohms_law::<f32>()
}
#[test]
fn ohms_law_f64() -> Result<(), MatrixError> {
    ohms_law::<f64>()
}
