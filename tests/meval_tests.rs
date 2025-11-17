#[macro_use]
extern crate serde_derive;
extern crate meval;
extern crate toml;
use meval::{Expr, Context};
#[derive(Deserialize)]
struct Func{
    f: Expr
}

#[test]
fn test() {
    let conf = r#"
        f = "x+2"
    "#;
    let func: Func = toml::from_str(conf).unwrap();
    assert_eq!(func.f.bind("x").unwrap()(2.), 4f64)
}