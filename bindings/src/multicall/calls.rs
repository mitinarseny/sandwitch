use core::convert::Infallible;

use ethers::{
    abi::{AbiDecode, AbiError},
    types::{Address, Selector, U256},
};

use super::errors::{InvalidLength, UnsupportedCommand, WrongCommand};
use crate::prelude::*;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
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

impl TryFrom<u8> for Cmd {
    type Error = UnsupportedCommand;

    fn try_from(cmd: u8) -> Result<Self, Self::Error> {
        Ok(match cmd {
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
            _ => return Err(UnsupportedCommand(cmd)),
        })
    }
}

pub trait Call: Sized {
    type Meta;

    type Ok;
    type Reverted;

    fn encode(self) -> (Cmd, bytes::Bytes, Self::Meta);
    fn encode_raw(self) -> (RawCall, Self::Meta) {
        let (cmd, input, meta) = self.encode();
        (RawCall(cmd, input), meta)
    }
    fn decode(cmd: Cmd, input: bytes::Bytes) -> Result<Self, AbiError>;
    fn decode_raw(c: RawCall) -> Result<Self, AbiError> {
        let RawCall(cmd, input) = c;
        Self::decode(cmd, input)
    }

    fn decode_ok(output: bytes::Bytes, meta: Self::Meta) -> Result<Self::Ok, AbiError>;
    fn decode_reverted(output: bytes::Bytes, meta: Self::Meta) -> Result<Self::Reverted, AbiError>;
}

pub struct RawCall(Cmd, bytes::Bytes);
pub type RawResult = Result<bytes::Bytes, bytes::Bytes>;

impl Call for RawCall {
    type Meta = ();
    type Ok = bytes::Bytes;
    type Reverted = bytes::Bytes;

    fn encode(self) -> (Cmd, bytes::Bytes, Self::Meta) {
        (self.0, self.1, ())
    }

    fn decode(cmd: Cmd, input: bytes::Bytes) -> Result<Self, AbiError> {
        Ok(Self(cmd, input))
    }

    fn decode_ok(output: bytes::Bytes, _meta: Self::Meta) -> Result<Self::Ok, AbiError> {
        Ok(output)
    }

    fn decode_reverted(
        output: bytes::Bytes,
        _meta: Self::Meta,
    ) -> Result<Self::Reverted, AbiError> {
        Ok(output)
    }
}

#[derive(Clone)]
pub struct ContractCall<C> {
    pub target: Address,
    pub value: U256,
    pub call: C,
}

impl<C: EthTypedCall> ContractCall<C> {
    fn try_from_raw(call: ContractCall<bytes::Bytes>) -> Result<Self, AbiError> {
        let ContractCall {
            target,
            value,
            mut call,
        } = call;
        let selector = Selector::try_read_from(&mut call).map_err(|_| InvalidLength)?;
        if selector != C::selector() {
            return Err(AbiError::WrongSelector);
        }
        Ok(Self {
            target,
            value,
            call: <C as AbiDecode>::decode(call)?,
        })
    }
}

impl<C> Call for ContractCall<C>
where
    C: EthTypedCall,
{
    type Meta = <ContractCall<bytes::Bytes> as Call>::Meta;

    type Ok = C::Ok;
    type Reverted = C::Reverted;

    fn encode(self) -> (Cmd, bytes::Bytes, Self::Meta) {
        ContractCall::<bytes::Bytes>::from(self).encode()
    }

    fn decode(cmd: Cmd, input: bytes::Bytes) -> Result<Self, AbiError> {
        Self::try_from_raw(ContractCall::<bytes::Bytes>::decode(cmd, input)?)
    }

    fn decode_ok(output: bytes::Bytes, _meta: Self::Meta) -> Result<Self::Ok, AbiError> {
        <C::Ok as AbiDecode>::decode(output)
    }

    fn decode_reverted(
        output: bytes::Bytes,
        _meta: Self::Meta,
    ) -> Result<Self::Reverted, AbiError> {
        <C::Reverted as AbiDecode>::decode(output)
    }
}

