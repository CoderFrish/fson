use fson::{JsonValue};
use fson::error::{JsonError};

#[test]
fn test_string() -> Result<(), JsonError> {
    let string = r#""Hello World!!!""#;
    let value = fson::parse(string)?;
    let expected = "Hello World!!!".to_owned();

    assert_eq!(value, JsonValue::String(expected));

    Ok(())
}

#[test]
fn test_string_with_escape() -> Result<(), JsonError> {
    let string = r#""\\u0041 Hello World!!!, I say \"Hello\" \\ \u4E2D文""#;
    let value = fson::parse(string)?;
    let expected = "\\u0041 Hello World!!!, I say \"Hello\" \\ 中文".to_owned();

    assert_eq!(value, JsonValue::String(expected));

    Ok(())
}

#[test]
fn test_boolean() -> Result<(), JsonError> {
    let string = r#"false"#;
    let value = fson::parse(string)?;

    println!("{:?}", value);

    Ok(())
}

#[test]
fn test_array() -> Result<(), JsonError> {
    let string = r#"[ "Frish2021", "CoderFrish" ]"#;
    let value = fson::parse(string)?;

    println!("{:?}", value);

    Ok(())
}

#[test]
fn test_object() -> Result<(), JsonError> {
    let string = r#"
    {
        "nickname": "Frish2021",
        "username": "CoderFrish",
        "numeric": Nan
    }
    "#;
    let value = fson::parse(string)?;

    println!("{:?}", value);

    Ok(())
}
