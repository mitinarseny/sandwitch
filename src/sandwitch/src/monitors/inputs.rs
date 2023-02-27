use ethers::types::Transaction;

pub(crate) trait FromWithTx<T>: Sized {
    fn from_with_tx(value: T, tx: &Transaction) -> Self;
}

pub(crate) trait IntoWithTx<T>: Sized {
    fn into_with_tx(self, tx: &Transaction) -> T;
}

impl<T, U> IntoWithTx<U> for T
where
    U: FromWithTx<T>,
{
    fn into_with_tx(self, tx: &Transaction) -> U {
        U::from_with_tx(self, tx)
    }
}
