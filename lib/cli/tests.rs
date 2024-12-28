use casper_types::{AsymmetricType, CLValue, EntityAddr, PublicKey, SecretKey, U512};
#[cfg(feature = "std-fs-io")]
use casper_types::{DeployExcessiveSizeError, ExecutableDeployItem};

use crate::cli::transaction::create_transaction;
#[cfg(feature = "std-fs-io")]
use crate::MAX_SERIALIZED_SIZE_OF_DEPLOY;
use crate::{Error, OutputKind};

use super::*;

const SAMPLE_ACCOUNT: &str = "01722e1b3d31bef0ba832121bd2941aae6a246d0d05ac95aa16dd587cc5469871d";
const PKG_HASH: &str = "09dcee4b212cfd53642ab323fbef07dafafc6f945a80a00147f62910a915c4e6";
const ENTRYPOINT: &str = "entrypoint";
const VERSION: &str = "2";
const SAMPLE_DEPLOY: &str = r#"{
  "hash": "1053f767f1734e3b5b31253ea680778ac53f134f7c24518bf2c4cbb204852617",
  "header": {
    "account": "01f60bce2bb1059c41910eac1e7ee6c3ef4c8fcc63a901eb9603c1524cadfb0c18",
    "timestamp": "2022-12-11T18:37:06.901Z",
    "ttl": "10s",
    "gas_price": 1,
    "body_hash": "0a80edb81389ead7fb3d6a783355d821313c8baa68718fa7478aa0ca6a6b3b59",
    "dependencies": [],
    "chain_name": "casper-test-chain-name-1"
  },
  "payment": {
    "StoredVersionedContractByHash": {
      "hash": "09dcee4b212cfd53642ab323fbef07dafafc6f945a80a00147f62910a915c4e6",
      "version": 2,
      "entry_point": "entrypoint",
      "args": [
        [
          "name_01",
          {
            "cl_type": "Bool",
            "bytes": "00",
            "parsed": false
          }
        ],
	[
          "name_02",
          {
            "cl_type": "I32",
            "bytes": "2a000000",
            "parsed": 42
          }
        ]
      ]
    }
  },
  "session": {
    "StoredVersionedContractByHash": {
      "hash": "09dcee4b212cfd53642ab323fbef07dafafc6f945a80a00147f62910a915c4e6",
      "version": 2,
      "entry_point": "entrypoint",
      "args": [
        [
          "name_01",
          {
            "cl_type": "Bool",
            "bytes": "00",
            "parsed": false
          }
        ],
        [
          "name_02",
          {
            "cl_type": "I32",
            "bytes": "2a000000",
            "parsed": 42
          }
        ]
      ]
 }
  },
  "approvals": [
    {
      "signer": "01f60bce2bb1059c41910eac1e7ee6c3ef4c8fcc63a901eb9603c1524cadfb0c18",
      "signature": "01d701c27d7dc36b48fa457e4c7cc9999b444d7efb4a118c805b82d1f1af337437d00f9a9562694a7dd707abc01fa0158428a365a970853327d70d6d8f15aeea00"
    },
    {
      "signer": "016e3725ffd940bddb56e692e6309c6c82d2def515421219ddfd1ea0952e52491a",
      "signature": "010a973a45b72208b18da27b25ea62c6be31cd1b53b723b74cdd7e9f356d83df821b6431c973e2f6e24d10fdb213dc5e02d552ba113254e610992b6942ff76390e"
    }
  ]
}"#;

#[cfg(test)]
pub(crate) const ARGS_MAP_KEY: u16 = 0;
#[cfg(test)]
pub(crate) const TARGET_MAP_KEY: u16 = 1;
#[cfg(test)]
pub(crate) const ENTRY_POINT_MAP_KEY: u16 = 2;

pub fn deploy_params_without_account() -> DeployStrParams<'static> {
    DeployStrParams {
        secret_key: "",
        ttl: "10s",
        chain_name: "casper-test-chain-name-1",
        ..Default::default()
    }
}

pub fn deploy_params_without_secret_key() -> DeployStrParams<'static> {
    DeployStrParams {
        ttl: "10s",
        chain_name: "casper-test-chain-name-1",
        session_account: SAMPLE_ACCOUNT,
        ..Default::default()
    }
}

pub fn deploy_params() -> DeployStrParams<'static> {
    DeployStrParams {
        secret_key: "resources/test.pem",
        ttl: "10s",
        chain_name: "casper-test-chain-name-1",
        ..Default::default()
    }
}

fn args_simple() -> Vec<&'static str> {
    vec!["name_01:bool='false'", "name_02:i32='42'"]
}

