use core::ops::Deref;

use bitvec::prelude::*;
use ethers::{
    abi::{AbiDecode, AbiEncode, AbiError},
    types::Bytes,
};
use thiserror::Error as ThisError;

use super::{
    calls::{Call, Calls, Cmd, RawCall, RawResult, TryCall},
    errors::{IndexTooBig, LengthMismatch, WrongCommand},
    raw,
};

#[derive(ThisError, Debug)]
pub enum MultiCallErrors<R> {
    #[error("reverted: {0}")]
    Reverted(R),
    #[error("length mismatch")]
    LengthMismatch,
    #[error("uncled")]
    Uncled,
    #[error("{0}")]
    RevertString(String),
}

pub trait MultiCall: Sized {
    type Meta;
    type Ok;
    type Reverted;

    fn encode_calls(self) -> (Calls<RawCall>, Self::Meta);
    fn encode_raw_calls(self) -> (raw::MulticallCall, Self::Meta) {
        let (calls, meta) = self.encode_calls();
        let (commands, inputs): (Vec<u8>, Vec<Bytes>) = calls
            .into_iter()
            .map(TryCall::encode)
            .map(|(cmd, input, _meta)| (cmd, input.into()))
            .unzip();
        (
            raw::MulticallCall {
                commands: commands.into(),
                inputs,
            },
            meta,
        )
    }

    fn decode_calls(calls: Calls<RawCall>) -> Result<Self, AbiError>;
    fn decode_raw_calls(r: raw::MulticallCall) -> Result<Self, AbiError> {
        let raw::MulticallCall { commands, inputs } = r;
        if commands.len() != inputs.len() {
            return Err(LengthMismatch.into());
        }
        Self::decode_calls(
            commands
                .into_iter()
                .zip(inputs.into_iter().map(|input| input.0))
                .map(|(cmd, input)| TryCall::decode(cmd, input))
                .try_collect()?,
        )
    }

    fn decode_ok(results: Vec<RawResult>, meta: &Self::Meta) -> Result<Self::Ok, AbiError>;
    fn decode_ok_raw(r: raw::MulticallReturn, meta: &Self::Meta) -> Result<Self::Ok, AbiError> {
        let raw::MulticallReturn { successes, outputs } = r;
        let successes: &BitSlice<_, Lsb0> =
            TryInto::try_into(successes.deref()).map_err(|_| IndexTooBig)?;
        if successes.len() != outputs.len() {
            return Err(LengthMismatch.into());
        }
        let results = successes
            .into_iter()
            .zip(outputs.into_iter().map(|o| o.0))
            .map(|(success, output)| if *success { Ok(output) } else { Err(output) })
            .collect();

        Self::decode_ok(results, meta)
    }

    fn decode_reverted(
        r: RevertedAt<bytes::Bytes>,
        meta: &Self::Meta,
    ) -> Result<Self::Reverted, AbiError>;
    fn decode_reverted_raw_errors(
        e: raw::MultiCallErrors,
        meta: &Self::Meta,
    ) -> Result<MultiCallErrors<Self::Reverted>, AbiError> {
        Ok(match e {
            raw::MultiCallErrors::Reverted(r) => {
                MultiCallErrors::Reverted(Self::decode_reverted(r.try_into()?, meta)?)
            }
            raw::MultiCallErrors::LengthMismatch(raw::LengthMismatch) => {
                MultiCallErrors::LengthMismatch
            }
            raw::MultiCallErrors::Uncled(raw::Uncled) => MultiCallErrors::Uncled,
            raw::MultiCallErrors::RevertString(s) => MultiCallErrors::RevertString(s),
        })
    }
}

impl<C: MultiCall> Call for C {
    type Meta = C::Meta;
    type Ok = C::Ok;
    type Reverted = MultiCallErrors<C::Reverted>;

    fn encode(self) -> (Cmd, bytes::Bytes, Self::Meta) {
        let (r, meta) = Self::encode_raw_calls(self);
        (Cmd::Group, r.encode().into(), meta)
    }

