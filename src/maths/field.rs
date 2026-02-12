use crate::maths::differential::Form;
use crate::maths::space::Space;

pub struct Field<'a> {
    expr: Form,
    space: &'a Space,
}

impl <'a> Field<'a> {
    pub fn new(expr: Form, space: &'a Space) -> Self {
        Self { expr, space }
    }

    

}

