use core::panic;
use std::ops::Deref;

use json::{json, Json};

#[test]
fn simple() {
    assert!(matches!(
            Json::deserialize("true"),
            Ok(Json::True)
    ));
    assert!(matches!(
            Json::deserialize("false"),
            Ok(Json::False)
    ));
    assert!(matches!(
            Json::deserialize("null"),
            Ok(Json::Null)
    ));
    let string = "\"string\"".to_owned();
    match Json::deserialize(&string) {
       Ok(Json::String(s)) => assert_eq!(s.deref(), "string"),
       _ => panic!()
    }
}

#[test]
fn nested() {
    let j = Json::deserialize(r#"{
        "array" : [1,2,3, {
            "nested_1" : 1
        }]
    }"#).unwrap();

    let expected = json!(
        {
            "array" : [1,2,3, {
                "nested_1" : 1
            }]
        }
    );

    assert_eq!(expected, j);
}

