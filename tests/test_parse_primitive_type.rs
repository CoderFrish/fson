use fson::JsonValue::{Object, String, Bool, Null, Numeric};

pub mod common;

#[test]
fn parse_string_type() {
    let object = common::parser().read_value();
    if let Object(map) = object {
        let string = &map["string"];
        if let String(string) = string {
            assert_eq!(string, "Hello World!!!");
        }
    }
}

#[test]
fn parse_float_type() {
    let object = common::parser().read_value();
    if let Object(map) = object {
        let float = &map["float"];
        if let Numeric(float) = float {
            assert_eq!(float.as_f64(), 0.5);
        }
    }
}

#[test]
fn parse_int_type() {
    let object = common::parser().read_value();
    if let Object(map) = object {
        let int = &map["int"];
        if let Numeric(int) = int {
            assert_eq!(int.as_i64(), 11);
        }
    }
}

#[test]
fn parse_bool_type() {
    let object = common::parser().read_value();
    if let Object(map) = object {
        let bool_true = &map["bool_true"];
        if let Bool(bool_true) = bool_true {
            assert!(bool_true)
        }

        let bool_false = &map["bool_false"];
        if let Bool(bool_false) = bool_false {
            assert!(!bool_false)
        }
    }
}


#[test]
fn parse_null_type() {
    let object = common::parser().read_value();
    if let Object(map) = object {
        let null = &map["null"];
        assert!(matches!(null, Null));
    }
}