impl<C: EthTypedCall> From<ContractCall<C>> for ContractCall<bytes::Bytes> {
    fn from(call: ContractCall<C>) -> Self {
        let ContractCall {
            target,
            value,
            call,
        } = call;
        let (selector, calldata) = (C::selector(), call.encode());
        ContractCall {
            target,
            value,
            call: bytes::BytesMut::with_capacity(Selector::len_bytes() + calldata.len())
                .write(selector.as_slice())
                .write(calldata.as_slice())
                .into(),
        }
    }
}

impl From<ContractCall<()>> for ContractCall<bytes::Bytes> {
    fn from(call: ContractCall<()>) -> Self {
        let ContractCall { target, value, .. } = call;
        Self {
            target,
            value,
            call: Default::default(),
        }
    }
}

type RawContractCall = ContractCall<bytes::Bytes>;

impl Call for RawContractCall {
    type Meta = ();
    type Ok = bytes::Bytes;
    type Reverted = bytes::Bytes;

    fn encode(self) -> (Cmd, bytes::Bytes, Self::Meta) {
        let Self {
            target,
            value,
            call: calldata,
        } = self;
        let mut input = bytes::BytesMut::with_capacity(
            if value.is_zero() {
                0
            } else {
                U256::len_bytes()
            } + Address::len_bytes()
                + calldata.len(),
        );
        (
            if value.is_zero() {
                Cmd::Call
            } else {
                input = input.write_big(value);
                Cmd::CallValue
            },
            input
                .write(target.to_fixed_bytes().as_slice())
                .write(calldata)
                .into(),
            (),
        )
    }

    fn decode(cmd: Cmd, mut input: bytes::Bytes) -> Result<Self, AbiError> {
        let value = match cmd {
            Cmd::Call => 0.into(),
            Cmd::CallValue => U256::try_read_from(&mut input).map_err(|_| InvalidLength)?,
            _ => return Err(WrongCommand.into()),
        };

        let target = Address::try_read_from(&mut input).map_err(|_| InvalidLength)?;
        Ok(Self {
            target,
            value,
            call: input,
        })
    }

    fn decode_ok(output: bytes::Bytes, _meta: Self::Meta) -> Result<Self::Ok, AbiError> {
        Ok(output)
    }

    fn decode_reverted(
        output: bytes::Bytes,
        _meta: Self::Meta,
    ) -> Result<Self::Reverted, AbiError> {
        Ok(output)
    }
}

pub type Transfer = ContractCall<()>;

impl Transfer {
    pub fn transafer(to: Address, amount: impl Into<U256>) -> Self {
        Self {
            target: to,
            value: amount.into(),
            call: (),
        }
    }

    fn try_from_raw(call: ContractCall<bytes::Bytes>) -> Result<Self, AbiError> {
        let ContractCall {
            target,
            value,
            call: calldata,
        } = call;
        if !calldata.is_empty() {
            return Err(InvalidLength.into());
        }
        Ok(Self {
            target,
            value,
            call: (),
        })
    }
}

impl Call for Transfer {
    type Meta = <ContractCall<bytes::Bytes> as Call>::Meta;
    type Ok = ();
    type Reverted = <ContractCall<bytes::Bytes> as Call>::Reverted;

    fn encode(self) -> (Cmd, bytes::Bytes, Self::Meta) {
        ContractCall::<bytes::Bytes>::from(self).encode()
    }

    fn decode(cmd: Cmd, input: bytes::Bytes) -> Result<Self, AbiError> {
        Self::try_from_raw(ContractCall::<bytes::Bytes>::decode(cmd, input)?)
    }

    fn decode_ok(output: bytes::Bytes, _meta: Self::Meta) -> Result<Self::Ok, AbiError> {
        if !output.is_empty() {
            return Err(InvalidLength.into());
        }
        Ok(())
    }

    fn decode_reverted(output: bytes::Bytes, meta: Self::Meta) -> Result<Self::Reverted, AbiError> {
        <ContractCall<bytes::Bytes> as Call>::decode_reverted(output, meta)
    }
}

#[derive(Clone, Copy)]
pub enum GetBalanceOf {
    This,
    MsgSender,
    Address(Address),
}

