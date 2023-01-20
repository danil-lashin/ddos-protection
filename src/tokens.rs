use std::convert::Into;

const SOME_FAKE_TOKEN: &str = "token";

pub fn generate_token() -> String {
    return SOME_FAKE_TOKEN.into(); // todo
}

pub fn check_token(token: String) -> bool {
    return token == SOME_FAKE_TOKEN; // todo
}
