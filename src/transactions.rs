use ethers::types::{
    transaction::{eip2718::TypedTransaction, eip2930::AccessList},
    Address, Bytes, NameOrAddress, OtherFields, Signature, TxHash, H256, U256,
};
use thiserror::Error as ThisError;

#[cfg(not(feature = "legacy"))]
pub type TransactionRequest = ethers::types::transaction::eip1559::Eip1559TransactionRequest;

#[cfg(feature = "legacy")]
pub type TransactionRequest = ethers::types::TransactionRequest;

/// Clone implentation of [`ethers::types::Transaction`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// The transaction's hash
    pub hash: H256,

    /// The transaction's nonce
    pub nonce: U256,

    /// Block hash. None when pending.
    pub block_hash: Option<H256>,

    /// Block number. None when pending.
    pub block_number: Option<u64>,

    /// Transaction Index. None when pending.
    pub transaction_index: Option<u64>,

    /// Sender
    pub from: Address,

    /// Recipient (None when contract creation)
    pub to: Option<Address>,

    /// Transferred value
    pub value: U256,

    /// Gas amount
    pub gas: U256,

    /// Input data
    pub input: Bytes,

    pub fees: Fees,

    /// ECDSA recovery id
    pub v: u64,

    /// ECDSA signature r
    pub r: U256,

    /// ECDSA signature s
    pub s: U256,

    // EIP2930
    pub access_list: Option<AccessList>,

    pub chain_id: Option<U256>,

    /// Captures unknown fields such as additional fields used by L2s
    pub other: OtherFields,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fees {
    Legacy {
        gas_price: U256,
    },
    EIP1559 {
        /// Represents the maximum tx fee that will go to the miner as part of the user's
        /// fee payment. It serves 3 purposes:
        /// 1. Compensates miners for the uncle/ommer risk + fixed costs of including transaction in a
        /// block; 2. Allows users with high opportunity costs to pay a premium to miners;
        /// 3. In times where demand exceeds the available block space (i.e. 100% full, 30mm gas),
        /// this component allows first price auctions (i.e. the pre-1559 fee model) to happen on the
        /// priority fee.
        ///
        /// More context [here](https://hackmd.io/@q8X_WM2nTfu6nuvAzqXiTQ/1559-wallets)
        max_priority_fee_per_gas: U256,

        /// Represents the maximum amount that a user is willing to pay for their tx (inclusive of
        /// baseFeePerGas and maxPriorityFeePerGas). The difference between maxFeePerGas and
        /// baseFeePerGas + maxPriorityFeePerGas is “refunded” to the user.
        max_fee_per_gas: U256,
    },
}

impl Fees {
    pub fn priority_fee(&self) -> U256 {
        match self {
            Fees::Legacy { gas_price } => *gas_price,
            Fees::EIP1559 {
                max_priority_fee_per_gas,
                ..
            } => *max_priority_fee_per_gas,
        }
    }
}

#[derive(ThisError, Debug)]
#[error("invalid transaction")]
pub struct InvalidTransaction;

impl TryFrom<ethers::types::Transaction> for Transaction {
    type Error = InvalidTransaction;

    fn try_from(tx: ethers::types::Transaction) -> Result<Self, Self::Error> {
        let ethers::types::Transaction {
            hash,
            nonce,
            block_hash,
            block_number,
            transaction_index,
            from,
            to,
            value,
            gas_price,
            gas,
            input,
            v,
            r,
            s,
            transaction_type,
            access_list,
            max_priority_fee_per_gas,
            max_fee_per_gas,
            chain_id,
            other,
        } = tx;
        Ok(Self {
            hash,
            nonce,
            block_hash,
            block_number: block_number.map(|n| n.as_u64()),
            transaction_index: transaction_index.map(|n| n.as_u64()),
            from,
            to,
            value,
            gas,
            input,
            fees: if transaction_type.is_some_and(|n| n == 2.into()) {
                let (max_priority_fee_per_gas, max_fee_per_gas) = max_priority_fee_per_gas
                    .zip(max_fee_per_gas)
                    .ok_or(InvalidTransaction)?;
                Fees::EIP1559 {
                    max_priority_fee_per_gas,
                    max_fee_per_gas,
                }
            } else {
                Fees::Legacy {
                    gas_price: gas_price.ok_or(InvalidTransaction)?,
                }
            },
            v: v.as_u64(),
            r,
            s,
            access_list,
            chain_id,
            other,
        })
    }
}

impl From<Transaction> for ethers::types::Transaction {
    fn from(tx: Transaction) -> Self {
        let Transaction {
            hash,
            nonce,
            block_hash,
            block_number,
            transaction_index,
            from,
            to,
            value,
            gas,
            input,
            fees: priority_fees,
            v,
            r,
            s,
            access_list,
            chain_id,
            other,
        } = tx;

        let mut tx = ethers::types::Transaction {
            hash,
            nonce,
            block_hash,
            block_number: block_number.map(Into::into),
            transaction_index: transaction_index.map(Into::into),
            from,
            to,
            value,
            gas,
            input,
            v: v.into(),
            r,
            s,
            access_list,
            chain_id,
            other,
            ..Default::default()
        };
        match priority_fees {
            Fees::Legacy { gas_price } => {
                tx.gas_price = Some(gas_price);
                tx.transaction_type = tx.access_list.is_some().then_some(1.into());
            }
            Fees::EIP1559 {
                max_priority_fee_per_gas,
                max_fee_per_gas,
            } => {
                tx.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
                tx.max_fee_per_gas = Some(max_fee_per_gas);
                tx.transaction_type = Some(2.into());
            }
        }
        tx
    }
}