#[cfg(feature = "std-fs-io")]
#[test]
fn should_create_deploy() {
    let deploy_params = deploy_params();
    let payment_params =
        PaymentStrParams::with_package_hash(PKG_HASH, VERSION, ENTRYPOINT, args_simple(), "");
    let session_params =
        SessionStrParams::with_package_hash(PKG_HASH, VERSION, ENTRYPOINT, args_simple(), "");

    let mut output = Vec::new();

    let deploy =
        deploy::with_payment_and_session(deploy_params, payment_params, session_params, false)
            .unwrap();
    crate::write_deploy(&deploy, &mut output).unwrap();

    // The test output can be used to generate data for SAMPLE_DEPLOY:
    // let secret_key = SecretKey::generate_ed25519().unwrap();
    // deploy.sign(&secret_key);
    // println!("{}", serde_json::to_string_pretty(&deploy).unwrap());

    let result = String::from_utf8(output).unwrap();

    let expected = crate::read_deploy(SAMPLE_DEPLOY.as_bytes()).unwrap();
    let actual = crate::read_deploy(result.as_bytes()).unwrap();

    assert_eq!(expected.header().account(), actual.header().account());
    assert_eq!(expected.header().ttl(), actual.header().ttl());
    assert_eq!(expected.header().gas_price(), actual.header().gas_price());
    assert_eq!(expected.header().body_hash(), actual.header().body_hash());
    assert_eq!(expected.payment(), actual.payment());
    assert_eq!(expected.session(), actual.session());
}

#[cfg(feature = "std-fs-io")]
#[test]
fn should_fail_to_create_large_deploy() {
    let deploy_params = deploy_params();
    let payment_params =
        PaymentStrParams::with_package_hash(PKG_HASH, VERSION, ENTRYPOINT, args_simple(), "");
    // Create a string arg of 1048576 letter 'a's to ensure the deploy is greater than 1048576
    // bytes.
    let large_args_simple = format!("name_01:string='{:a<1048576}'", "");

    let session_params = SessionStrParams::with_package_hash(
        PKG_HASH,
        VERSION,
        ENTRYPOINT,
        vec![large_args_simple.as_str()],
        "",
    );

    match deploy::with_payment_and_session(deploy_params, payment_params, session_params, false) {
        Err(CliError::Core(Error::DeploySize(DeployExcessiveSizeError {
            max_transaction_size,
            actual_deploy_size,
        }))) => {
            assert_eq!(max_transaction_size, MAX_SERIALIZED_SIZE_OF_DEPLOY);
            assert!(actual_deploy_size > MAX_SERIALIZED_SIZE_OF_DEPLOY as usize);
        }
        Err(error) => panic!("unexpected error: {}", error),
        Ok(_) => panic!("failed to error while creating an excessively large deploy"),
    }
}

#[test]
fn should_read_deploy() {
    let bytes = SAMPLE_DEPLOY.as_bytes();
    assert!(crate::read_deploy(bytes).is_ok());
}

#[test]
fn should_sign_deploy() {
    let bytes = SAMPLE_DEPLOY.as_bytes();
    let deploy = crate::read_deploy(bytes).unwrap();
    assert_eq!(
        deploy.approvals().len(),
        2,
        "Sample deploy should have 2 approvals."
    );

    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("deploy.json");

    crate::output_deploy(OutputKind::file(&path, false), &deploy).unwrap();

    let secret_key = SecretKey::generate_ed25519().unwrap();
    crate::sign_deploy_file(&path, &secret_key, OutputKind::file(&path, true)).unwrap();
    let signed_deploy = crate::read_deploy_file(&path).unwrap();

    assert_eq!(
        signed_deploy.approvals().len(),
        deploy.approvals().len() + 1,
    );
}

#[cfg(feature = "std-fs-io")]
#[test]
fn should_create_transfer() {
    use casper_types::{AsymmetricType, PublicKey};

    // with public key.
    let secret_key = SecretKey::generate_ed25519().unwrap();
    let public_key = PublicKey::from(&secret_key).to_hex();
    let transfer_deploy = deploy::new_transfer(
        "10000",
        None,
        &public_key,
        "1",
        deploy_params(),
        PaymentStrParams::with_amount("100"),
        false,
    );

    assert!(transfer_deploy.is_ok());
    assert!(matches!(
        transfer_deploy.unwrap().session(),
        ExecutableDeployItem::Transfer { .. }
    ));

    // with account hash
    let account_hash =
        "account-hash-0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
    let transfer_deploy = deploy::new_transfer(
        "10000",
        None,
        account_hash,
        "1",
        deploy_params(),
        PaymentStrParams::with_amount("100"),
        false,
    );

    assert!(transfer_deploy.is_ok());
    assert!(matches!(
        transfer_deploy.unwrap().session(),
        ExecutableDeployItem::Transfer { .. }
    ));

    // with uref.
    let uref = "uref-0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20-007";
    let transfer_deploy = deploy::new_transfer(
        "10000",
        None,
        uref,
        "1",
        deploy_params(),
        PaymentStrParams::with_amount("100"),
        false,
    );

    assert!(transfer_deploy.is_ok());
    assert!(matches!(
        transfer_deploy.unwrap().session(),
        ExecutableDeployItem::Transfer { .. }
    ));
}

