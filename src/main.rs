use fson::JsonElement;

fn main() {
    let object = fson::new(
        r#"{ "name": "Frish2021", "age": "16", "other": { "id": "1145141919810", "status": "writing" } }"#,
    );
    if let JsonElement::Object(object) = object {
        let other = &object["other"];
        let name = &object["name"];

        if let JsonElement::String(name) = name {
            println!("Name: {}", name);
        }

        if let JsonElement::Object(other) = other {
            let id = &other["id"];

            if let JsonElement::String(id) = id {
                println!("Id: {}", id)
            }
        }
    }
}
