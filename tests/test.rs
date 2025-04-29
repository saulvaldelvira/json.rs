use core::panic;
use std::ops::Deref;

use json::{Json, JsonConfig, json};

#[test]
fn simple() {
    assert!(matches!(Json::deserialize("true"), Ok(Json::True)));
    assert!(matches!(Json::deserialize("false"), Ok(Json::False)));
    assert!(matches!(Json::deserialize("null"), Ok(Json::Null)));
    let string = "\"string\"".to_owned();
    match Json::deserialize(&string) {
        Ok(Json::String(s)) => assert_eq!(s.deref(), "string"),
        _ => panic!(),
    }
}

#[test]
fn nested() {
    let j = Json::deserialize(
        r#"{
        "array" : [1,2,3, {
            "nested_1" : 1
        }]
    }"#,
    )
    .unwrap();

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

    for _ in 0..DEPTH - 1 {
        s = format!("[{s}]");
    }

    assert!(Json::deserialize_with_config(&s, conf).is_ok());

    s = format!("[{s}]");

    let j = Json::deserialize_with_config(&s, conf);
    match j {
        Ok(_) => panic!("Expected error"),
        Err(err) => assert_eq!(err.get_message(), "Max depth reached"),
    }
}

#[test]
fn index_obj() {
    let mut json = json!({
        "a": 1,
        "b": 2,
        "c": 3,
        "d": 4,
        "e": 5
    });

    assert_eq!(json["a"].expect_number(), 1.0);
    assert_eq!(json["b"].expect_number(), 2.0);
    assert_eq!(json["c"].expect_number(), 3.0);
    assert_eq!(json["d"].expect_number(), 4.0);
    assert_eq!(json["e"].expect_number(), 5.0);

    for e in ["a", "b", "c", "d", "e"] {
        json[e] += 1;
    }

    assert_eq!(json["a"].expect_number(), 2.0);
    assert_eq!(json["b"].expect_number(), 3.0);
    assert_eq!(json["c"].expect_number(), 4.0);
    assert_eq!(json["d"].expect_number(), 5.0);
    assert_eq!(json["e"].expect_number(), 6.0);
}

#[test]
#[should_panic(expected = "Attemp to index a json element that doesn't contain the given key: 'h'")]
fn index_obj_panic() {
    let json = json!({
        "a": 1,
        "b": 2
    });
    let _ = &json["h"];
}

#[test]
fn index_arr() {
    let mut json = json!([0, 1, 2, 3, 4, 5]);

    for i in 0..6 {
        assert_eq!(json[i].expect_number(), i as f64);
        json[i] *= 2;
        assert_eq!(json[i].expect_number(), (i * 2) as f64);
    }
}

#[test]
#[should_panic(expected = "Attemp to index a json element that can't be indexed by 12")]
fn index_arr_panic() {
    let json = json!({
        "a": 1,
        "b": 2
    });
    let _ = &json[12];
}

#[test]
fn macro_test() {
    fn foo() -> u16 {
        12
    }

    let a = 1.0;
    let json = json! {{
        "a + 2" : { a + 2.0 },
        "foo()" : { foo() },
        "nest" : [
            { (1 + 1) * 0 },
            { 2 },
        ]
    }};

    let obj = json.expect_object();
    assert_eq!(obj["a + 2"].expect_number(), a + 2.0);
    assert_eq!(obj["foo()"].expect_number(), foo() as f64);

    let nest = &obj["nest"];
    assert_eq!(nest[0].expect_number(), 0.0);
    assert_eq!(nest[1].expect_number(), 2.0);
}
