use super::plane_wave::plane_wave_magnetic_exprs;
use mathhook_core::Parser;

#[test]
fn electric_source_mixed_static_term_does_not_use_plane_wave_shortcut() {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    let electric_exprs = [parse("0"), parse("cos(z - t) + x"), parse("0")];

    assert!(plane_wave_magnetic_exprs(&electric_exprs, 1.0).is_none());
}
