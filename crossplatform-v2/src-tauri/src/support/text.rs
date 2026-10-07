/// Upper-cases the first character; vendor plan names arrive as "pro", "free", ...
pub fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn caps_first_letter_only() {
        assert_eq!(super::cap("pro plan"), "Pro plan");
        assert_eq!(super::cap(""), "");
    }
}
