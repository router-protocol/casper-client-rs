use casper_types::bytesrepr::Bytes;

/// Container for session-related arguments used while constructing a `Deploy`.
///
/// ## `session_args_simple`
///
/// For methods taking `session_args_simple`, this parameter is the session contract arguments, in
/// the form `<NAME:TYPE='VALUE'>` or `<NAME:TYPE=null>`.
///
/// There are further details in
/// [the docs for the equivalent
/// `payment_args_simple`](struct.PaymentStrParams.html#payment_args_simple).
///
/// ## `session_args_json`
///
/// For methods taking `session_args_json`, this parameter is the session contract arguments, as a
/// JSON-encoded Array of JSON Objects of the form:
/// ```json
/// [{"name":<String>,"type":<VALUE>,"value":<VALUE>}]
/// ```
///
/// There are further details in
/// [the docs for the equivalent `payment_args_json`](struct.PaymentStrParams.html#payment_args_json).
///
/// ---
///
/// **Note** while multiple session args can be specified for a single session code instance, only
/// one of `session_args_simple`, or `session_args_json` may be used.
#[derive(Default, Debug)]
pub struct SessionStrParams<'a> {
    pub(super) session_hash: &'a str,
    pub(super) session_name: &'a str,
    pub(super) session_package_hash: &'a str,
    pub(super) session_package_name: &'a str,
    pub(super) session_path: &'a str,
    pub(super) session_bytes: Bytes,
    pub(super) session_args_simple: Vec<&'a str>,
    pub(super) session_args_json: &'a str,
    pub(super) session_version: &'a str,
    pub(super) session_entry_point: &'a str,
    pub(super) is_session_transfer: bool,
    pub(super) session_chunked_args: Option<&'a [u8]>,
}

impl<'a> SessionStrParams<'a> {
    /// Constructs a `SessionStrParams` using a session smart contract file.
    ///
    /// * `session_path` is the path to the compiled Wasm session code.
    /// * See the struct docs for a description of [`session_args_simple`](#session_args_simple),
    ///   [`session_args_json`](#session_args_json)
    pub fn with_path(
        session_path: &'a str,
        session_args_simple: Vec<&'a str>,
        session_args_json: &'a str,
    ) -> Self {
        Self {
            session_path,
            session_args_simple,
            session_args_json,
            ..Default::default()
        }
    }

    /// Constructs a `SessionStrParams` using session bytes.
    ///
    /// * `session_bytes` are the bytes of the compiled Wasm session code.
    /// * See the struct docs for a description of [`session_args_simple`](#session_args_simple),
    ///   [`session_args_json`](#session_args_json)
    pub fn with_bytes(
        session_bytes: Bytes,
        session_args_simple: Vec<&'a str>,
        session_args_json: &'a str,
    ) -> Self {
        Self {
            session_bytes,
            session_args_simple,
            session_args_json,
            ..Default::default()
        }
    }

    /// Constructs a `SessionStrParams` using a stored contract's name.
    ///
    /// * `session_name` is the name of the stored contract (associated with the executing account)
    ///   to be called as the session.
    /// * `session_entry_point` is the name of the method that will be used when calling the session
    ///   contract.
    /// * See the struct docs for a description of [`session_args_simple`](#session_args_simple),
    ///   [`session_args_json`](#session_args_json)
    pub fn with_name(
        session_name: &'a str,
        session_entry_point: &'a str,
        session_args_simple: Vec<&'a str>,
        session_args_json: &'a str,
    ) -> Self {
        Self {
            session_name,
            session_args_simple,
            session_args_json,
            session_entry_point,
            ..Default::default()
        }
    }

    /// Constructs a `SessionStrParams` using a stored contract's hex-encoded hash.
    ///
    /// * `session_hash` is the hex-encoded hash of the stored contract to be called as the session.
    /// * `session_entry_point` is the name of the method that will be used when calling the session
    ///   contract.
    /// * See the struct docs for a description of [`session_args_simple`](#session_args_simple),
    ///   [`session_args_json`](#session_args_json)
    pub fn with_hash(
        session_hash: &'a str,
        session_entry_point: &'a str,
        session_args_simple: Vec<&'a str>,
        session_args_json: &'a str,
    ) -> Self {
        Self {
            session_hash,
            session_args_simple,
            session_args_json,
            session_entry_point,
            ..Default::default()
        }
    }

    /// Constructs a `SessionStrParams` using a stored contract's package name.
    ///
    /// * `session_package_name` is the name of the stored package to be called as the session.
    /// * `session_version` is the version of the called session contract. The latest will be used
    ///   if `session_version` is empty.
    /// * `session_entry_point` is the name of the method that will be used when calling the session
    ///   contract.
    /// * See the struct docs for a description of [`session_args_simple`](#session_args_simple),
    ///   [`session_args_json`](#session_args_json)
    pub fn with_package_name(
        session_package_name: &'a str,
        session_version: &'a str,
        session_entry_point: &'a str,
        session_args_simple: Vec<&'a str>,
        session_args_json: &'a str,
    ) -> Self {
        Self {
            session_package_name,
            session_args_simple,
            session_args_json,
            session_version,
            session_entry_point,
            ..Default::default()
        }
    }

    /// Constructs a `SessionStrParams` using a stored contract's package hash.
    ///
    /// * `session_package_hash` is the hex-encoded hash of the stored package to be called as the
    ///   session.
    /// * `session_version` is the version of the called session contract. The latest will be used
    ///   if `session_version` is empty.
    /// * `session_entry_point` is the name of the method that will be used when calling the session
    ///   contract.
    /// * See the struct docs for a description of [`session_args_simple`](#session_args_simple),
    ///   [`session_args_json`](#session_args_json)
    pub fn with_package_hash(
        session_package_hash: &'a str,
        session_version: &'a str,
        session_entry_point: &'a str,
        session_args_simple: Vec<&'a str>,
        session_args_json: &'a str,
    ) -> Self {
        Self {
            session_package_hash,
            session_args_simple,
            session_args_json,
            session_version,
            session_entry_point,
            ..Default::default()
        }
    }

    /// Constructs a `SessionStrParams` representing a `Transfer` type of `Deploy`.
    ///
    /// * See the struct docs for a description of [`session_args_simple`](#session_args_simple),
    ///   [`session_args_json`](#session_args_json)
    pub fn with_transfer(session_args_simple: Vec<&'a str>, session_args_json: &'a str) -> Self {
        Self {
            is_session_transfer: true,
            session_args_simple,
            session_args_json,
            ..Default::default()
        }
    }
}
