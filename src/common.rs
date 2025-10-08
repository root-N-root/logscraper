pub trait Filter {
    fn is_include(&self, line: &String) -> bool;
}

pub trait Batch {
    //TODO:: functions
    fn sort(&self); //TODO:: sort fn
}