    fn decode(cmd: Cmd, input: bytes::Bytes) -> Result<Self, AbiError> {
        if cmd != Cmd::Group {
            return Err(WrongCommand.into());
        }
        Self::decode_raw_calls(raw::MulticallCall::decode(input)?)
    }

    fn decode_ok(output: bytes::Bytes, meta: &Self::Meta) -> Result<Self::Ok, AbiError> {
        C::decode_ok_raw(AbiDecode::decode(output)?, meta)
    }

    fn decode_reverted(
        output: bytes::Bytes,
        meta: &Self::Meta,
    ) -> Result<Self::Reverted, AbiError> {
        C::decode_reverted_raw_errors(AbiDecode::decode(output)?, meta)
    }
}

pub struct RevertedAt<R>(pub usize, pub R);

impl TryFrom<raw::Reverted> for RevertedAt<bytes::Bytes> {
    type Error = IndexTooBig;
    fn try_from(r: raw::Reverted) -> Result<Self, Self::Error> {
        let raw::Reverted {
            index,
            data: Bytes(data),
        } = r;
        if index > usize::MAX.into() {
            return Err(IndexTooBig.into());
        }
        Ok(RevertedAt(index.as_usize(), data))
    }
}

impl<C: Call> MultiCall for Calls<C> {
    type Meta = Vec<C::Meta>;
    type Ok = Vec<Result<C::Ok, C::Reverted>>;
    type Reverted = RevertedAt<C::Reverted>;

    fn encode_calls(self) -> (Calls<RawCall>, Self::Meta) {
        self.into_iter().map(TryCall::encode_raw).unzip()
    }

    fn decode_calls(calls: Calls<RawCall>) -> Result<Self, AbiError> {
        calls.into_iter().map(TryCall::decode_raw).try_collect()
    }

    fn decode_ok(results: Vec<RawResult>, metas: &Self::Meta) -> Result<Self::Ok, AbiError> {
        if results.len() != metas.len() {
            return Err(LengthMismatch.into());
        }
        results
            .into_iter()
            .zip(metas)
            .map(|(r, meta)| {
                Ok(match r {
                    Ok(output) => Ok(C::decode_ok(output, meta)?),
                    Err(output) => Err(C::decode_reverted(output, meta)?),
                })
            })
            .try_collect()
    }

    fn decode_reverted(
        r: RevertedAt<bytes::Bytes>,
        metas: &Self::Meta,
    ) -> Result<Self::Reverted, AbiError> {
        let RevertedAt(index, data) = r;
        Ok(RevertedAt(
            index,
            C::decode_reverted(data, metas.get(index).ok_or(IndexTooBig)?)?,
        ))
    }
}

macro_rules! tuple_multicall {
    (impl<$N:literal> MultiCall<Reverted = $vis:vis enum $reverted:ident>
        for ($( .$n:tt: $t:ident ),+$(,)?)) => {
        #[derive(Debug)]
        $vis enum $reverted<$( $t ),+> {
            $(
                $t($t),
            )+
        }

        impl<$( $t ),+> MultiCall for ($( TryCall<$t>, )+)
        where
            $( $t: Call ),+,
        {
            type Meta = ($( $t::Meta ),+,);
            type Ok = ($( Result<$t::Ok, $t::Reverted> ),+,);
            type Reverted = $reverted<$( $t::Reverted ),+>;

            fn encode_calls(self) -> (Calls<RawCall>, Self::Meta) {
                let calls = ($(self.$n.encode_raw(),)+);
                ([$( calls.$n.0, )+].into(), ($(calls.$n.1,)+))
            }

            fn decode_calls(calls: Calls<RawCall>) -> Result<Self, AbiError> {
                let mut calls = <[TryCall::<RawCall>; $N]>::try_from(calls)
                    .map_err(|_| LengthMismatch)?
                    .into_iter();
                Ok(($(TryCall::<$t>::decode_raw(calls.next().unwrap())?,)+))
            }

            fn decode_ok(results: Vec<RawResult>, metas: &Self::Meta) -> Result<Self::Ok, AbiError> {
                let mut results = <[RawResult; $N]>::try_from(results)
                    .map_err(|_| LengthMismatch)?
                    .into_iter();
                Ok(($(
                    match results.next().unwrap() {
                        Ok(output) => Ok($t::decode_ok(output, &metas.$n)?),
                        Err(data) => Err($t::decode_reverted(data, &metas.$n)?),
                    },
                )+))
            }

            fn decode_reverted(r: RevertedAt<bytes::Bytes>, metas: &Self::Meta) -> Result<Self::Reverted, AbiError> {
                let RevertedAt(index, data) = r;
                Ok(match index {
                    $(
                        $n => <Self::Reverted>::$t(<$t as Call>::decode_reverted(data, &metas.$n)?),
                    )+
                    _ => return Err(IndexTooBig.into()),
                })
            }
        }
    };
}

