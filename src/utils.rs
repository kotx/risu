pub fn alphanumeric(str: &str) -> String {
    str.chars().filter(|char| char.is_alphanumeric()).collect()
}
