pub mod error;
use super::arg_handling;
use crate::{
    cli::{FieldsContainer, FieldsContainerError},
    types::InitiatorAddrAndSecretKey,
};
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use casper_types::{
    bytesrepr::{Bytes, ToBytes},
    system::auction::{DelegatorKind, Reservation},
    AddressableEntityHash, CLValueError, Digest, EntityVersion, InitiatorAddr, PackageHash,
    PricingMode, PublicKey, RuntimeArgs, SecretKey, TimeDiff, Timestamp, TransactionArgs,
    TransactionEntryPoint, TransactionInvocationTarget, TransactionRuntimeParams,
    TransactionScheduling, TransactionTarget, TransactionV1, TransactionV1Payload, TransferTarget,
    URef, U512,
};
use core::marker::PhantomData;
pub use error::TransactionV1BuilderError;

/// A builder for constructing `TransactionV1` instances with various configuration options.
///
/// The `TransactionV1Builder` provides a flexible API for specifying different transaction
/// parameters like the target, scheduling, entry point, and signing options. Once all the required
/// fields are set, the transaction can be built by calling [`build`](Self::build).
///
/// # Fields
///
/// - `args`: Arguments passed to the transaction's runtime, initialized to
///   [`RuntimeArgs::new`](RuntimeArgs::new).
/// - `target`: Specifies the target of the transaction, which can be native or other custom
///   targets. Defaults to [`TransactionTarget::Native`](TransactionTarget::Native).
/// - `scheduling`: Determines the scheduling mechanism of the transaction, e.g., standard or
///   immediate, and is initialized to
///   [`TransactionScheduling::Standard`](TransactionScheduling::Standard).
/// - `entry_point`: Defines the transaction's entry point, such as transfer or another defined
///   action. Defaults to [`TransactionEntryPoint::Transfer`](TransactionEntryPoint::Transfer).
/// - `chain_name`: The name of the blockchain where the transaction will be executed. Initially set
///   to `None` and must be provided before building the transaction.
///
/// ## Time-Related Fields
/// - `timestamp`: The timestamp at which the transaction is created. It is either set to the
///   current time using [`Timestamp::now`](Timestamp::now) or [`Timestamp::zero`](Timestamp::zero)
///   without the `std-fs-io` feature.
/// - `ttl`: Time-to-live for the transaction, specified as a [`TimeDiff`], representing how long
///   the transaction is valid for execution. Defaults to [`Self::DEFAULT_TTL`].
///
/// ## Pricing and Initiator Fields
/// - `pricing_mode`: Specifies the pricing mode to use for transaction execution (e.g., fixed or
///   dynamic). Defaults to [`Self::DEFAULT_PRICING_MODE`].
/// - `initiator_addr`: The address of the initiator who creates and signs the transaction.
///   Initially set to `None` and must be set before building.
///
/// ## Signing Fields
/// - `secret_key`: The secret key used to sign the transaction. This field is conditional based on
/// the compilation environment:
///     - In normal mode, it holds a reference to the secret key (`Option<&'a SecretKey>`).
///     - In testing mode or with the `std` feature enabled, it holds an owned secret key
///  (`Option<SecretKey>`).
///
/// ## Phantom Data
/// - `_phantom_data`: Ensures the correct lifetime `'a` is respected for the builder, helping with
///   proper borrowing and memory safety.
#[derive(Debug)]
pub struct TransactionV1Builder<'a> {
    /// Arguments passed to the transaction's runtime.
    args: TransactionArgs,
    /// The target of the transaction (e.g., native).
    target: TransactionTarget,
    /// Defines how the transaction is scheduled (e.g., standard, immediate).
    scheduling: TransactionScheduling,
    /// Specifies the entry point of the transaction (e.g., transfer).
    entry_point: TransactionEntryPoint,
    /// The name of the blockchain where the transaction will be executed.
    chain_name: Option<String>,
    /// The timestamp of the transaction.
    timestamp: Timestamp,
    /// The time-to-live for the transaction, representing how long it's valid for execution.
    ttl: TimeDiff,
    /// The pricing mode used for the transaction's execution cost.
    pricing_mode: PricingMode,
    /// The address of the transaction initiator.
    initiator_addr: Option<InitiatorAddr>,
    /// The secret key used for signing the transaction (in normal mode).
    #[cfg(not(test))]
    secret_key: Option<&'a SecretKey>,
    /// The secret key used for signing the transaction (in testing).
    #[cfg(test)]
    secret_key: Option<SecretKey>,
    /// Phantom data to ensure the correct lifetime for references.
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a> TransactionV1Builder<'a> {
    /// The default time-to-live for transactions, i.e. 30 minutes.
    pub const DEFAULT_TTL: TimeDiff = TimeDiff::from_millis(30 * 60 * 1_000);
    /// The default pricing mode for v1 transactions, ie FIXED cost.
    pub const DEFAULT_PRICING_MODE: PricingMode = PricingMode::Fixed {
        gas_price_tolerance: 5,
        additional_computation_factor: 0,
    };
    /// The default scheduling for transactions, i.e. `Standard`.
    pub const DEFAULT_SCHEDULING: TransactionScheduling = TransactionScheduling::Standard;

    /// Creates a new `TransactionV1Builder` instance with default settings.
    ///
    /// # Important
    ///
    /// Before calling [`build`](Self::build), you must ensure that either:
    /// - A chain name is provided by calling [`with_chain_name`](Self::with_chain_name),
    /// - An initiator address is set by calling [`with_initiator_addr`](Self::with_initiator_addr),
    /// - or a secret key is set by calling [`with_secret_key`](Self::with_secret_key).
    ///
    /// # Default Values
    /// This function sets the following default values upon creation:
    ///
    /// - `chain_name`: Initialized to `None`.
    /// - `timestamp`: Set to the current time using [`Timestamp::now`](Timestamp::now), or
    ///   [`Timestamp::zero`](Timestamp::zero) if the `std-fs-io` feature is disabled.
    /// - `ttl`: Defaults to [`Self::DEFAULT_TTL`].
    /// - `pricing_mode`: Defaults to [`Self::DEFAULT_PRICING_MODE`].
    /// - `initiator_addr`: Initialized to `None`.
    /// - `secret_key`: Initialized to `None`.
    ///
    /// Additionally, the following internal fields are configured:
    ///
    /// - `args`: Initialized to an empty [`RuntimeArgs::new`](RuntimeArgs::new).
    /// - `entry_point`: Set to
    ///   [`TransactionEntryPoint::Transfer`](TransactionEntryPoint::Transfer).
    /// - `target`: Defaults to [`TransactionTarget::Native`](TransactionTarget::Native).
    /// - `scheduling`: Defaults to
    ///   [`TransactionScheduling::Standard`](TransactionScheduling::Standard).
    ///
    /// # Testing and Additional Configuration
    ///
    /// # Returns
    ///
    /// A new `TransactionV1Builder` instance.
    pub(crate) fn new() -> Self {
        #[cfg(any(feature = "std-fs-io", test))]
        let timestamp = Timestamp::now();
        #[cfg(not(any(feature = "std-fs-io", test)))]
        let timestamp = Timestamp::zero();

        TransactionV1Builder {
            args: TransactionArgs::Named(RuntimeArgs::new()),
            entry_point: TransactionEntryPoint::Transfer,
            target: TransactionTarget::Native,
            scheduling: TransactionScheduling::Standard,
            chain_name: None,
            timestamp,
            ttl: Self::DEFAULT_TTL,
            pricing_mode: Self::DEFAULT_PRICING_MODE,
            initiator_addr: None,
            secret_key: None,
            _phantom_data: PhantomData,
        }
    }

    /// Returns a new `TransactionV1Builder` suitable for building a native transfer transaction.
    pub fn new_transfer<A: Into<U512>, T: Into<TransferTarget>>(
        amount: A,
        maybe_source: Option<URef>,
        target: T,
        maybe_id: Option<u64>,
    ) -> Result<Self, CLValueError> {
        let args = arg_handling::new_transfer_args(amount, maybe_source, target, maybe_id)?;
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(args);
        builder.target = TransactionTarget::Native;
        builder.entry_point = TransactionEntryPoint::Transfer;
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        Ok(builder)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a native add_bid transaction.
    pub fn new_add_bid<A: Into<U512>>(
        public_key: PublicKey,
        delegation_rate: u8,
        amount: A,
        minimum_delegation_amount: Option<u64>,
        maximum_delegation_amount: Option<u64>,
        reserved_slots: Option<u32>,
    ) -> Result<Self, CLValueError> {
        let args = arg_handling::new_add_bid_args(
            public_key,
            delegation_rate,
            amount,
            minimum_delegation_amount,
            maximum_delegation_amount,
            reserved_slots,
        )?;
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(args);
        builder.target = TransactionTarget::Native;
        builder.entry_point = TransactionEntryPoint::AddBid;
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        Ok(builder)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a native withdraw_bid
    /// transaction.
    pub fn new_withdraw_bid<A: Into<U512>>(
        public_key: PublicKey,
        amount: A,
    ) -> Result<Self, CLValueError> {
        let args = arg_handling::new_withdraw_bid_args(public_key, amount)?;
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(args);
        builder.target = TransactionTarget::Native;
        builder.entry_point = TransactionEntryPoint::WithdrawBid;
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        Ok(builder)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a native delegate transaction.
    pub fn new_delegate<A: Into<U512>>(
        delegator: PublicKey,
        validator: PublicKey,
        amount: A,
    ) -> Result<Self, CLValueError> {
        let args = arg_handling::new_delegate_args(delegator, validator, amount)?;
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(args);
        builder.target = TransactionTarget::Native;
        builder.entry_point = TransactionEntryPoint::Delegate;
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        Ok(builder)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a native undelegate transaction.
    pub fn new_undelegate<A: Into<U512>>(
        delegator: PublicKey,
        validator: PublicKey,
        amount: A,
    ) -> Result<Self, CLValueError> {
        let args = arg_handling::new_undelegate_args(delegator, validator, amount)?;
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(args);
        builder.target = TransactionTarget::Native;
        builder.entry_point = TransactionEntryPoint::Undelegate;
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        Ok(builder)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a native redelegate transaction.
    pub fn new_redelegate<A: Into<U512>>(
        delegator: PublicKey,
        validator: PublicKey,
        amount: A,
        new_validator: PublicKey,
    ) -> Result<Self, CLValueError> {
        let args = arg_handling::new_redelegate_args(delegator, validator, amount, new_validator)?;
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(args);
        builder.target = TransactionTarget::Native;
        builder.entry_point = TransactionEntryPoint::Redelegate;
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        Ok(builder)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a native activate_bid
    /// transaction.
    pub fn new_activate_bid(validator: PublicKey) -> Result<Self, CLValueError> {
        let args = arg_handling::new_activate_bid_args(validator)?;
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(args);
        builder.target = TransactionTarget::Native;
        builder.entry_point = TransactionEntryPoint::ActivateBid;
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        Ok(builder)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a native change_bid_public_key
    /// transaction.
    pub fn new_change_bid_public_key(
        public_key: PublicKey,
        new_public_key: PublicKey,
    ) -> Result<Self, CLValueError> {
        let args = arg_handling::new_change_bid_public_key_args(public_key, new_public_key)?;
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(args);
        builder.target = TransactionTarget::Native;
        builder.entry_point = TransactionEntryPoint::ChangeBidPublicKey;
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        Ok(builder)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a native add_reservations
    /// transaction.
    pub fn new_add_reservations(reservations: Vec<Reservation>) -> Result<Self, CLValueError> {
        let args = arg_handling::new_add_reservations_args(reservations)?;
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(args);
        builder.target = TransactionTarget::Native;
        builder.entry_point = TransactionEntryPoint::AddReservations;
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        Ok(builder)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a native cancel_reservations
    /// transaction.
    pub fn new_cancel_reservations(
        validator: PublicKey,
        delegators: Vec<DelegatorKind>,
    ) -> Result<Self, CLValueError> {
        let args = arg_handling::new_cancel_reservations_args(validator, delegators)?;
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(args);
        builder.target = TransactionTarget::Native;
        builder.entry_point = TransactionEntryPoint::CancelReservations;
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        Ok(builder)
    }

    fn new_targeting_stored<E: Into<String>>(
        id: TransactionInvocationTarget,
        entry_point: E,
        runtime: TransactionRuntimeParams,
    ) -> Self {
        let target = TransactionTarget::Stored { id, runtime };
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(RuntimeArgs::new());
        builder.target = target;
        builder.entry_point = TransactionEntryPoint::Custom(entry_point.into());
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        builder
    }

    /// Returns a new `TransactionV1Builder` suitable for building a transaction targeting a stored
    /// entity.
    pub fn new_targeting_invocable_entity<E: Into<String>>(
        hash: AddressableEntityHash,
        entry_point: E,
        runtime: TransactionRuntimeParams,
    ) -> Self {
        let id = TransactionInvocationTarget::new_invocable_entity(hash);
        Self::new_targeting_stored(id, entry_point, runtime)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a transaction targeting a stored
    /// entity via its alias.
    pub fn new_targeting_invocable_entity_via_alias<A: Into<String>, E: Into<String>>(
        alias: A,
        entry_point: E,
        runtime: TransactionRuntimeParams,
    ) -> Self {
        let id = TransactionInvocationTarget::new_invocable_entity_alias(alias.into());
        Self::new_targeting_stored(id, entry_point, runtime)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a transaction targeting a
    /// package.
    pub fn new_targeting_package<E: Into<String>>(
        hash: PackageHash,
        version: Option<EntityVersion>,
        entry_point: E,
        runtime: TransactionRuntimeParams,
    ) -> Self {
        let id = TransactionInvocationTarget::new_package(hash, version);
        Self::new_targeting_stored(id, entry_point, runtime)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a transaction targeting a
    /// package via its alias.
    pub fn new_targeting_package_via_alias<A: Into<String>, E: Into<String>>(
        alias: A,
        version: Option<EntityVersion>,
        entry_point: E,
        runtime: TransactionRuntimeParams,
    ) -> Self {
        let id = TransactionInvocationTarget::new_package_alias(alias.into(), version);
        Self::new_targeting_stored(id, entry_point, runtime)
    }

    /// Returns a new `TransactionV1Builder` suitable for building a transaction for running session
    /// logic, i.e. compiled Wasm.
    pub fn new_session(
        is_install_upgrade: bool,
        module_bytes: Bytes,
        runtime: TransactionRuntimeParams,
    ) -> Self {
        let target = TransactionTarget::Session {
            is_install_upgrade,
            module_bytes,
            runtime,
        };
        let mut builder = TransactionV1Builder::new();
        builder.args = TransactionArgs::Named(RuntimeArgs::new());
        builder.target = target;
        builder.entry_point = TransactionEntryPoint::Call;
        builder.scheduling = Self::DEFAULT_SCHEDULING;
        builder
    }

    /// Sets the `chain_name` in the transaction.
    ///
    /// Must be provided or building will fail.
    pub fn with_chain_name<C: Into<String>>(mut self, chain_name: C) -> Self {
        self.chain_name = Some(chain_name.into());
        self
    }

    /// Sets the `timestamp` in the transaction.
    ///
    /// If not provided, the timestamp will be set to the time when the builder was constructed.
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the `ttl` (time-to-live) in the transaction.
    ///
    /// If not provided, the ttl will be set to [`Self::DEFAULT_TTL`].
    pub fn with_ttl(mut self, ttl: TimeDiff) -> Self {
        self.ttl = ttl;
        self
    }

    /// Sets the `pricing_mode` in the transaction.
    ///
    /// If not provided, the pricing mode will be set to [`Self::DEFAULT_PRICING_MODE`].
    pub fn with_pricing_mode(mut self, pricing_mode: PricingMode) -> Self {
        self.pricing_mode = pricing_mode;
        self
    }

    /// Sets the `initiator_addr` in the transaction.
    ///
    /// If not provided, the public key derived from the secret key used in the builder will be
    /// used as the `InitiatorAddr::PublicKey` in the transaction.
    pub fn with_initiator_addr<I: Into<InitiatorAddr>>(mut self, initiator_addr: I) -> Self {
        self.initiator_addr = Some(initiator_addr.into());
        self
    }

    /// Sets the secret key used to sign the transaction on calling [`build`](Self::build).
    ///
    /// If not provided, the transaction can still be built, but will be unsigned and will be
    /// invalid until subsequently signed.
    pub fn with_secret_key(mut self, secret_key: &'a SecretKey) -> Self {
        #[cfg(not(test))]
        {
            self.secret_key = Some(secret_key);
        }
        #[cfg(test)]
        {
            self.secret_key = Some(
                SecretKey::from_der(secret_key.to_der().expect("should der-encode"))
                    .expect("should der-decode"),
            );
        }
        self
    }

    /// Sets the runtime args in the transaction.
    ///
    /// NOTE: this overwrites any existing runtime args.  To append to existing args, use
    /// [`TransactionV1Builder::with_runtime_arg`].
    pub fn with_runtime_args(mut self, args: RuntimeArgs) -> Self {
        self.args = TransactionArgs::Named(args);
        self
    }

    /// Sets the runtime args in the transaction.
    pub fn with_chunked_args(mut self, args: Bytes) -> Self {
        self.args = TransactionArgs::Bytesrepr(args);
        self
    }

    /// Sets the entry point for the transaction.
    pub fn with_entry_point(mut self, entry_point: TransactionEntryPoint) -> Self {
        self.entry_point = entry_point;
        self
    }

    /// Returns the new transaction, or an error if non-defaulted fields were not set.
    ///
    /// For more info, see [the `TransactionBuilder` documentation](TransactionV1Builder).
    pub fn build(self) -> Result<TransactionV1, TransactionV1BuilderError> {
        self.do_build()
    }

    fn build_transaction_inner(
        chain_name: String,
        timestamp: Timestamp,
        ttl: TimeDiff,
        pricing_mode: PricingMode,
        fields: BTreeMap<u16, Bytes>,
        initiator_addr_and_secret_key: InitiatorAddrAndSecretKey,
    ) -> TransactionV1 {
        let initiator_addr = initiator_addr_and_secret_key.initiator_addr();
        let transaction_v1_payload = TransactionV1Payload::new(
            chain_name,
            timestamp,
            ttl,
            pricing_mode,
            initiator_addr,
            fields,
        );
        let hash = Digest::hash(
            transaction_v1_payload
                .to_bytes()
                .unwrap_or_else(|error| panic!("should serialize body: {}", error)),
        );
        let mut transaction =
            TransactionV1::new(hash.into(), transaction_v1_payload, BTreeSet::new());

        if let Some(secret_key) = initiator_addr_and_secret_key.secret_key() {
            transaction.sign(secret_key);
        }
        transaction
    }

    fn do_build(self) -> Result<TransactionV1, TransactionV1BuilderError> {
        let initiator_addr_and_secret_key = match (self.initiator_addr, &self.secret_key) {
            (Some(initiator_addr), Some(secret_key)) => InitiatorAddrAndSecretKey::Both {
                initiator_addr,
                secret_key,
            },
            (Some(initiator_addr), None) => {
                InitiatorAddrAndSecretKey::InitiatorAddr(initiator_addr)
            }
            (None, Some(secret_key)) => InitiatorAddrAndSecretKey::SecretKey(secret_key),
            (None, None) => return Err(TransactionV1BuilderError::MissingInitiatorAddr),
        };

        let chain_name = self
            .chain_name
            .ok_or(TransactionV1BuilderError::MissingChainName)?;

        let container =
            FieldsContainer::new(self.args, self.target, self.entry_point, self.scheduling)
                .to_map()
                .map_err(|err| match err {
                    FieldsContainerError::CouldNotSerializeField { field_index } => {
                        TransactionV1BuilderError::CouldNotSerializeField { field_index }
                    }
                })?;

        let transaction = Self::build_transaction_inner(
            chain_name,
            self.timestamp,
            self.ttl,
            self.pricing_mode,
            container,
            initiator_addr_and_secret_key,
        );

        Ok(transaction)
    }
}
