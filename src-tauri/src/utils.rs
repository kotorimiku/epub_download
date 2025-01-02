use character_converter::traditional_to_simplified;

pub fn t2s(str: &str) -> String {
    traditional_to_simplified(str).to_string()
}

// pub fn s2t(str: &str) -> String {
//     simplified_to_traditional(str).to_string()
// }

pub fn remove_invalid_chars(filename: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    filename.chars().filter(|&c| !invalid_chars.contains(&c)).collect()
}

