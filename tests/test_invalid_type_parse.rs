use fson::JsonValue::Numeric;

#[test]
#[should_panic(expected = "invalid float literal")]
fn should_invalid_float_type() {
    let value = fson::from_json("0.5.0");
    match value {
        Ok(value) => {
            if let Numeric(numeric) = value {
                numeric.as_f64();
            }
        },

        Err(err) => {
            panic!("{}", err);
        }
    }
}
