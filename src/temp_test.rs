#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_addition() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn test_string_concatenation() {
        let name = "ninja";
        let full_name = format!("{}-linter", name);
        assert_eq!(full_name, "ninja-linter");
    }
}
