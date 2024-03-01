use ethers::abi::{self, AbiError};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
#[error("wrong command")]
pub struct WrongCommand;

impl From<WrongCommand> for AbiError {
    fn from(e: WrongCommand) -> Self {
        Self::DecodingError(abi::Error::Other(e.to_string().into()))
    }
}

#[derive(ThisError, Debug)]
#[error("invalid input/output length")]
pub struct InvalidLength;

impl From<InvalidLength> for AbiError {
    fn from(e: InvalidLength) -> Self {
        Self::DecodingError(abi::Error::Other(e.to_string().into()))
    }
}

#[derive(ThisError, Debug)]
#[error("length mismatch")]
pub struct LengthMismatch;

impl From<LengthMismatch> for AbiError {
    fn from(e: LengthMismatch) -> Self {
        Self::DecodingError(abi::Error::Other(e.to_string().into()))
    }
}

#[derive(ThisError, Debug)]
#[error("index too big")]
pub struct IndexTooBig;

impl From<IndexTooBig> for AbiError {
    fn from(e: IndexTooBig) -> Self {
        Self::DecodingError(abi::Error::Other(e.to_string().into()))
    }
}

#[derive(ThisError, Debug)]
#[error("call must not fail")]
pub struct MustNotFail;

impl From<MustNotFail> for AbiError {
    fn from(e: MustNotFail) -> Self {
        Self::DecodingError(abi::Error::Other(e.to_string().into()))
    }
}
