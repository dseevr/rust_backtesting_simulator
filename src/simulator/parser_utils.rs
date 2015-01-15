
pub fn empty_or_comment(line: &str) -> bool {
    0 == line.len() || line.char_at(0) == '/' && line.char_at(1) == '/'
}

pub fn validate_name(name: &str) {
    if 0 == name.len() {
        panic!("variable name must not be empty");
    }

    for c in name.chars() {
        let alpha = (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
        let numeric = c >= '0' && c <= '9';
        let special = c == '_' || c == '-';
        
        if !alpha && !numeric && !special {
            panic!("invalid character in variable name: {}", c);
        }
    }
}