#[test]
fn should_fail_to_create_transfer_with_bad_args() {
    let transfer_deploy = deploy::new_transfer(
        "10000",
        None,
        "bad public key.",
        "1",
        deploy_params(),
        PaymentStrParams::with_amount("100"),
        false,
    );

    println!("{:?}", transfer_deploy);

    assert!(matches!(
        transfer_deploy,
        Err(CliError::InvalidArgument {
            context: "new_transfer target_account",
            error: _
        })
    ));
}

#[test]
fn should_create_unsigned_deploy() {
    let deploy_params = deploy_params_without_secret_key();
    let payment_params =
        PaymentStrParams::with_package_hash(PKG_HASH, VERSION, ENTRYPOINT, args_simple(), "");
    let session_params =
        SessionStrParams::with_package_hash(PKG_HASH, VERSION, ENTRYPOINT, args_simple(), "");

    let deploy =
        deploy::with_payment_and_session(deploy_params, payment_params, session_params, true)
            .unwrap();

    assert!(deploy.approvals().is_empty());
    assert_eq!(
        *deploy.header().account(),
        PublicKey::from_hex(SAMPLE_ACCOUNT).unwrap()
    );
}

#[test]
fn should_fail_to_create_deploy_with_no_session_account() {
    let deploy_params = deploy_params_without_account();
    let payment_params =
        PaymentStrParams::with_package_hash(PKG_HASH, VERSION, ENTRYPOINT, args_simple(), "");
    let session_params =
        SessionStrParams::with_package_hash(PKG_HASH, VERSION, ENTRYPOINT, args_simple(), "");

    let deploy =
        deploy::with_payment_and_session(deploy_params, payment_params, session_params, true);
    assert!(deploy.is_err());
    assert!(matches!(
        deploy.unwrap_err(),
        CliError::Core(Error::DeployBuild(
            DeployBuilderError::DeployMissingSessionAccount
        ))
    ));
}

#[test]
fn should_create_unsigned_transfer() {
    use casper_types::{AsymmetricType, PublicKey};

    // with public key.
    let secret_key = SecretKey::generate_ed25519().unwrap();
    let public_key = PublicKey::from(&secret_key).to_hex();
    let transfer_deploy = deploy::new_transfer(
        "10000",
        None,
        &public_key,
        "1",
        deploy_params_without_secret_key(),
        PaymentStrParams::with_amount("100"),
        true,
    )
    .unwrap();
    assert!(transfer_deploy.approvals().is_empty());
}

#[test]
fn should_fail_to_create_transfer_without_account() {
    use casper_types::{AsymmetricType, PublicKey};

    // with public key.
    let secret_key = SecretKey::generate_ed25519().unwrap();
    let public_key = PublicKey::from(&secret_key).to_hex();

    let transfer_deploy = deploy::new_transfer(
        "10000",
        None,
        &public_key,
        "1",
        deploy_params_without_account(),
        PaymentStrParams::with_amount("100"),
        true,
    );
    assert!(transfer_deploy.is_err());
    assert!(matches!(
        transfer_deploy.unwrap_err(),
        CliError::Core(Error::DeployBuild(
            DeployBuilderError::DeployMissingSessionAccount
        ))
    ));
}

#[test]
fn should_fail_to_create_transfer_with_no_secret_key_while_not_allowing_unsigned_deploy() {
    let deploy_params = deploy_params_without_secret_key();
    let payment_params =
        PaymentStrParams::with_package_hash(PKG_HASH, VERSION, ENTRYPOINT, args_simple(), "");

    // with public key.
    let secret_key = SecretKey::generate_ed25519().unwrap();
    let public_key = PublicKey::from(&secret_key).to_hex();

    let transfer_deploy = deploy::new_transfer(
        "10000",
        None,
        &public_key,
        "1",
        deploy_params,
        payment_params,
        false,
    );

    assert!(transfer_deploy.is_err());
    let error_string = "No secret key provided and unsigned deploys are not allowed".to_string();
    assert!(matches!(
        transfer_deploy.unwrap_err(),
        CliError::InvalidArgument {
            context: "new_transfer",
            error,
        } if error == error_string
    ));
}

