pub struct EtherCrabError(ethercrab::error::Error);

// do the same for result

pub type EtherCrabResult<T> = Result<T, EtherCrabError>;

impl<T> From<Result<T, ethercrab::error::Error>> for EtherCrabResult<T> {
    fn from(res: Result<T, ethercrab::error::Error>) -> Self {
        res.map_err(|e| e.into())
    }
}

impl<T> From<EtherCrabResult<T>> for anyhow::Result<T> {
    fn from(res: EtherCrabResult<T>) -> Self {
        res.map_err(|e| e.into())
    }
}