impl Call for GetBalanceOf {
    type Meta = ();
    type Ok = U256;
    type Reverted = Infallible;

    fn encode(self) -> (Cmd, bytes::Bytes, Self::Meta) {
        match self {
            GetBalanceOf::This => (Cmd::GetBalanceOfThis, Default::default(), ()),
            GetBalanceOf::MsgSender => (Cmd::GetBalanceOfMsgSender, Default::default(), ()),
            GetBalanceOf::Address(addr) => (
                Cmd::GetBalanceOfAddress,
                bytes::BytesMut::with_capacity(Address::len_bytes())
                    .write(addr.to_fixed_bytes().as_slice())
                    .into(),
                (),
            ),
        }
    }

    fn decode(cmd: Cmd, mut input: bytes::Bytes) -> Result<Self, AbiError> {
        Ok(match cmd {
            Cmd::GetBalanceOfThis => Self::This,
            Cmd::GetBalanceOfMsgSender => Self::MsgSender,
            Cmd::GetBalanceOfAddress => {
                Self::Address(Address::try_read_from(&mut input).map_err(|_| InvalidLength)?)
            }
            _ => return Err(WrongCommand.into()),
        })
    }

    fn decode_ok(output: bytes::Bytes, _meta: Self::Meta) -> Result<Self::Ok, AbiError> {
        U256::decode(output)
    }

    fn decode_reverted(
        _output: bytes::Bytes,
        _meta: Self::Meta,
    ) -> Result<Self::Reverted, AbiError> {
        unreachable!()
    }
}

#[derive(Clone)]
pub struct Create {
    pub value: U256,
    pub bytecode: bytes::Bytes,
}

impl Call for Create {
    type Meta = ();
    type Ok = Address;
    type Reverted = bytes::Bytes;

    fn encode(self) -> (Cmd, bytes::Bytes, Self::Meta) {
        let Self { value, bytecode } = self;
        if value.is_zero() {
            return (Cmd::Create, bytecode, ());
        }

        (
            Cmd::CreateValue,
            bytes::BytesMut::with_capacity(U256::len_bytes() + bytecode.len())
                .write_big(value)
                .write(bytecode)
                .into(),
            (),
        )
    }

    fn decode(cmd: Cmd, mut input: bytes::Bytes) -> Result<Self, AbiError> {
        Ok(match cmd {
            Cmd::Create | Cmd::CreateValue => Self {
                value: if cmd == Cmd::CreateValue {
                    U256::try_read_from(&mut input).map_err(|_| InvalidLength)?
                } else {
                    0.into()
                },
                bytecode: input,
            },
            _ => return Err(WrongCommand.into()),
        })
    }

    fn decode_ok(mut output: bytes::Bytes, _meta: Self::Meta) -> Result<Self::Ok, AbiError> {
        Address::try_read_from(&mut output)
            .map_err(|_| InvalidLength)
            .map_err(Into::into)
    }

    fn decode_reverted(
        output: bytes::Bytes,
        _meta: Self::Meta,
    ) -> Result<Self::Reverted, AbiError> {
        Ok(output)
    }
}

#[derive(Clone)]
pub struct Create2 {
    pub value: U256,
    pub salt: U256,
    pub bytecode: bytes::Bytes,
}

impl Call for Create2 {
    type Meta = ();
    type Ok = Address;
    type Reverted = bytes::Bytes;

    fn encode(self) -> (Cmd, bytes::Bytes, Self::Meta) {
        let Self {
            value,
            salt,
            bytecode,
        } = self;
        let mut input = bytes::BytesMut::with_capacity(
            if value.is_zero() {
                0
            } else {
                U256::len_bytes()
            } + U256::len_bytes()
                + bytecode.len(),
        );
        (
            if value.is_zero() {
                Cmd::Create2
            } else {
                input = input.write_big(value);
                Cmd::Create2Value
            },
            input.write_big(salt).write(bytecode).into(),
            (),
        )
    }

