extern crate mnad;

use mnad::solver::*;
use mnad::*;

fn main() {
    io_main_2().expect("main failed");
}

fn io_main_1() -> Result<(), MatrixError> {
    let mut builder = MatrixBuilder::<f64>::new(2, 1)?;
    builder.add_conductance(0, Some(1), 0.1f64);
    builder.add_conductance(1, None, 0.1f64);
    builder.add_vs_con(0, Some(0), Some(1));
    let mut circuit = builder.build()?;
    circuit.src_potentials()[0] = 1.0f64;
    for (idx, p) in circuit.node_potentials()?.iter().enumerate() {
        println!("node {} potential {}", idx, p);
    }
    Ok(())
}

fn io_main_2() -> Result<(), MatrixError> {
    let mut builder = MatrixBuilder::<f64>::new(3, 2)?;
    builder.add_conductance(1, Some(2), 0.1f64);
    builder.add_conductance(0, Some(1), 1.0 / 30.0f64);
    builder.add_vs_con(0, None, Some(0));
    builder.add_vs_con(1, None, Some(2));
    print_matrix(builder.size(), &builder.matrix());
    let mut circuit = builder.build()?;
    {
        let mut pots = circuit.src_potentials();
        pots[0] = 5.0f64;
        pots[1] = 20.0f64;
    }
    for (idx, p) in circuit.node_potentials()?.iter().enumerate() {
        println!("node {} potential {}", idx, p);
    }
    Ok(())
}
