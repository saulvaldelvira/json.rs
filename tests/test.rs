use core::panic;
use std::ops::Deref;

use json::{json, Json, JsonConfig};

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

#[test]
fn maximun_depth() {
    let mut s = "[12]".to_string();
    const DEPTH: u32 = 500;
    let conf = JsonConfig {
        max_depth: DEPTH,
        ..Default::default()
    };

    for _ in 0..DEPTH-1 {
        s = format!("[{s}]");
    }

    assert!(Json::deserialize_with_config(&s, conf).is_ok());

    s = format!("[{s}]");

    let j = Json::deserialize_with_config(&s, conf);
    match j {
        Ok(_) => panic!("Expected error"),
        Err(err) => assert_eq!(err.get_message(), "Max depth reached")
    }
}
