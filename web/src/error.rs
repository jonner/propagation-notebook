#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Topcoat(#[from] topcoat::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Propagation(#[from] libpropagation::error::Error),
}
