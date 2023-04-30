use core::array::TryFromSliceError;

use bytes::{Buf, BufMut};
use ethers::types::U256;

pub trait TryFromSlice<const N: usize>: From<[u8; N]> {
    #[inline]
    fn len_bytes() -> usize {
        N
    }

    fn try_from_slice(slice: impl AsRef<[u8]>) -> Result<Self, TryFromSliceError> {
        Ok(<[u8; N]>::try_from(slice.as_ref())?.into())
    }

    fn try_read_from<B: Buf>(buf: &mut B) -> Result<Self, TryFromSliceError> {
        let this = Self::try_from_slice(buf.chunk())?;
        buf.advance(N);
        Ok(this)
    }
}

impl<T, const N: usize> TryFromSlice<N> for T where T: From<[u8; N]> {}

pub trait ToFixedBytes<const N: usize>: TryFromSlice<N> {
    fn to_fixed_bytes_in(&self, bytes: &mut [u8; N]);

    fn to_fixed_bytes(&self) -> [u8; N] {
        let mut b = [0; N];
        self.to_fixed_bytes_in(&mut b);
        b
    }
}

impl ToFixedBytes<32> for U256 {
    fn to_fixed_bytes_in(&self, bytes: &mut [u8; 32]) {
        self.to_big_endian(bytes)
    }
}

pub trait ChainBufMut: BufMut {
    fn write<B: Buf>(mut self, src: B) -> Self
    where
        Self: Sized,
    {
        self.put(src);
        self
    }

    fn write_big<const N: usize>(self, v: impl ToFixedBytes<N>) -> Self
    where
        Self: Sized,
    {
        self.write(v.to_fixed_bytes().as_slice())
    }
}

impl<B> ChainBufMut for B where B: BufMut {}
