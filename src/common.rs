pub trait Filter {
    fn is_include(&self, line: &String) -> bool;
}
