use crate::common::traits::Filter;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead},
};

async fn read_tail(
    path: String,
    filters: Option<Vec<Box<dyn Filter>>>,
) -> Result<TailStream, Box<dyn Error>> {
    let file = File::open(path)?;

    //let mut lines = Vec::with_capacity(limit); // Чтобы сразу выйти при limit = 0
}

#[cfg(test)]
mod tests {
    #[test]
    async fn test_tail_error_wrong_path() {
        let random_path = random_str::get_string(6, true, false, true, true);
        let res = read_tail(random_path, 1, 1, None);
        assert!(res.await.is_err())
    }
}