tuple_multicall!(impl<1> MultiCall<Reverted = pub enum Reverted1> for (.0: C0,));
tuple_multicall!(impl<2> MultiCall<Reverted = pub enum Reverted2> for (.0: C0, .1: C1));
tuple_multicall!(impl<3> MultiCall<Reverted = pub enum Reverted3> for (.0: C0, .1: C1, .2: C2));
tuple_multicall!(impl<4> MultiCall<Reverted = pub enum Reverted4> for (.0: C0, .1: C1, .2: C2, .3: C3));
tuple_multicall!(impl<5> MultiCall<Reverted = pub enum Reverted5> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,
));
tuple_multicall!(impl<6> MultiCall<Reverted = pub enum Reverted6> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,
));
tuple_multicall!(impl<7> MultiCall<Reverted = pub enum Reverted7> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,
));
tuple_multicall!(impl<8> MultiCall<Reverted = pub enum Reverted8> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,
));
tuple_multicall!(impl<9> MultiCall<Reverted = pub enum Reverted9> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,
));
tuple_multicall!(impl<10> MultiCall<Reverted = pub enum Reverted10> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,  .9:  C9,
));
tuple_multicall!(impl<11> MultiCall<Reverted = pub enum Reverted11> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,  .9:  C9,
    .10: C10,
));
tuple_multicall!(impl<12> MultiCall<Reverted = pub enum Reverted12> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,  .9:  C9,
    .10: C10, .11: C11,
));
tuple_multicall!(impl<13> MultiCall<Reverted = pub enum Reverted13> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,  .9:  C9,
    .10: C10, .11: C11, .12: C12,
));
tuple_multicall!(impl<14> MultiCall<Reverted = pub enum Reverted14> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,  .9:  C9,
    .10: C10, .11: C11, .12: C12, .13: C13,
));
tuple_multicall!(impl<15> MultiCall<Reverted = pub enum Reverted15> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,  .9:  C9,
    .10: C10, .11: C11, .12: C12, .13: C13, .14: C14,
));
tuple_multicall!(impl<16> MultiCall<Reverted = pub enum Reverted16> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,  .9:  C9,
    .10: C10, .11: C11, .12: C12, .13: C13, .14: C14, .15: C15,
));
tuple_multicall!(impl<17> MultiCall<Reverted = pub enum Reverted17> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,  .9:  C9,
    .10: C10, .11: C11, .12: C12, .13: C13, .14: C14, .15: C15, .16: C16,
));
tuple_multicall!(impl<18> MultiCall<Reverted = pub enum Reverted18> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,  .9:  C9,
    .10: C10, .11: C11, .12: C12, .13: C13, .14: C14, .15: C15, .16: C16, .17: C17,
));
tuple_multicall!(impl<19> MultiCall<Reverted = pub enum Reverted19> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,  .9:  C9,
    .10: C10, .11: C11, .12: C12, .13: C13, .14: C14, .15: C15, .16: C16, .17: C17, .18: C18,
));
tuple_multicall!(impl<20> MultiCall<Reverted = pub enum Reverted20> for (
    .0:  C0,  .1:  C1,  .2:  C2,  .3:  C3,  .4:  C4,  .5:  C5,  .6:  C6,  .7:  C7,  .8:  C8,  .9:  C9,
    .10: C10, .11: C11, .12: C12, .13: C13, .14: C14, .15: C15, .16: C16, .17: C17, .18: C18, .19: C19,
));
