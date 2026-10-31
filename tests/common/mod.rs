use fson::Parser;

const JSON: &str = r#"{
    "string": "Hello World!!!",
    "bool_true": true,
    "bool_false": false,
    "null": null,
    "float": 0.5,
    "int": 11,
    "object": {
        "target": "This is a object content."
    },
    "array": ["student0", "student1", "student2"]
}"#;

pub fn parser() -> Parser<'static> {
    Parser::new(JSON)
}