#[test]
fn should_fail_to_create_deploy_with_payment_and_session_with_no_secret_key_while_not_allowing_unsigned_deploy(
) {
    let deploy_params = deploy_params_without_secret_key();
    let payment_params =
        PaymentStrParams::with_package_hash(PKG_HASH, VERSION, ENTRYPOINT, args_simple(), "");
    let session_params =
        SessionStrParams::with_package_hash(PKG_HASH, VERSION, ENTRYPOINT, args_simple(), "");

    let transfer_deploy =
        deploy::with_payment_and_session(deploy_params, payment_params, session_params, false);

    assert!(transfer_deploy.is_err());
    let error_string = "No secret key provided and unsigned deploys are not allowed".to_string();
    assert!(matches!(
        transfer_deploy.unwrap_err(),
        CliError::InvalidArgument {
            context: "with_payment_and_session",
            error,
        } if error == error_string
    ));
}

mod transaction {
    use super::*;
    use crate::{cli::TransactionV1BuilderError, Error::TransactionBuild};
    use casper_types::{
        bytesrepr::Bytes,
        system::auction::{DelegatorKind, Reservation},
        PackageAddr, TransactionArgs, TransactionEntryPoint, TransactionInvocationTarget,
        TransactionRuntimeParams, TransactionTarget, TransferTarget,
    };
    use once_cell::sync::Lazy;
    use rand::{thread_rng, Rng};
    use serde_json::json;
    static SAMPLE_TRANSACTION: Lazy<serde_json::Value> = Lazy::new(|| {
        json!({"Version1": {
            "hash": "57144349509f7cb9374e0f38b4e4910526b397a38f0dc21eaae1df916df66aae",
            "payload": {
                "initiator_addr": {
                    "PublicKey": "01722e1b3d31bef0ba832121bd2941aae6a246d0d05ac95aa16dd587cc5469871d",
                },
                "timestamp": "2024-10-07T16:45:27.994Z",
                "ttl": "30m",
                "chain_name": "test",
                "pricing_mode": {
                "Fixed": {
                    "additional_computation_factor": 0,
                    "gas_price_tolerance": 10,
                }
                },
                "fields": {
                  "args": {
                    "Named": [
                      [
                        "xyz",
                        {
                          "bytes": "0d0000001af81d860f238f832b8f8e648c",
                          "cl_type": {
                            "List": "U8"
                          }
                        }
                      ]
                    ]
                  },
                  "entry_point": "AddBid",
                  "scheduling": {
                    "FutureEra": 195120
                  },
                  "target": "Native"
                }
            },
            "approvals": [],
        }})
    });
    const SAMPLE_DIGEST: &str =
        "01722e1b3d31bef0ba832121bd2941aae6a246d0d05ac95aa16dd587cc5469871d";

    #[test]
    fn should_sign_transaction() {
        let bytes = serde_json::to_string_pretty(&*SAMPLE_TRANSACTION).unwrap();
        let transaction = crate::read_transaction(bytes.as_bytes()).unwrap();
        assert_eq!(
            transaction.approvals().len(),
            0,
            "Sample transaction should have 0 approvals."
        );

        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("deploy.json");

        crate::output_transaction(OutputKind::file(&path, false), &transaction).unwrap();

        let secret_key = SecretKey::generate_ed25519().unwrap();
        crate::sign_transaction_file(&path, &secret_key, OutputKind::file(&path, true)).unwrap();
        let signed_transaction = crate::read_transaction_file(&path).unwrap();

        assert_eq!(
            signed_transaction.approvals().len(),
            transaction.approvals().len() + 1,
        );
    }

