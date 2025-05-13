pub fn retry<T, E, F>(n: usize, mut f: F) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut last_err: Option<E> = None;
    for _ in 0..=n {
        match f() {
            Ok(value) => return Ok(value),
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.unwrap())
}
