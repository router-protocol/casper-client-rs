use core::fmt::{self, Display, Formatter};
use std::error::Error as StdError;

#[cfg(doc)]
use super::{TransactionV1, TransactionV1Builder};

/// Errors returned while building a [`TransactionV1`] using a [`TransactionV1Builder`].
#[derive(Clone, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum TransactionV1BuilderError {
    /// Failed to build transaction due to missing initiator_addr.
    ///
    /// Call [`TransactionV1Builder::with_initiator_addr`] or
    /// [`TransactionV1Builder::with_secret_key`] before calling [`TransactionV1Builder::build`].
    MissingInitiatorAddr,
    /// Failed to build transaction due to missing chain name.
    ///
    /// Call [`TransactionV1Builder::with_chain_name`] before calling
    /// [`TransactionV1Builder::build`].
    MissingChainName,
    /// Failed to build transaction due to an error when calling `to_bytes` on one of the payload
    /// `field`.
    CouldNotSerializeField {
        /// The field index that failed to serialize.
        field_index: u16,
    },
}

impl Display for TransactionV1BuilderError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            TransactionV1BuilderError::MissingInitiatorAddr => {
                write!(
                    formatter,
                    "transaction requires account - use `with_account` or `with_secret_key`"
                )
            }
            TransactionV1BuilderError::MissingChainName => {
                write!(
                    formatter,
                    "transaction requires chain name - use `with_chain_name`"
                )
            }
            TransactionV1BuilderError::CouldNotSerializeField { field_index } => {
                write!(formatter, "Cannot serialize field at index {}", field_index)
            }
        }
    }
}

impl StdError for TransactionV1BuilderError {}
