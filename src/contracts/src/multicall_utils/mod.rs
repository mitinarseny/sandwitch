use ethers::{
    abi::{
        self, ethabi, AbiArrayType, AbiDecode, AbiEncode, AbiError, AbiType, Detokenize,
        InvalidOutputType, ParamType, Token, Tokenizable, TokenizableItem, Tokenize,
    },
    contract::{builders::ContractCall, EthCall},
    providers::Middleware,
    types::{Address, Bytes, U256},
};
use itertools::Itertools;

#[path = "../multicall/mod.rs"]
mod inner;
use inner::multi_call::MultiCall as MultiCallInner;

use self::inner::multi_call::{Failed, MulticallCall};

#[derive(Clone)]
pub enum Call {
    Call {
        target: Address,
        value: U256,
        calldata: Bytes,
    },
    GetBalance(BalanceOf),
    Transfer {
        to: Address,
        amount: U256,
    },
    Create {
        value: U256,
        bytecode: Bytes,
    },
    Create2 {
        value: U256,
        salt: U256,
        bytecode: Bytes,
    },
    Group(MultiCall),
}

#[derive(Clone, Copy)]
pub enum BalanceOf {
    This,
    MsgSender,
    Address(Address),
}

impl Call {
    pub fn call<C: EthCall + Tokenizable>(
        target: Address,
        value: impl Into<U256>,
        call: C,
    ) -> Self {
        Self::Call {
            target,
            value: value.into(),
            calldata: {
                let inputs = abi::encode(&call.into_tokens());
                C::selector().into_iter().chain(inputs).collect().into()
            },
        }
    }

    pub const fn get_balance_of_this() -> Self {
        Self::GetBalance(BalanceOf::This)
    }

    pub const fn get_balance_of_msg_sender() -> Self {
        Self::GetBalance(BalanceOf::MsgSender)
    }

    pub const fn get_balance_of(address: Address) -> Self {
        Self::GetBalance(BalanceOf::Address(address))
    }

    pub const fn transfer(to: Address, amount: impl Into<U256>) -> Self {
        Self::Transfer {
            to,
            amount: amount.into(),
        }
    }

    pub fn create(value: impl Into<U256>, bytecode: impl Into<Bytes>) -> Self {
        Self::Create {
            value: value.into(),
            bytecode: bytecode.into(),
        }
    }

    pub fn create2(
        value: impl Into<U256>,
        salt: impl Into<U256>,
        bytecode: impl Into<Bytes>,
    ) -> Self {
        Self::Create2 {
            value: value.into(),
            salt: salt.into(),
            bytecode: bytecode.into(),
        }
    }

    pub fn group(calls: impl IntoIterator<Item = TryCall>) -> Self {
        Self::Group(calls.into_iter().collect())
    }

    pub fn encode(self) -> (Cmd, Bytes) {
        match self {
            Self::Call {
                target,
                value,
                calldata,
            } if value.is_zero() => (Cmd::Call, {
                let mut b =
                    bytes::BytesMut::with_capacity(target.as_bytes().len() + calldata.len());
                b.extend(target.to_fixed_bytes());
                b.extend(calldata);
                Bytes(b.into())
            }),
            Self::Call {
                target,
                value,
                calldata,
            } => (Cmd::CallValue, {
                let value = Big(value).to_fixed_bytes();
                let mut b = bytes::BytesMut::with_capacity(
                    value.len() + target.as_bytes().len() + calldata.len(),
                );
                b.extend(value);
                b.extend(target.to_fixed_bytes());
                b.extend(calldata);
                Bytes(b.into())
            }),
            Self::GetBalance(BalanceOf::This) => (Cmd::GetBalanceOfThis, Default::default()),
            Self::GetBalance(BalanceOf::MsgSender) => {
                (Cmd::GetBalanceOfMsgSender, Default::default())
            }
            Self::GetBalance(BalanceOf::Address(target)) => (
                Cmd::GetBalanceOfAddress,
                Bytes(target.to_fixed_bytes().into_iter().collect()),
            ),
            Self::Transfer { to, amount } => Self::Call {
                target: to,
                value: amount,
                calldata: Default::default(),
            }
            .encode(),
            Self::Create { value, bytecode } if value.is_zero() => (Cmd::Create, bytecode),
            Self::Create { value, bytecode } => (Cmd::CreateValue, {
                let value = Big(value).to_fixed_bytes();
                let mut b = bytes::BytesMut::with_capacity(value.len() + bytecode.len());
                b.extend(value);
                b.extend(bytecode);
                Bytes(b.into())
            }),
            Self::Create2 {
                value,
                salt,
                bytecode,
            } if value.is_zero() => (Cmd::Create2, {
                let salt = Big(salt).to_fixed_bytes();
                let mut b = bytes::BytesMut::with_capacity(salt.len() + bytecode.len());
                b.extend(salt);
                b.extend(bytecode);
                Bytes(b.into())
            }),
            Self::Create2 {
                value,
                salt,
                bytecode,
            } => (Cmd::Create2Value, {
                let value = Big(value).to_fixed_bytes();
                let salt = Big(salt).to_fixed_bytes();
                let mut b =
                    bytes::BytesMut::with_capacity(value.len() + salt.len() + bytecode.len());
                b.extend(value);
                b.extend(salt);
                b.extend(bytecode);
                Bytes(b.into())
            }),
            Self::Group(calls) => (Cmd::Group, MultiCallWrapper(calls).encode().into()),
        }
    }

