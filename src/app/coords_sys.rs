use crate::maths::{expr_to_fastexpr2dto1d, expr_to_fastexpr3d, Expr, FastExpr2dto1d, FastExpr3d, COORD};
use exmex::{Calculate, Differentiate, ExError, ExResult, Express};
use integrate::prelude::trapezoidal_rule;
use nalgebra::Vector3;
use std::ops::Deref;

pub struct CoordsSys {
    x_eq: Expr,
    y_eq: Expr,
    z_eq: Expr,
    fast_x_eq: FastExpr3d,
    fast_y_eq: FastExpr3d,
    fast_z_eq: FastExpr3d,
    x_curvature: FastExpr2dto1d,
    y_curvature: FastExpr2dto1d,
    z_curvature: FastExpr2dto1d,
}

impl CoordsSys {
    pub fn new(mut x_eq: Expr, mut y_eq: Expr, mut z_eq: Expr) -> Self {
        let (x_curvature, y_curvature, z_curvature) = Self::calculate_curvature(&x_eq, &y_eq, &z_eq).unwrap();
        let fast_x_eq = expr_to_fastexpr3d(x_eq.clone());
        let fast_y_eq = expr_to_fastexpr3d(y_eq.clone());
        let fast_z_eq = expr_to_fastexpr3d(z_eq.clone());
        Self {
            x_eq,
            y_eq,
            z_eq,
            fast_x_eq, fast_y_eq, fast_z_eq, x_curvature, y_curvature, z_curvature }
    }

    #[inline]
    fn calculate_curvature(x_eq: &Expr, y_eq: &Expr, z_eq: &Expr) -> ExResult<(FastExpr2dto1d, FastExpr2dto1d, FastExpr2dto1d)>{
        let mut curvature = Vec::new();
        let two = Expr::from_num(2f64);
        for x_i in 0..3 {
            let mk = |expr: &Expr| -> ExResult<Expr> {
                let vars = expr.var_names();
                match vars {
                    [x] => {
                        if x == COORD[x_i] {
                            Ok(expr.clone().partial_nth(0, 2)?.operate_binary(two.clone(), "^")?)
                        }else {
                            Ok(Expr::from_num(0f64))
                        }
                    },
                    [x, y] => {
                        if x == COORD[x_i] {    // coord[1] = y => dy
                            Ok(expr.clone().partial_nth(0, 2)?.operate_binary(two.clone(), "^")?)
                        } else if y == COORD[x_i] {
                            Ok(expr.clone().partial_nth(1, 2)?.operate_binary(two.clone(), "^")?)
                        }else {
                            Ok(Expr::from_num(0f64))
                        }
                    },
                    [_, _, _] => Ok(expr.clone().partial_nth(x_i, 2)?.operate_binary(two.clone(), "^")?),
                    _ => Err(ExError::new(""))
                }
            };
            let ddx_1 = mk(&x_eq)?;
            let ddx_2 = mk(&y_eq)?;
            let ddx_3 = mk(&z_eq)?;
            let ddx = ddx_1.operate_binary(ddx_2, "+")?.operate_binary(ddx_3, "+")?;
            let ddx = ddx.operate_unary("sqrt")?;
            curvature.push(ddx);
        }
        let [a, b, c] = curvature.try_into().expect("COORD must have 3 elements");
        Ok((expr_to_fastexpr2dto1d(a, "x".to_string()), expr_to_fastexpr2dto1d(b, "y".to_string()), expr_to_fastexpr2dto1d(c, "z".to_string())))
    }

    pub fn get_curvature(&self, point: Vector3<f64>, len: f64) -> (f64, f64, f64) {
        let (x, y, z) = (point.x, point.y, point.z);
        let fx = (self.x_curvature)(y,z);
        let fy = (self.y_curvature)(x,z);
        let fz = (self.z_curvature)(x,y);
        let cx = trapezoidal_rule(fx.deref(), x - len, x + len, 100u32);
        let cy = trapezoidal_rule(fy.deref(), y - len, y + len, 100u32);
        let cz = trapezoidal_rule(fz.deref(), z - len, z + len, 100u32);
        (cx, cy, cz)
    }

    pub fn eval(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let x_ = (self.fast_x_eq)(x, y, z);
        let y_ = (self.fast_y_eq)(x, y, z);
        let z_ = (self.fast_z_eq)(x, y, z);
        (x_, y_, z_)
    }

    pub fn is_equivalent(&self, eqs: &[String; 3]) -> bool {
        println!("{:?}", self.x_eq.to_string());
        eqs[0] == self.x_eq.to_string() && eqs[1] == self.y_eq.to_string() && eqs[2] == self.z_eq.to_string()
    }
}