    fn decode(cmd: Cmd, mut input: bytes::Bytes) -> Result<Self, AbiError> {
        Ok(match cmd {
            Cmd::Create2 | Cmd::Create2Value => Self {
                value: if cmd == Cmd::CreateValue {
                    U256::try_read_from(&mut input).map_err(|_| InvalidLength)?
                } else {
                    0.into()
                },
                salt: U256::try_read_from(&mut input).map_err(|_| InvalidLength)?,
                bytecode: input,
            },
            _ => return Err(WrongCommand.into()),
        })
    }

    fn decode_ok(mut output: bytes::Bytes, _meta: Self::Meta) -> Result<Self::Ok, AbiError> {
        Address::try_read_from(&mut output)
            .map_err(|_| InvalidLength)
            .map_err(Into::into)
    }

    fn decode_reverted(
        output: bytes::Bytes,
        _meta: Self::Meta,
    ) -> Result<Self::Reverted, AbiError> {
        Ok(output)
    }
}

#[derive(Clone)]
pub struct TryCall<C> {
    pub allow_failure: bool,
    pub call: C,
}

impl<C: Call> TryCall<C> {
    const ALLOW_FAILURE: u8 = 1 << 7;

    pub fn encode(self) -> (u8, bytes::Bytes, C::Meta) {
        let (cmd, input, meta) = self.call.encode();
        let mut cmd = cmd as u8;
        if self.allow_failure {
            cmd |= Self::ALLOW_FAILURE;
        }
        (cmd, input, meta)
    }
    pub fn encode_raw(self) -> (TryCall<RawCall>, C::Meta) {
        let Self {
            allow_failure,
            call,
        } = self;
        let (call, meta) = call.encode_raw();
        (
            TryCall {
                allow_failure,
                call,
            },
            meta,
        )
    }

    pub fn decode(cmd: u8, input: bytes::Bytes) -> Result<Self, AbiError> {
        Ok(Self {
            allow_failure: cmd & Self::ALLOW_FAILURE != 0,
            call: C::decode((cmd & !Self::ALLOW_FAILURE).try_into()?, input)?,
        })
    }
    pub fn decode_raw(c: TryCall<RawCall>) -> Result<Self, AbiError> {
        let TryCall {
            allow_failure,
            call,
        } = c;
        Ok(Self {
            allow_failure,
            call: C::decode_raw(call)?,
        })
    }

    pub fn into_call(self) -> C {
        self.call
    }
}

pub type Calls<C> = Vec<TryCall<C>>;
pub type DynCalls = Calls<DynCall>;

#[derive(Clone)]
pub enum DynCall {
    ContractCall(RawContractCall),
    GetBalanceOf(GetBalanceOf),
    Transfer(Transfer),
    Create(Create),
    Create2(Create2),
    Group(DynCalls),
}

impl<C: EthTypedCall> From<ContractCall<C>> for DynCall {
    fn from(c: ContractCall<C>) -> Self {
        Self::ContractCall(c.into())
    }
}

impl From<RawContractCall> for DynCall {
    fn from(c: RawContractCall) -> Self {
        Self::ContractCall(c)
    }
}

impl From<GetBalanceOf> for DynCall {
    fn from(c: GetBalanceOf) -> Self {
        Self::GetBalanceOf(c)
    }
}

impl From<Transfer> for DynCall {
    fn from(c: Transfer) -> Self {
        Self::Transfer(c)
    }
}

impl From<Create> for DynCall {
    fn from(c: Create) -> Self {
        Self::Create(c)
    }
}

impl From<Create2> for DynCall {
    fn from(c: Create2) -> Self {
        Self::Create2(c)
    }
}

impl FromIterator<TryCall<Self>> for DynCall {
    fn from_iter<T: IntoIterator<Item = TryCall<Self>>>(calls: T) -> Self {
        Self::Group(calls.into_iter().collect())
    }
}

#[derive(Clone)]
pub enum DynCallType {
    Call,
    GetBalanceOf,
    Transfer,
    Create,
    Group(Vec<Self>),
}

pub enum DynCallOutput {
    ContractCalled(<RawContractCall as Call>::Ok),
    Balance(<GetBalanceOf as Call>::Ok),
    Transferred,
    ContractCreated(<Create as Call>::Ok),
    Group(<Calls<DynCall> as Call>::Ok),
}