    #[test]
    fn should_create_add_bid_transaction() {
        let secret_key = SecretKey::generate_ed25519().unwrap();
        let amount = U512::from(1000);
        let minimum_delegation_amount = 100u64;
        let maximum_delegation_amount = 10000u64;
        let public_key = PublicKey::from(&secret_key);

        let amount_cl = &CLValue::from_t(amount).unwrap();
        let public_key_cl = &CLValue::from_t(&public_key).unwrap();

        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "add-bid-test",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let transaction_builder_params = TransactionBuilderParams::AddBid {
            public_key,
            delegation_rate: 0,
            amount,
            minimum_delegation_amount: Some(minimum_delegation_amount),
            maximum_delegation_amount: Some(maximum_delegation_amount),
            reserved_slots: None,
        };

        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);

        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "add-bid-test");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("public_key")
                .unwrap(),
            public_key_cl
        );
        assert!(transaction_v1
            .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
            .unwrap()
            .into_named()
            .unwrap()
            .get("delegation_rate")
            .is_some());
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("amount")
                .unwrap(),
            amount_cl
        );
    }

    #[test]
    fn should_create_delegate_transaction() {
        let delegator_secret_key = SecretKey::generate_ed25519().unwrap();
        let validator_secret_key = SecretKey::generate_ed25519().unwrap();

        let delegator_public_key = PublicKey::from(&delegator_secret_key);
        let validator_public_key = PublicKey::from(&validator_secret_key);
        let amount = U512::from(2000);

        let delegator_public_key_cl = &CLValue::from_t(delegator_public_key).unwrap();
        let validator_public_key_cl = &CLValue::from_t(validator_public_key).unwrap();
        let amount_cl = &CLValue::from_t(amount).unwrap();

        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "delegate",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let transaction_builder_params = TransactionBuilderParams::Delegate {
            delegator: PublicKey::from(&delegator_secret_key),
            validator: PublicKey::from(&validator_secret_key),
            amount,
        };

        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);

        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "delegate");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("amount")
                .unwrap(),
            amount_cl
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("delegator")
                .unwrap(),
            delegator_public_key_cl
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("validator")
                .unwrap(),
            validator_public_key_cl
        );
    }

    #[test]
    fn should_create_activate_bid_transaction() {
        let secret_key = SecretKey::generate_ed25519().unwrap();

        let validator = PublicKey::from(&secret_key);

        let validator_cl = &CLValue::from_t(&validator).unwrap();

        let transaction_str_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "activate-bid",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "0",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };
        let transaction_string_params = transaction_str_params;

        let transaction_builder_params = TransactionBuilderParams::ActivateBid { validator };

        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);

        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "activate-bid");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("validator")
                .unwrap(),
            validator_cl
        );
    }

    #[test]
    fn should_create_withdraw_bid_transaction() {
        let secret_key = SecretKey::generate_ed25519().unwrap();

        let public_key = PublicKey::from(&secret_key);
        let amount = U512::from(3000);

        let public_key_cl = &CLValue::from_t(&public_key).unwrap();
        let amount_cl = &CLValue::from_t(amount).unwrap();

        let transaction_str_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "withdraw-bid",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "0",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };
        let transaction_string_params = transaction_str_params;

        let transaction_builder_params =
            TransactionBuilderParams::WithdrawBid { public_key, amount };

        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);

        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "withdraw-bid");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("amount")
                .unwrap(),
            amount_cl
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("public_key")
                .unwrap(),
            public_key_cl
        );
    }

    #[test]
    fn should_create_undelegatge_transaction() {
        let delegator_secret_key = SecretKey::generate_ed25519().unwrap();
        let validator_secret_key = SecretKey::generate_ed25519().unwrap();

        let amount = U512::from(4000);
        let delegator_public_key = PublicKey::from(&delegator_secret_key);
        let validator_public_key = PublicKey::from(&validator_secret_key);

        let amount_cl = &CLValue::from_t(amount).unwrap();
        let delegator_public_key_cl = &CLValue::from_t(delegator_public_key).unwrap();
        let validator_public_key_cl = &CLValue::from_t(validator_public_key).unwrap();

        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "undelegate",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "0",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let transaction_builder_params = TransactionBuilderParams::Undelegate {
            delegator: PublicKey::from(&delegator_secret_key),
            validator: PublicKey::from(&validator_secret_key),
            amount,
        };

        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);

        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "undelegate");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("amount")
                .unwrap(),
            amount_cl
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("delegator")
                .unwrap(),
            delegator_public_key_cl
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("validator")
                .unwrap(),
            validator_public_key_cl
        );
    }

    #[test]
    fn should_create_redelegatge_transaction() {
        let delegator_secret_key = SecretKey::generate_ed25519().unwrap();
        let validator_secret_key = SecretKey::generate_ed25519().unwrap();
        let new_validator_secret_key = SecretKey::generate_ed25519().unwrap();

        let delegator_public_key = PublicKey::from(&delegator_secret_key);
        let validator_public_key = PublicKey::from(&validator_secret_key);
        let new_validator_public_key = PublicKey::from(&new_validator_secret_key);
        let amount = U512::from(5000);

        let delegator_public_key_cl = &CLValue::from_t(delegator_public_key).unwrap();
        let validator_public_key_cl = &CLValue::from_t(validator_public_key).unwrap();
        let new_validator_public_key_cl = &CLValue::from_t(new_validator_public_key).unwrap();
        let amount_cl = &CLValue::from_t(amount).unwrap();

        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "redelegate",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let transaction_builder_params = TransactionBuilderParams::Redelegate {
            delegator: PublicKey::from(&delegator_secret_key),
            validator: PublicKey::from(&validator_secret_key),
            amount,
            new_validator: PublicKey::from(&new_validator_secret_key),
        };
        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);
        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "redelegate");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("amount")
                .unwrap(),
            amount_cl
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("delegator")
                .unwrap(),
            delegator_public_key_cl
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("validator")
                .unwrap(),
            validator_public_key_cl
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("new_validator")
                .unwrap(),
            new_validator_public_key_cl
        );
    }

    #[test]
    fn should_create_change_bid_public_key_transaction() {
        let secret_key = SecretKey::generate_ed25519().unwrap();
        let public_key = PublicKey::from(&secret_key);

        let secret_key = SecretKey::generate_ed25519().unwrap();
        let new_public_key = PublicKey::from(&secret_key);

        let public_key_cl = &CLValue::from_t(&public_key).unwrap();
        let new_public_key_cl = &CLValue::from_t(&new_public_key).unwrap();

        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "change-bid-public-key-test",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let transaction_builder_params = TransactionBuilderParams::ChangeBidPublicKey {
            public_key,
            new_public_key,
        };

        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);

        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "change-bid-public-key-test");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("public_key")
                .unwrap(),
            public_key_cl
        );
        assert!(transaction_v1
            .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
            .unwrap()
            .into_named()
            .unwrap()
            .get("new_public_key")
            .is_some());
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("new_public_key")
                .unwrap(),
            new_public_key_cl
        );
    }

    #[test]
    fn should_create_add_reservations_transaction() {
        let mut rng = thread_rng();

        let reservations = (0..rng.gen_range(1..10))
            .map(|_| {
                let secret_key = SecretKey::generate_ed25519().unwrap();
                let validator_public_key = PublicKey::from(&secret_key);

                let delegator_kind = rng.gen();

                let delegation_rate = rng.gen_range(0..50);
                Reservation::new(validator_public_key, delegator_kind, delegation_rate)
            })
            .collect();

        let reservations_cl = &CLValue::from_t(&reservations).unwrap();

        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "add-reservations-test",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let transaction_builder_params = TransactionBuilderParams::AddReservations { reservations };

        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);

        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "add-reservations-test");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("reservations")
                .unwrap(),
            reservations_cl
        );
    }

    #[test]
    fn should_create_cancel_reservations_transaction() {
        let mut rng = thread_rng();

        let validator_secret_key = SecretKey::generate_ed25519().unwrap();
        let validator = PublicKey::from(&validator_secret_key);

        let validator_cl = &CLValue::from_t(&validator).unwrap();

        let delegators = (0..rng.gen_range(1..10))
            .map(|_| {
                let secret_key = SecretKey::generate_ed25519().unwrap();
                DelegatorKind::PublicKey(PublicKey::from(&secret_key))
            })
            .collect();

        let delegators_cl = &CLValue::from_t(&delegators).unwrap();

        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "cancel-reservations-test",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let transaction_builder_params = TransactionBuilderParams::CancelReservations {
            validator,
            delegators,
        };

        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);

        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "cancel-reservations-test");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("validator")
                .unwrap(),
            validator_cl
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("delegators")
                .unwrap(),
            delegators_cl
        );
    }

    #[test]
    fn should_create_invocable_entity_transaction() {
        let entity_addr: EntityAddr = EntityAddr::new_account([0u8; 32]);
        let entity_hash = entity_addr.value();
        let entry_point = String::from("test-entry-point");
        let params = TransactionRuntimeParams::VmCasperV1;
        let target = &TransactionTarget::Stored {
            id: TransactionInvocationTarget::ByHash(entity_hash),
            runtime: params,
        };

        let entry_point_ref = &TransactionEntryPoint::Custom(entry_point);

        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "invocable-entity",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "0",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let params = TransactionRuntimeParams::VmCasperV1;
        let transaction_builder_params = TransactionBuilderParams::InvocableEntity {
            entity_hash: entity_hash.into(),
            entry_point: "test-entry-point",
            runtime: params,
        };
        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);

        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "invocable-entity");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionEntryPoint>(ENTRY_POINT_MAP_KEY)
                .unwrap(),
            *entry_point_ref
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionTarget>(TARGET_MAP_KEY)
                .unwrap(),
            *target
        );
    }
    #[test]
    fn should_create_invocable_entity_alias_transaction() {
        let alias = String::from("alias");
        let params = TransactionRuntimeParams::VmCasperV1;
        let target = &TransactionTarget::Stored {
            id: TransactionInvocationTarget::ByName(alias),
            runtime: params,
        };
        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "invocable-entity-alias",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            receipt: SAMPLE_DIGEST,
            additional_computation_factor: "",
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let params = TransactionRuntimeParams::VmCasperV1;
        let transaction_builder_params = TransactionBuilderParams::InvocableEntityAlias {
            entity_alias: "alias",
            entry_point: "entry-point-alias",
            runtime: params,
        };
        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);
        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "invocable-entity-alias");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionEntryPoint>(ENTRY_POINT_MAP_KEY)
                .unwrap(),
            TransactionEntryPoint::Custom("entry-point-alias".to_string())
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionTarget>(TARGET_MAP_KEY)
                .unwrap(),
            *target
        );
    }

    #[test]
    fn should_create_package_transaction() {
        let package_addr: PackageAddr = vec![0u8; 32].as_slice().try_into().unwrap();
        let entry_point = "test-entry-point-package";
        let maybe_entity_version = Some(23);
        let params = TransactionRuntimeParams::VmCasperV1;
        let target = &TransactionTarget::Stored {
            id: TransactionInvocationTarget::ByPackageHash {
                addr: package_addr,
                version: maybe_entity_version,
            },
            runtime: params,
        };
        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "package",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            receipt: SAMPLE_DIGEST,
            additional_computation_factor: "",
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };
        let params = TransactionRuntimeParams::VmCasperV1;
        let transaction_builder_params = TransactionBuilderParams::Package {
            package_hash: package_addr.into(),
            entry_point,
            maybe_entity_version,
            runtime: params,
        };
        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);
        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "package");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionEntryPoint>(ENTRY_POINT_MAP_KEY)
                .unwrap(),
            TransactionEntryPoint::Custom("test-entry-point-package".to_string())
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionTarget>(TARGET_MAP_KEY)
                .unwrap(),
            *target
        );
    }

    #[test]
    fn should_create_package_alias_transaction() {
        let package_name = String::from("package-name");
        let entry_point = "test-entry-point-package";
        let maybe_entity_version = Some(23);
        let params = TransactionRuntimeParams::VmCasperV1;
        let target = &TransactionTarget::Stored {
            id: TransactionInvocationTarget::ByPackageName {
                name: package_name.clone(),
                version: maybe_entity_version,
            },
            runtime: params,
        };
        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "package",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let params = TransactionRuntimeParams::VmCasperV1;
        let transaction_builder_params = TransactionBuilderParams::PackageAlias {
            package_alias: &package_name,
            entry_point,
            maybe_entity_version,
            runtime: params,
        };
        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);
        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "package");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionEntryPoint>(ENTRY_POINT_MAP_KEY)
                .unwrap(),
            TransactionEntryPoint::Custom("test-entry-point-package".to_string())
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionTarget>(TARGET_MAP_KEY)
                .unwrap(),
            *target
        );
    }
    #[test]
    fn should_create_session_transaction() {
        let transaction_bytes = Bytes::from(vec![1u8; 32]);
        let is_install_upgrade = true;
        let params = TransactionRuntimeParams::VmCasperV1;
        let target = &TransactionTarget::Session {
            is_install_upgrade,
            runtime: params,
            module_bytes: transaction_bytes.clone(),
        };
        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "session",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "0",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let params = TransactionRuntimeParams::VmCasperV1;
        let transaction_builder_params = TransactionBuilderParams::Session {
            is_install_upgrade,
            transaction_bytes,
            runtime: params,
        };
        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);
        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "session");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionEntryPoint>(ENTRY_POINT_MAP_KEY)
                .unwrap(),
            TransactionEntryPoint::Call
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionTarget>(TARGET_MAP_KEY)
                .unwrap(),
            *target
        );
    }

    #[test]
    fn should_create_transfer_transaction() {
        let source_uref = URef::from_formatted_str(
            "uref-0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20-007",
        )
        .unwrap();
        let target_uref = URef::from_formatted_str(
            "uref-0202030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20-007",
        )
        .unwrap();

        let transfer_target = TransferTarget::URef(target_uref);

        let maybe_source = Some(source_uref);

        let source_uref_cl = &CLValue::from_t(source_uref).unwrap();
        let target_uref_cl = &CLValue::from_t(target_uref).unwrap();

        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "transfer",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "1",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };

        let transaction_builder_params = TransactionBuilderParams::Transfer {
            maybe_source,
            target: transfer_target,
            amount: Default::default(),
            maybe_id: None,
        };
        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);
        assert!(transaction.is_ok(), "{:?}", transaction);
        let transaction_v1 = unwrap_transaction(transaction);
        assert_eq!(transaction_v1.chain_name(), "transfer");
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionEntryPoint>(ENTRY_POINT_MAP_KEY)
                .unwrap(),
            TransactionEntryPoint::Transfer
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("source")
                .unwrap(),
            source_uref_cl
        );
        assert_eq!(
            transaction_v1
                .deserialize_field::<TransactionArgs>(ARGS_MAP_KEY)
                .unwrap()
                .into_named()
                .unwrap()
                .get("target")
                .unwrap(),
            target_uref_cl
        );
    }

    fn unwrap_transaction(
        transaction: Result<casper_types::Transaction, CliError>,
    ) -> casper_types::TransactionV1 {
        match transaction.unwrap() {
            casper_types::Transaction::Deploy(_) => {
                unreachable!("Expected transaction, got deploy")
            }
            casper_types::Transaction::V1(transaction_v1) => transaction_v1,
        }
    }
    #[test]
    fn should_fail_to_create_transaction_with_no_secret_or_public_key() {
        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "no-secret",
            initiator_addr: "".to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };
        let transaction_builder_params = TransactionBuilderParams::Transfer {
            maybe_source: Default::default(),
            target: TransferTarget::URef(Default::default()),
            amount: Default::default(),
            maybe_id: None,
        };
        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);
        assert!(transaction.is_err());
        assert!(matches!(
            transaction.unwrap_err(),
            CliError::Core(TransactionBuild(
                TransactionV1BuilderError::MissingInitiatorAddr
            ))
        ));
    }

    #[cfg(feature = "std-fs-io")]
    #[test]
    fn should_create_transaction_with_secret_key_but_no_initiator_addr() {
        let minimum_delegation_amount = 100u64;
        let maximum_delegation_amount = 10000u64;

        let transaction_string_params = TransactionStrParams {
            secret_key: "resources/test.pem",
            timestamp: "",
            ttl: "30min",
            chain_name: "has-secret",
            initiator_addr: "".to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "10",
            additional_computation_factor: "",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };
        let transaction_builder_params = TransactionBuilderParams::AddBid {
            public_key: PublicKey::from_hex(SAMPLE_ACCOUNT).unwrap(),
            delegation_rate: 0,
            amount: U512::from(10),
            minimum_delegation_amount: Some(minimum_delegation_amount),
            maximum_delegation_amount: Some(maximum_delegation_amount),
            reserved_slots: None,
        };
        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, true);
        assert!(transaction.is_ok(), "{:?}", transaction);
        println!("{:?}", transaction);
    }

    #[test]
    fn should_fail_to_create_transaction_with_no_secret_and_no_unsigned_transactions() {
        let minimum_delegation_amount = 100u64;
        let maximum_delegation_amount = 10000u64;

        let transaction_string_params = TransactionStrParams {
            secret_key: "",
            timestamp: "",
            ttl: "30min",
            chain_name: "no-secret-must-be-signed",
            initiator_addr: SAMPLE_ACCOUNT.to_string(),
            session_args_simple: vec![],
            session_args_json: "",
            pricing_mode: "fixed",
            output_path: "",
            payment_amount: "100",
            gas_price_tolerance: "",
            additional_computation_factor: "",
            receipt: SAMPLE_DIGEST,
            standard_payment: "true",
            transferred_value: "0",
            session_entry_point: None,
            chunked_args: None,
        };
        let transaction_builder_params = TransactionBuilderParams::AddBid {
            public_key: PublicKey::from_hex(SAMPLE_ACCOUNT).unwrap(),
            delegation_rate: 0,
            amount: U512::from(10),
            minimum_delegation_amount: Some(minimum_delegation_amount),
            maximum_delegation_amount: Some(maximum_delegation_amount),
            reserved_slots: Some(0),
        };
        let transaction =
            create_transaction(transaction_builder_params, transaction_string_params, false);
        assert!(transaction.is_err(), "{:?}", transaction);
        println!("{:?}", transaction);
        let _error_string =
            "allow_unsigned_deploy was false, but no secret key was provided".to_string();
        assert!(matches!(
            transaction.unwrap_err(),
            CliError::InvalidArgument {
                context: "create_transaction",
                error: _error_string
            }
        ));
    }
}
