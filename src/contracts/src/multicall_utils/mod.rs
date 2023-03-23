use arrow2::bitmap::{chunk_iter_to_vec, Bitmap, MutableBitmap};
use bytes::{Buf, BufMut};
use core::convert;
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
use std::{array::TryFromSliceError, ops::Index};

#[path = "../multicall/mod.rs"]
mod inner;
use inner::multi_call::MultiCall as MultiCallInner;

use self::inner::multi_call::{
    Failed as FailedRaw, MulticallCall as MultiCallRaw, MulticallReturn,
};

#[derive(Clone, Debug)]
pub enum CallType {
    Call,
    GetBalance,
    Create,
    Group(Vec<CallType>),
}

#[derive(Clone)]
pub enum Call {
    Call {
        target: Address,
        value: U256,
        calldata: bytes::Bytes,
    },
    GetBalance(BalanceOf),
    Transfer {
        to: Address,
        amount: U256,
    },
    Create {
        value: U256,
        bytecode: bytes::Bytes,
    },
    Create2 {
        value: U256,
        salt: U256,
        bytecode: bytes::Bytes,
    },
    Group(MultiCall),
}

type MultiCall = Vec<TryCall>;

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
                C::selector().into_iter().chain(inputs).collect()
            },
        }
    }

    fn call_type(&self) -> CallType {
        match self {
            Self::Call | Self::Transfer => CallType::Call,
            Self::Create | Self::Create2 => CallType::Create,
            Call::Group(calls) => calls.iter().map(Self::call_type).collect(),
        }
    }

    fn encode(self) -> (Cmd, bytes::Bytes) {
        match self {
            Self::Call {
                target,
                value,
                calldata,
            } if value.is_zero() => (
                Cmd::Call,
                bytes::BytesMut::with_capacity(target.as_bytes().len() + calldata.len())
                    .write(target.to_fixed_bytes())
                    .write(calldata)
                    .into(),
            ),
            Self::Call {
                target,
                value,
                calldata,
            } => (
                Cmd::CallValue,
                bytes::BytesMut::with_capacity(
                    U256::len_bytes() + Address::len_bytes() + calldata.len(),
                )
                .write_big(value)
                .write(target.to_fixed_bytes())
                .write(calldata)
                .into(),
            ),
            Self::GetBalance(BalanceOf::This) => (Cmd::GetBalanceOfThis, Default::default()),
            Self::GetBalance(BalanceOf::MsgSender) => {
                (Cmd::GetBalanceOfMsgSender, Default::default())
            }
            Self::GetBalance(BalanceOf::Address(target)) => (
                Cmd::GetBalanceOfAddress,
                target.to_fixed_bytes().into_iter().collect(),
            ),
            Self::Transfer { to, amount } => Self::Call {
                target: to,
                value: amount,
                calldata: Default::default(),
            }
            .encode(),
            Self::Create { value, bytecode } if value.is_zero() => (Cmd::Create, bytecode),
            Self::Create { value, bytecode } => (
                Cmd::CreateValue,
                bytes::BytesMut::with_capacity(U256::len_bytes() + bytecode.len())
                    .write_big(value)
                    .write(bytecode)
                    .into(),
            ),
            Self::Create2 {
                value,
                salt,
                bytecode,
            } if value.is_zero() => (
                Cmd::Create2,
                bytes::BytesMut::with_capacity(U256::len_bytes() + bytecode.len())
                    .write_big(salt)
                    .write(bytecode)
                    .into(),
            ),
            Self::Create2 {
                value,
                salt,
                bytecode,
            } => (
                Cmd::Create2Value,
                bytes::BytesMut::with_capacity(2 * U256::len_bytes() + bytecode.len())
                    .write_big(value)
                    .write_big(salt)
                    .write(bytecode)
                    .into(),
            ),
            Self::Group(calls) => (Cmd::Group, MultiCallWrapper(calls).encode().into()),
        }
    }

    fn decode(cmd: Cmd, mut inputs: bytes::Bytes) -> Result<Self, InvalidOutputType> {
        Ok(match cmd {
            Cmd::Group => Self::Group(
                MultiCallWrapper::decode(inputs)
                    .map_err(|e| InvalidOutputType(e.to_string()))?
                    .into_inner(),
            ),
            Cmd::GetBalanceOfThis => Self::GetBalance(BalanceOf::This),
            Cmd::GetBalanceOfMsgSender => Self::GetBalance(BalanceOf::MsgSender),
            Cmd::GetBalanceOfAddress => Self::GetBalance(BalanceOf::Address(
                Address::try_from_slice(inputs)
                    .map_err(|_| InvalidOutputType(format!("invalid inputs length for ")))?,
            )),
            _ => {
                let value = if let Cmd::CallValue | Cmd::CreateValue | Cmd::Create2Value = cmd {
                    U256::try_read_from(&mut inputs)
                        .map_err(|_| AbiError::DecodingError(ethabi::Error::InvalidData))?;
                } else {
                    U256::zero()
                };
                match cmd {
                    Cmd::Call | Cmd::CallValue => {
                        let target = Address::try_read_from(&mut inputs)
                            .map_err(|_| AbiError::DecodingError(ethabi::Error::InvalidData))?;
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
                        let salt = U256::try_read_from(&mut inputs)
                            .map_err(|_| AbiError::DecodingError(ethabi::Error::InvalidData))?;
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

    fn encode(self) -> (u8, bytes::Bytes) {
        let (cmd, inputs) = self.call.encode();
        (cmd.with_allow_failure(self.allow_failure), inputs)
    }

    fn decode(cmd: u8, inputs: bytes::Bytes) -> Result<Self, InvalidOutputType> {
        let (cmd, allow_failure) = Cmd::try_from_allow_failure(cmd)
            .ok_or(AbiError::DecodingError(ethabi::Error::InvalidData))?;
        Ok(Self {
            allow_failure,
            call: Call::decode(cmd, inputs)?,
        })
    }
}

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

    fn encode_raw(self) -> MultiCallRaw {
        let (commands, inputs): (Vec<u8>, Vec<bytes::Bytes>) =
            self.0.into_iter().map(TryCall::encode).unzip();
        MultiCallRaw {
            commands: commands.into(),
            inputs: inputs.into_iter().map(Into::into).collect(),
        }
    }

    fn decode_raw(call: MultiCallRaw) -> Result<Self, InvalidOutputType> {
        call.commands
            .into_iter()
            .zip(call.inputs)
            .map(|(cmd, inputs)| TryCall::decode(cmd, inputs.0))
            .try_collect()
    }

    fn into_inner(self) -> Vec<TryCall> {
        self.0
    }
}

impl AbiType for MultiCallWrapper {
    fn param_type() -> ParamType {
        MultiCallRaw::param_type()
    }
}

impl Tokenizable for MultiCallWrapper {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType>
    where
        Self: Sized,
    {
        Self::decode_raw(MultiCallInner::from_token(token)?)
    }

    fn into_token(self) -> Token {
        self.encode_raw().into_token()
    }
}

impl AbiEncode for MultiCallWrapper {
    fn encode(self) -> Vec<u8> {
        abi::encode(&Self::into_tokens(self))
    }
}

impl AbiDecode for MultiCallWrapper {
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
        let tokens = abi::decode(&[Self::param_type()], bytes.as_ref())?;
        Self::from_tokens(tokens).map_err(Into::into)
    }
}

trait TryFromSlice<const N: usize>: for<'a> From<&'a [u8; N]> {
    const fn len_bytes() -> usize {
        N
    }
    fn try_from_slice(slice: impl AsRef<[u8]>) -> Result<Self, TryFromSliceError> {
        <[u8; N]>::try_from(slice).into()
    }

    fn try_read_from<B: Buf>(buf: &mut B) -> Result<Self, TryFromSliceError> {
        let self = Self::try_from_slice(buf.chunk())?;
        buf.advance(N);
        Ok(self)
    }
}

impl<T, const N: usize> TryFromSlice<N> for T where T: for<'a> From<&'a [u8; N]> {}

trait ChainBufMut: BufMut {
    fn write<B: Buf>(&mut self, src: B) -> &mut Self
    where
        Self: Sized,
    {
        self.put(src);
        &mut self
    }

    fn write_big<const N: usize>(&mut self, v: impl ToFixedBytes<N>) -> &mut Self {
        self.write(v.to_fixed_bytes())
    }
}

impl<B> ChainBufMut for B where B: BufMut {}

trait ToFixedBytes<const N: usize>: TryFromSlice<N> {
    fn to_fixed_bytes_in(&self, bytes: &mut [u8; N]);

    fn to_fixed_bytes(&self) -> [u8; N] {
        let mut b = [0; N];
        self.to_fixed_bytes_in(&mut b);
        b
    }
}

impl ToFixedBytes<32> for U256 {
    fn to_fixed_bytes_in(&self, bytes: &mut [u8; N]) {
        self.to_big_endian(bytes)
    }
}

pub enum CallResult {
    Call(Result<bytes::Bytes, bytes::Bytes>),
    Balance(U256),
    Create(Result<Address, bytes::Bytes>),
    Group(Result<MultiCallOutput, (usize, Failed)>),
}

pub type MultiCallOutput = Vec<CallResult>;

impl CallResult {
    // TODO: derive custom macro
    pub fn to_call(self) -> Option<Result<bytes::Bytes, bytes::Bytes>> {
        if let Self::Call(r) = self {
            Some(r)
        } else {
            None
        }
    }

    pub fn to_balance(self) -> Option<U256> {
        if let Self::Balance(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn to_create(self) -> Option<Result<Address, bytes::Bytes>> {
        if let Self::Create(r) = self {
            Some(r)
        } else {
            None
        }
    }

    pub fn to_group(self) -> Option<Result<MultiCallOutput, MultiCallFailed>> {
        if let Self::Group(r) = self {
            Some(r)
        } else {
            None
        }
    }

    fn encode(self) -> (bool, bytes::Bytes) {
        match self {
            CallResult::Call(r) => match r {
                Ok(output) => (true, output),
                Err(data) => (false, data),
            },
            CallResult::Balance(b) => (true, Big(b).to_fixed_bytes().into_iter().collect()),
            CallResult::Create(r) => match r {
                Ok(address) => (true, address.to_fixed_bytes().into_iter().collect()),
                Err(data) => (false, data),
            },
            CallResult::Group(r) => match r {
                Ok(results) => (true, MultiCallOutputWrapper(results).encode().into()),
                Err(failed) => (false, failed.encode().into()),
            },
        }
    }

    fn decode_as(call_type: CallType, success: bool, output: bytes::Bytes) -> Result<Self> {
        Ok(match call_type {
            CallType::Call => Self::Call(if success { Ok(output) } else { Err(output) }),
            CallType::GetBalance => Self::Balance(Address::try_from_slice(output)?),
            CallType::Create => Self::Create(if success {
                Ok(Address::try_from_slice(output)?)
            } else {
                Err(output)
            }),
            CallType::Group(calls) => Self::Group(if success {
                Ok(MultiCallOutputWrapper(calls).decode(output)?.into_inner())
            } else {
                Err(MultiCallFailed::decode(output)?)
            }),
        })
    }
}

pub struct MultiCallFailedWrapper((usize, Failed));

impl MultiCallFailed {
    fn into_inner(self) -> (usize, Failed) {
        self.0
    }
    fn encode_raw(self) -> FailedRaw {
        FailedRaw {
            index: self.index.into(),
            data: self.reason.encode().into(),
        }
    }

    fn decode_raw_as(call_types: Vec<CallType>, raw: FailedRaw) -> Result<Self> {
        let (index, data) = (raw.index.as_usize(), raw.data);
        if index >= call_types.len() {
            return Err("TODO");
        }
        Ok(Self {
            index,
            reason: Failed::decode_as(call_types.swap_remove(index), data)?,
        })
    }

    fn decode_as(call_types: Vec<CallType>, data: bytes::Bytes) -> Result<Self> {
        Self::decode_as_from_raw(call_types, FailedRaw::decode(data)?)
    }
}

impl AbiEncode for MultiCallFailed {
    fn encode(self) -> Vec<u8> {
        self.encode_raw().encode()
    }
}

pub enum Failed {
    External(bytes::Bytes),
    Group(Box<MultiCallFailed>),
}

impl Failed {
    fn encode(self) -> bytes::Bytes {
        match self {
            Failed::External(data) => data,
            Failed::Group(g) => g.encode().into(),
        }
    }

    fn decode_as(call_type: CallType, output: bytes::Bytes) -> Result<Self> {
        Ok(match call_type {
            CallType::Call | CallType::Create => Self::External(output),
            CallType::Group(calls) => {
                Self::Group(MultiCallFailed::decode_as(call_type, output)?.into())
            }
            CallType::GetBalance => return Err("Call {call_type:?} cannot fail"),
        })
    }
}

struct MultiCallOutputWrapper(MultiCallOutput);

impl MultiCallOutputWrapper {
    fn into_inner(self) -> MultiCallOutput {
        self.0
    }

    fn encode_raw(self) -> MulticallReturn {
        let (successes, outputs): (Bitmap, Vec<Bytes>) = self
            .0
            .into_iter()
            .map(CallResult::encode)
            .map(|(success, output)| (success, output.into()))
            .unzip();
        MulticallReturn {
            successes: chunk_iter_to_vec(successes.chunks()),
            outputs,
        }
    }
    fn decode_raw_as(call_types: Vec<CallType>, raw: MulticallReturn) -> Result<Self, AbiError> {
        let MulticallReturn { successes, outputs } = raw;
        if call_types.len() != outputs.len() {
            return Err("TODO");
        }
        if (call_types.len() + 7) / 8 != successes.len() {
            return Err("TODO");
        }
        let successes = Bitmap::from_u8_slice(successes, outputs.len());
        Ok(MultiCallOutputWrapper(
            call_types
                .into_iter()
                .zip(successes.into_iter().zip(outputs.into_iter().map(|b| b.0)))
                .map(|(c, (success, output))| CallResult::decode_as(c, success, output))
                .try_collect(),
        ))
    }

    fn decode_as(call_types: Vec<CallType>, data: bytes::Bytes) -> Result<Self, AbiError> {
        Self::decode_raw_as(call_types, MulticallReturn::decode(data)?)
    }
}

impl AbiEncode for MultiCallOutputWrapper {
    fn encode(self) -> Vec<u8> {
        self.encode_raw().encode()
    }
}

impl AbiDecode for MultiCallOutputWrapper {
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethabi::AbiError> {
        todo!()
    }
}

// TODO: tests

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
