use super::*;

#[test]
fn unknown_keyword() {
    match tokenize("keywrd") {
        Ok(_) => panic!("Expected error"),
        Err(msg) => assert_eq!(msg.get_message(), "Unknown keyword 'keywrd'")
    }
}
