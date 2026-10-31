use fson::JsonValue::Numeric;
use fson::Parser;

#[test]
#[should_panic(expected = "invalid float literal")]
fn should_invalid_float_type() {
    let mut parser = Parser::new("0.5.0");
    let value = parser.read_value();
    if let Numeric(numeric) = value {
        numeric.as_f64();
    }
}