pub enum DynCallReverted {
    ContractCallReverted(<RawContractCall as Call>::Reverted),
    TransferReverted(<Transfer as Call>::Reverted),
    ContractCreateReverted(<Create as Call>::Reverted),
    GroupReverted(Box<<DynCalls as Call>::Reverted>),
}

impl Call for DynCall {
    type Meta = DynCallType;
    type Ok = DynCallOutput;
    type Reverted = DynCallReverted;

    fn encode(self) -> (Cmd, bytes::Bytes, Self::Meta) {
        match self {
            Self::ContractCall(c) => {
                let (cmd, input, _meta) = c.encode();
                (cmd, input, DynCallType::Call)
            }
            Self::GetBalanceOf(c) => {
                let (cmd, input, _meta) = c.encode();
                (cmd, input, DynCallType::GetBalanceOf)
            }
            Self::Transfer(c) => {
                let (cmd, input, _meta) = c.encode();
                (cmd, input, DynCallType::Transfer)
            }
            Self::Create(c) => {
                let (cmd, input, _meta) = c.encode();
                (cmd, input, DynCallType::Create)
            }
            Self::Create2(c) => {
                let (cmd, input, _meta) = c.encode();
                (cmd, input, DynCallType::Create)
            }
            Self::Group(c) => {
                let (cmd, inputs, call_types) = <DynCalls as Call>::encode(c);
                (cmd, inputs, DynCallType::Group(call_types))
            }
        }
    }

    fn decode(cmd: Cmd, input: bytes::Bytes) -> Result<Self, AbiError> {
        Ok(match cmd {
            Cmd::Group => Self::Group(Call::decode(cmd, input)?),
            Cmd::Call | Cmd::CallValue => {
                let r = ContractCall::<bytes::Bytes>::decode(cmd, input)?;
                if r.call.is_empty() {
                    Self::Transfer(Transfer::try_from_raw(r)?)
                } else {
                    Self::ContractCall(r)
                }
            }
            Cmd::GetBalanceOfThis | Cmd::GetBalanceOfMsgSender | Cmd::GetBalanceOfAddress => {
                Self::GetBalanceOf(Call::decode(cmd, input)?)
            }
            Cmd::Create | Cmd::CreateValue => Self::Create(Call::decode(cmd, input)?),
            Cmd::Create2 | Cmd::Create2Value => Self::Create2(Call::decode(cmd, input)?),
        })
    }

    fn decode_ok(output: bytes::Bytes, call_type: Self::Meta) -> Result<Self::Ok, AbiError> {
        Ok(match call_type {
            DynCallType::Call => {
                DynCallOutput::ContractCalled(<RawContractCall as Call>::decode_ok(output, ())?)
            }
            DynCallType::GetBalanceOf => {
                DynCallOutput::Balance(<GetBalanceOf as Call>::decode_ok(output, ())?)
            }
            DynCallType::Transfer => {
                let () = <Transfer as Call>::decode_ok(output, ())?;
                DynCallOutput::Transferred
            }
            DynCallType::Create => {
                DynCallOutput::ContractCreated(<Create as Call>::decode_ok(output, ())?)
            }
            DynCallType::Group(call_types) => {
                DynCallOutput::Group(<DynCalls as Call>::decode_ok(output, call_types)?)
            }
        })
    }

    fn decode_reverted(
        output: bytes::Bytes,
        call_type: Self::Meta,
    ) -> Result<Self::Reverted, AbiError> {
        Ok(match call_type {
            DynCallType::Call => DynCallReverted::ContractCallReverted(
                <RawContractCall as Call>::decode_reverted(output, ())?,
            ),
            DynCallType::GetBalanceOf => return Err(WrongCommand.into()),
            DynCallType::Transfer => {
                DynCallReverted::TransferReverted(<Transfer as Call>::decode_reverted(output, ())?)
            }
            DynCallType::Create => DynCallReverted::ContractCreateReverted(
                <Create as Call>::decode_reverted(output, ())?,
            ),
            DynCallType::Group(call_types) => DynCallReverted::GroupReverted(
                <DynCalls as Call>::decode_reverted(output, call_types)?.into(),
            ),
        })
    }
}
