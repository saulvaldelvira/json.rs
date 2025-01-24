#![feature(test)]
extern crate test;

use json::Json;
use test::bench::Bencher;

const TEXT: &str = r#"{"hello":"hello","obj":{"22":2,"1":1},"arr":[1,2,3,4,5],"inner":{"hello":"hello","arr":[1,2,3,4,5],"inner":{"inner":{"obj":{"22":2,"1":1},"hello":"hello","arr":[1,2,3,4,5],"super_inner":{"inner":{"arr":[1,2,3,4,5],"hello":"hello","obj":{"1":1,"22":2},"inner":{"hello":"hello","inner":{"arr":[1,2,3,4,5],"obj":{"1":1,"22":2},"hyper_inner":{"inner":{"obj":{"22":2,"1":1},"arr":[1,2,3,4,5],"inner":{"hello":"hello","obj":{"1":1,"22":2},"arr":[1,2,3,4,5],"inner":{"super_inner":{"obj":{"22":2,"1":1},"arr":[1,2,3,4,5],"hello":"hello","inner":{"arr":[1,2,3,4,5],"inner":{"obj":{"22":2,"1":1},"hello":"hello","inner":{"obj":{"22":2,"1":1},"hello":"hello","arr":[1,2,3,4,5]},"arr":[1,2,3,4,5]},"obj":{"22":2,"1":1},"hello":"hello"}},"obj":{"1":1,"22":2},"hello":"hello","arr":[1,2,3,4,5]}},"hello":"hello"},"obj":{"22":2,"1":1},"hello":"hello","arr":[1,2,3,4,5]},"hello":"hello"},"obj":{"22":2,"1":1},"arr":[1,2,3,4,5]}},"hello":"hello","arr":[1,2,3,4,5],"obj":{"1":1,"22":2}}},"obj":{"1":1,"22":2},"arr":[1,2,3,4,5],"hello":"hello"},"obj":{"1":1,"22":2}}}"#;

fn get_huge() -> String {
    let mut s = format!("[{TEXT}");
    for _ in 0..100 {
        s.push(',');
        s.push_str(TEXT);
    }
    s.push(']');
    s
}

#[bench]
fn deserialize(b: &mut Bencher) {
    b.iter(|| {
        Json::deserialize(TEXT).unwrap();
    })
}

#[bench]
fn serialize(b: &mut Bencher) {
    let j = Json::deserialize(TEXT).unwrap();
    b.iter(|| {
        let mut s = String::new();
        j.serialize(&mut s).unwrap();
    })
}

#[bench]
fn deserialize_huge(b: &mut Bencher) {
    let huge = get_huge();
    b.iter(|| {
        Json::deserialize(&huge).unwrap();
    })
}

#[bench]
fn serialize_huge(b: &mut Bencher) {
    let j = Json::deserialize(&get_huge()).unwrap();
    b.iter(|| {
        let mut s = String::new();
        j.serialize(&mut s).unwrap();
    })
}
