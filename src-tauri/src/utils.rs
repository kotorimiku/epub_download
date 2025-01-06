use fast2s::convert;

pub fn t2s(str: &str) -> String {
    // traditional_to_simplified(str).to_string()
    convert(str)
}

pub fn remove_invalid_chars(filename: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    filename.chars().filter(|&c| !invalid_chars.contains(&c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t2s() {
        println!("{}", t2s("妳"));
        println!("{}", convert("妳"));
    }
}