    fn decode(cmd: Cmd, inputs: Bytes) -> Result<Self, AbiError> {
        let mut inputs = inputs.as_ref();
        Ok(match cmd {
            Cmd::Group => Self::Group(MultiCallWrapper::decode(inputs)?.into_inner()),
            Cmd::GetBalanceOfThis => Self::GetBalance(BalanceOf::This),
            Cmd::GetBalanceOfMsgSender => Self::GetBalance(BalanceOf::MsgSender),
            Cmd::GetBalanceOfAddress => Self::GetBalance(BalanceOf::Address(
                <[u8; 20]>::try_from(inputs)
                    .map_err(|_| AbiError::DecodingError(ethabi::Error::InvalidData))?
                    .into(),
            )),
            _ => {
                let value = if let Cmd::CallValue | Cmd::CreateValue | Cmd::Create2Value = cmd {
                    let value = <[u8; 32]>::try_from(inputs)
                        .map_err(|_| AbiError::DecodingError(ethabi::Error::InvalidData))?
                        .into();
                    inputs = &inputs[32..];
                    value
                } else {
                    U256::zero()
                };
                match cmd {
                    Cmd::Call | Cmd::CallValue => {
                        let target: Address = <[u8; 20]>::try_from(inputs)
                            .map_err(|_| AbiError::DecodingError(ethabi::Error::InvalidData))?
                            .into();
                        inputs = &inputs[20..];
                        match cmd {
                            Cmd::CallValue if inputs.is_empty() => Self::Transfer {
                                to: target,
                                amount: value,
                            },
                            _ => Self::Call {
                                target,
                                value,
                                calldata: inputs.to_vec().into(),
                            },
                        }
                    }
                    Cmd::Create | Cmd::CreateValue => Self::Create {
                        value,
                        bytecode: inputs.to_vec().into(),
                    },
                    Cmd::Create2 | Cmd::Create2Value => {
                        let salt: U256 = <[u8; 32]>::try_from(inputs)
                            .map_err(|_| AbiError::DecodingError(ethabi::Error::InvalidData))?
                            .into();
                        inputs = &inputs[32..];
                        Self::Create2 {
                            value,
                            salt,
                            bytecode: inputs.to_vec().into(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
        })
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum Cmd {
    Group = 0,
    Call = 1,
    CallValue = 2, // also used for Transfer
    GetBalanceOfThis = 3,
    GetBalanceOfMsgSender = 4,
    GetBalanceOfAddress = 5,
    Create = 6,
    CreateValue = 7,
    Create2 = 8,
    Create2Value = 9,
}

impl Cmd {
    const ALLOW_FAILURE: u8 = 1 << 7;

    fn try_from_allow_failure(cmd: u8) -> Option<(Self, bool)> {
        Some((
            match cmd & !Self::ALLOW_FAILURE {
                0 => Self::Group,
                1 => Self::Call,
                2 => Self::CallValue,
                3 => Self::GetBalanceOfThis,
                4 => Self::GetBalanceOfMsgSender,
                5 => Self::GetBalanceOfAddress,
                6 => Self::Create,
                7 => Self::CreateValue,
                8 => Self::Create2,
                9 => Self::Create2Value,
                _ => return None,
            },
            (cmd & Self::ALLOW_FAILURE != 0),
        ))
    }

    fn with_allow_failure(self, allow_failure: bool) -> u8 {
        let mut cmd = self as u8;
        if allow_failure {
            cmd |= Self::ALLOW_FAILURE
        }
        cmd
    }
}

#[derive(Clone)]
pub struct TryCall {
    allow_failure: bool,
    call: Call,
}

impl TryCall {
    pub fn reduce(self) -> Self {
        match self.call {
            Call::Group(mut calls) if calls.len() == 1 => {
                let inner = calls.remove(0).reduce();
                return TryCall {
                    allow_failure: self.allow_failure || inner.allow_failure,
                    call: inner.call,
                };
            }
            _ => self,
        }
    }

    fn encode(self) -> (u8, Bytes) {
        let (cmd, inputs) = self.call.encode();
        (cmd.with_allow_failure(self.allow_failure), inputs)
    }

    fn decode(cmd: u8, inputs: Bytes) -> Result<Self, AbiError> {
        let (cmd, allow_failure) = Cmd::try_from_allow_failure(cmd)
            .ok_or(AbiError::DecodingError(ethabi::Error::InvalidData))?;
        Ok(Self {
            allow_failure,
            call: Call::decode(cmd, inputs)?,
        })
    }
}

type MultiCall = Vec<TryCall>;

#[derive(Clone)]
struct MultiCallWrapper(MultiCall);

impl FromIterator<TryCall> for MultiCallWrapper {
    fn from_iter<T: IntoIterator<Item = TryCall>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl MultiCallWrapper {
    pub fn reduce(self) -> Self {
        MultiCallWrapper(self.0.into_iter().map(TryCall::reduce).collect())
    }

    fn encode_call(self) -> MulticallCall {
        let (commands, inputs): (Vec<u8>, Vec<Bytes>) =
            self.0.into_iter().map(TryCall::encode).unzip();
        MulticallCall {
            commands: commands.into(),
            inputs,
        }
    }

    fn decode_call(call: MulticallCall) -> Result<Self, AbiError> {
        call.commands
            .into_iter()
            .zip(call.inputs)
            .map(|(cmd, inputs)| TryCall::decode(cmd, inputs))
            .try_collect()
    }

    fn into_inner(self) -> Vec<TryCall> {
        self.0
    }
}

impl AbiType for MultiCallWrapper {
    fn param_type() -> ParamType {
        MulticallCall::param_type()
    }
}

impl AbiEncode for MultiCallWrapper {
    fn encode(self) -> Vec<u8> {
        self.encode_call().encode()
    }
}

impl AbiDecode for MultiCallWrapper {
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
        Self::decode_call(MulticallCall::decode(bytes)?)
    }
}

struct Big<T>(T);

impl<T> From<T> for Big<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl Big<U256> {
    fn to_fixed_bytes(&self) -> [u8; 32] {
        let mut a = [0; 32];
        self.0.to_big_endian(&mut a);
        a
    }
}

pub enum CallResult {
    Call(Result<(), Bytes>),
    Balance(U256),
    CreatedAt(Result<Address, Bytes>),
    Group(Result<Vec<CallResult>, FailedCall>),
}

pub struct MultiCallFailed {
    index: usize,
    reason: Failed,
}

pub enum Failed {
    MultiCall(MultiCallFailed),
    External(Bytes),
}

pub enum MultiCallError {
    Failed{
        index: usize,
        data: Bytes,
    },
    Other(Bytes),
}

pub type MultiCallOutput = Result<Vec<CallOutput>>;

pub struct MultiCallOutputWrapper(MultiCallOutput);

// pub enum CallError {
//
// }
//
// pub struct CallResult(Result<CallOutput, CallError>);

// pub struct Failed<T> {
//     stack: Vec<usize>,
//     data: T,
// }

// pub struct CallResult<T, E>(Result<T, Failed<E>>);
//
// pub struct MultiCallResults<T, E>(Vec<CallResult<T, E>>);

// impl<T, E> AbiType for MultiCallResults<T, E> {
//     fn param_type() -> ParamType {
//         ParamType::Tuple([ParamType::Bytes, ParamType::Array(ParamType::Bytes.into())].into())
//     }
// }
//
// impl<T, E> Tokenizable for MultiCallResults<T, E> {
//     fn from_token(token: Token) -> Result<Self, InvalidOutputType>
//     where
//         Self: Sized {
//         todo!()
//     }
//
//     fn into_token(self) -> Token {
//         todo!()
//     }
// }
//
// impl<T, E> AbiEncode for MultiCallResults<T, E> {
//     fn encode(self) -> Vec<u8> {
//         let token = self.into_token();
//         abi::encode(&[token])
//     }
// }
//
// impl<T, E> AbiDecode for MultiCallResults<T, E> {
//
// }

// pub struct MultiCallContract<M: Middleware>(MultiCallInner<M>);
//
// impl<M: Middleware> MultiCallContract<M> {
//     pub fn new(address: impl Into<Address>, client: impl Into<Arc<M>>) -> Self {
//         Self(MultiCallInner::new(address, client.into()))
//     }
//
//     pub fn multicall(&self, calls: Vec<TryCall>) -> MultiCallCall<M> {
//         MultiCallCall(
//             self.0
//                 .method_hash(MulticallCall::selector(), MultiCall(calls))
//                 .unwrap(),
//         )
//     }
//     // pub fn builder(&self) -> MultiCallBuilder<M> {
//     //     self.0.address
//     // }
// }
//
// pub struct MultiCallCall<M: Middleware>(ContractCall<M, MultiCallResults<T, E>>);
//
// impl<M: Middleware> MultiCallCall<M> {
//     pub async fn call(&self) -> Result<MultiCallResults, ContractError<M>> {
//         self.0.call().await
//     }
// }
