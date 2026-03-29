use mathhook_core::matrices::Matrix;
use mathhook_core::MathLanguage::LaTeX;

#[allow(dead_code)]
pub fn print_matrix(matrix: &Matrix) {
    let (n, m) = matrix.dimensions();
    for i in 0..n {
        for j in 0..m {
            println!(
                "M_{}{} = {} ",
                i,
                j,
                matrix
                    .get_element(i, j)
                    .format_as(LaTeX)
                    .unwrap()
                    .to_string()
            );
        }
    }
}
