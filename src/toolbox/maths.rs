use mathhook_core::MathLanguage::LaTeX;
use mathhook_core::matrices::Matrix;

pub fn print_matrix(matrix: &Matrix) {
    let (n, m) = matrix.dimensions();
    for i in 0..n {
        for j in 0..m {
            println!("{} ", matrix.get_element(i, j).format_as(LaTeX).unwrap().to_string());
        }
    }
}