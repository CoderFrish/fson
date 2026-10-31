use fson::JsonValue::{Array, Object, String};

pub mod common;

#[test]
fn parse_object_type() {
    let object = common::parser().read_value();
    if let Object(map) = object {
        let object = &map["object"];
        if let Object(object) = object {
            let target = &object["target"];
            if let String(target) = target {
                assert_eq!(target, "This is a object content.")
            }
        }
    }
}

#[test]
fn parse_array_type() {
    let object = common::parser().read_value();
    if let Object(map) = object {
        let array = &map["array"];
        if let Array(array) = array {
            let value0 = &array[0];
            let value1 = &array[1];
            let value2 = &array[2];

            if let String(value0) = value0 {
                assert_eq!(value0, "student0");
            }

            if let String(value1) = value1 {
                assert_eq!(value1, "student1");
            }

            if let String(value2) = value2 {
                assert_eq!(value2, "student2");
            }
        }
    }
}
