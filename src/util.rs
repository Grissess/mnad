use super::*;

pub fn print_matrix<S: Scalar>(size: usize, matrix: &[S]) {
    for row in 0..size {
        for col in 0..size {
            print!("{:+5.3}\t", matrix[size * row + col]);
        }
        print!("\n");
    }
}
