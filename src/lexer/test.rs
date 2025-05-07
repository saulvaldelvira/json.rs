use crate::DEFAULT_CONFIG;

use super::*;

#[test]
fn unknown_keyword() {
    match tokenize("keywrd", DEFAULT_CONFIG) {
        Ok(_) => panic!("Expected error"),
        Err(msg) => assert_eq!(msg.get_message(), "Unknown keyword 'keywrd'"),
    }
}

#[test]
fn with_comments() {
    let tokens = tokenize(
        "12 // kjdakldjs",
        JsonConfig {
            allow_comments: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(tokens[0].get_type(), TokenKind::Number);
}

#[test]
fn comments_non_supported() {
    match tokenize(
        "12 // kjdakldjs",
        JsonConfig {
            allow_comments: false,
            ..Default::default()
        },
    ) {
        Ok(_) => panic!("Expected Ok"),
        Err(err) => assert_eq!(err.get_message(), "[0:3] Comments are not supported"),
    }
}
