use ethers::abi::{AbiDecode, AbiEncode, AbiError, InvalidOutputType, Token, Tokenizable};

pub struct OkOrNone(bool);

impl From<OkOrNone> for bool {
    fn from(value: OkOrNone) -> Self {
        value.0
    }
}

impl AbiDecode for OkOrNone {
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
        let bytes = bytes.as_ref();
        if bytes.is_empty() {
            return Ok(Self(true));
        }
        bool::decode(bytes).map(Self)
    }
}

impl AbiEncode for OkOrNone {
    fn encode(self) -> Vec<u8> {
        self.0.encode()
    }
}

impl Tokenizable for OkOrNone {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType>
    where
        Self: Sized,
    {
        bool::from_token(token).map(Self)
    }

    fn into_token(self) -> Token {
        self.0.into_token()
    }
}
