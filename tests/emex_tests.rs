use exmex::prelude::*;

#[test]
fn tests() {
    let to_parsed = "-x^2";
    let expr = exmex::parse::<f32>(to_parsed).unwrap();
    assert_eq!(expr.eval(&[2.]).unwrap(), 4.);
    
    use exmex::{DeepEx, prelude::*};
    {
        let deep_cos_x = DeepEx::<f64>::parse("cos(x)").unwrap();
        let deep_identity = deep_cos_x.operate_unary("acos").unwrap();
        let one = DeepEx::one();
        let deep_identity = deep_identity.operate_binary(one, "*").unwrap();
        let flat_identity = FlatEx::from_deepex(deep_identity).unwrap();
        println!("{}", flat_identity.eval(&[3.0]).unwrap())
    }
    {
        let deep_cos_x = DeepEx::<f64>::parse("cos(x)").unwrap();
        let deep_identity = deep_cos_x.acos().unwrap();
        let one = DeepEx::one();
        let deep_identity = (deep_identity * one).unwrap();
        let flat_identity = FlatEx::from_deepex(deep_identity).unwrap();
        println!("{}", flat_identity.eval(&[3.0]).unwrap())
    }
    {
        let deep_cos_x = FlatEx::<f32>::parse("cos(x)").unwrap();
        deep_cos_x.operator_reprs(); // Return the operator representation
        let deep_identity = deep_cos_x.operate_unary("acos").unwrap();
        let one = FlatEx::<f32>::parse("1").unwrap();
        let flat_identity = deep_identity.operate_binary(one, "*").unwrap();
        println!("{}", flat_identity.eval(&[3.0]).unwrap())
    }
    
}