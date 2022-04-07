use std::str::FromStr;

use bigdecimal::BigDecimal;
use near_indexer_primitives::views::{AccessKeyPermissionView, ActionView};
use num_traits::FromPrimitive;
use sqlx::Arguments;

use crate::models::{FieldCount, PrintEnum};

#[derive(Debug, sqlx::FromRow, FieldCount)]
pub struct DataReceipt {
    pub receipt_id: String,
    pub included_in_block_hash: String,
    pub included_in_chunk_hash: String,
    pub receipt_index_in_chunk: i32,
    pub included_in_block_timestamp: BigDecimal,
    pub predecessor_account_id: String,
    pub receiver_account_id: String,
    pub originated_from_transaction_hash: String,
    pub data_id: String,
    pub data: Option<Vec<u8>>,
}

impl DataReceipt {
    pub fn try_from_data_receipt_view(
        receipt: &near_indexer_primitives::views::ReceiptView,
        block_hash: &near_indexer_primitives::CryptoHash,
        transaction_hash: &str,
        chunk_hash: &near_indexer_primitives::CryptoHash,
        index_in_chunk: i32,
        block_timestamp: u64,
    ) -> anyhow::Result<Self> {
        if let near_indexer_primitives::views::ReceiptEnumView::Data { data_id, data } =
            &receipt.receipt
        {
            Ok(Self {
                receipt_id: receipt.receipt_id.to_string(),
                included_in_block_hash: block_hash.to_string(),
                included_in_chunk_hash: chunk_hash.to_string(),
                predecessor_account_id: receipt.predecessor_id.to_string(),
                receiver_account_id: receipt.receiver_id.to_string(),
                originated_from_transaction_hash: transaction_hash.to_string(),
                receipt_index_in_chunk: index_in_chunk,
                included_in_block_timestamp: block_timestamp.into(),
                data_id: data_id.to_string(),
                data: data.clone(),
            })
        } else {
            Err(anyhow::anyhow!("Given ReceiptView is not of Data variant"))
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.receipt_id);
        args.add(&self.included_in_block_hash);
        args.add(&self.included_in_chunk_hash);
        args.add(&self.receipt_index_in_chunk);
        args.add(&self.included_in_block_timestamp);
        args.add(&self.predecessor_account_id);
        args.add(&self.receiver_account_id);
        args.add(&self.originated_from_transaction_hash);
        args.add(&self.data_id);
        args.add(&self.data);
    }

    pub fn get_query(items_count: usize) -> anyhow::Result<String> {
        crate::models::create_query_with_placeholders(
            "INSERT IGNORE INTO data_receipts VALUES",
            items_count,
            DataReceipt::field_count(),
        )
    }
}

#[derive(Debug, sqlx::FromRow, FieldCount)]
pub struct ActionReceipt {
    pub receipt_id: String,
    pub included_in_block_hash: String,
    pub included_in_chunk_hash: String,
    pub receipt_index_in_chunk: i32,
    pub included_in_block_timestamp: BigDecimal,
    pub predecessor_account_id: String,
    pub receiver_account_id: String,
    pub originated_from_transaction_hash: String,
    pub signer_account_id: String,
    pub signer_public_key: String,
    pub gas_price: BigDecimal,
}

impl ActionReceipt {
    pub fn try_from_action_receipt_view(
        receipt: &near_indexer_primitives::views::ReceiptView,
        block_hash: &near_indexer_primitives::CryptoHash,
        transaction_hash: &str,
        chunk_hash: &near_indexer_primitives::CryptoHash,
        index_in_chunk: i32,
        block_timestamp: u64,
    ) -> anyhow::Result<Self> {
        if let near_indexer_primitives::views::ReceiptEnumView::Action {
            signer_id,
            signer_public_key,
            gas_price,
            ..
        } = &receipt.receipt
        {
            Ok(Self {
                receipt_id: receipt.receipt_id.to_string(),
                included_in_block_hash: block_hash.to_string(),
                included_in_chunk_hash: chunk_hash.to_string(),
                predecessor_account_id: receipt.predecessor_id.to_string(),
                receiver_account_id: receipt.receiver_id.to_string(),
                originated_from_transaction_hash: transaction_hash.to_string(),
                receipt_index_in_chunk: index_in_chunk,
                included_in_block_timestamp: block_timestamp.into(),
                signer_account_id: signer_id.to_string(),
                signer_public_key: signer_public_key.to_string(),
                gas_price: BigDecimal::from_str(gas_price.to_string().as_str())
                    .expect("gas_price expected to be u128"),
            })
        } else {
            Err(anyhow::anyhow!(
                "Given ReceiptView is not of Action variant"
            ))
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.receipt_id);
        args.add(&self.included_in_block_hash);
        args.add(&self.included_in_chunk_hash);
        args.add(&self.receipt_index_in_chunk);
        args.add(&self.included_in_block_timestamp);
        args.add(&self.predecessor_account_id);
        args.add(&self.receiver_account_id);
        args.add(&self.originated_from_transaction_hash);
        args.add(&self.signer_account_id);
        args.add(&self.signer_public_key);
        args.add(&self.gas_price);
    }

    pub fn get_query(items_count: usize) -> anyhow::Result<String> {
        crate::models::create_query_with_placeholders(
            "INSERT IGNORE INTO action_receipts VALUES",
            items_count,
            ActionReceipt::field_count(),
        )
    }
}

#[derive(Debug, sqlx::FromRow, FieldCount)]
pub struct ActionReceiptAction {
    pub receipt_id: String,
    pub index_in_action_receipt: i32,
    pub action_kind: String,
    pub receipt_predecessor_account_id: String,
    pub receipt_receiver_account_id: String,
    pub receipt_included_in_block_timestamp: BigDecimal,

    pub deployed_contract_code_sha256: Option<String>,
    pub function_call_attached_gas: Option<BigDecimal>,
    pub function_call_attached_deposit: Option<BigDecimal>,
    pub function_call_args: Option<String>,
    pub function_call_method_name: Option<String>,
    pub transfer_amount: Option<BigDecimal>,
    pub stake_amount: Option<BigDecimal>,
    pub stake_used_access_key: Option<String>,
    pub added_access_key: Option<String>,
    pub added_access_key_permission_kind: Option<String>,
    pub added_access_key_allowance: Option<BigDecimal>,
    pub added_access_key_receiver_id: Option<String>,
    pub added_access_key_method_names: Option<serde_json::Value>,
    pub deleted_access_key: Option<String>,
    pub deleted_account_beneficiary_account_id: Option<String>,
}

impl ActionReceiptAction {
    pub fn from_action_view(
        receipt_id: String,
        index: i32,
        action_view: &near_indexer_primitives::views::ActionView,
        predecessor_account_id: String,
        receiver_account_id: String,
        block_timestamp: u64,
    ) -> Self {
        let deployed_contract_code_sha256 = match action_view {
            ActionView::DeployContract { code } => Some(hex::encode(
                base64::decode(code).expect("code expected to be encoded to base64"),
            )),
            _ => None,
        };
        let (
            function_call_attached_gas,
            function_call_attached_deposit,
            function_call_args,
            function_call_method_name,
        ) = match action_view {
            ActionView::FunctionCall {
                method_name,
                args,
                gas,
                deposit,
            } => (
                Some(BigDecimal::from(*gas)),
                BigDecimal::from_u128(*deposit),
                Some(args.clone()),
                Some(method_name.escape_default().to_string()),
            ),
            _ => (None, None, None, None),
        };
        let transfer_amount = match action_view {
            ActionView::Transfer { deposit } => BigDecimal::from_u128(*deposit),
            _ => None,
        };
        let (stake_amount, stake_used_access_key) = match action_view {
            ActionView::Stake { stake, public_key } => {
                (BigDecimal::from_u128(*stake), Some(public_key.to_string()))
            }
            _ => (None, None),
        };
        let (
            added_access_key,
            added_access_key_permission_kind,
            added_access_key_allowance,
            added_access_key_receiver_id,
            added_access_key_method_names,
        ) = match action_view {
            ActionView::AddKey {
                public_key,
                access_key,
            } => {
                let (
                    added_access_key_permission_kind,
                    added_access_key_allowance,
                    added_access_key_receiver_id,
                    added_access_key_method_names,
                ) = match &access_key.permission {
                    AccessKeyPermissionView::FunctionCall {
                        allowance,
                        receiver_id,
                        method_names,
                    } => (
                        Some("FUNCTION_CALL".to_string()),
                        allowance.and_then(BigDecimal::from_u128),
                        Some(receiver_id.to_string()),
                        Some(serde_json::json!(method_names)),
                    ),
                    AccessKeyPermissionView::FullAccess => {
                        (Some("FULL_ACCESS".to_string()), None, None, None)
                    }
                };

                (
                    Some(public_key.to_string()),
                    added_access_key_permission_kind,
                    added_access_key_allowance,
                    added_access_key_receiver_id,
                    added_access_key_method_names,
                )
            }
            _ => (None, None, None, None, None),
        };
        let deleted_access_key = match action_view {
            ActionView::DeleteKey { public_key } => Some(public_key.to_string()),
            _ => None,
        };
        let deleted_account_beneficiary_account_id = match action_view {
            ActionView::DeleteAccount { beneficiary_id } => Some(beneficiary_id.to_string()),
            _ => None,
        };

        Self {
            receipt_id,
            index_in_action_receipt: index,
            action_kind: action_view.print().to_string(),
            receipt_predecessor_account_id: predecessor_account_id,
            receipt_receiver_account_id: receiver_account_id,
            receipt_included_in_block_timestamp: block_timestamp.into(),
            deployed_contract_code_sha256,
            function_call_attached_gas,
            function_call_attached_deposit,
            function_call_args,
            function_call_method_name,
            transfer_amount,
            stake_amount,
            stake_used_access_key,
            added_access_key,
            added_access_key_permission_kind,
            added_access_key_allowance,
            added_access_key_receiver_id,
            added_access_key_method_names,
            deleted_access_key,
            deleted_account_beneficiary_account_id,
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.receipt_id);
        args.add(&self.index_in_action_receipt);
        args.add(&self.action_kind);
        args.add(&self.receipt_predecessor_account_id);
        args.add(&self.receipt_receiver_account_id);
        args.add(&self.receipt_included_in_block_timestamp);

        args.add(&self.deployed_contract_code_sha256);
        args.add(&self.function_call_attached_gas);
        args.add(&self.function_call_attached_deposit);
        args.add(&self.function_call_args);
        args.add(&self.function_call_method_name);
        args.add(&self.transfer_amount);
        args.add(&self.stake_amount);
        args.add(&self.stake_used_access_key);
        args.add(&self.added_access_key);
        args.add(&self.added_access_key_permission_kind);
        args.add(&self.added_access_key_allowance);
        args.add(&self.added_access_key_receiver_id);
        args.add(
            self.added_access_key_method_names
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
        );
        args.add(&self.deleted_access_key);
        args.add(&self.deleted_account_beneficiary_account_id);
    }

    pub fn get_query(items_count: usize) -> anyhow::Result<String> {
        crate::models::create_query_with_placeholders(
            "INSERT IGNORE INTO action_receipt_actions VALUES",
            items_count,
            ActionReceiptAction::field_count(),
        )
    }
}

#[derive(Debug, sqlx::FromRow, FieldCount)]
pub struct ActionReceiptInputData {
    pub block_timestamp: BigDecimal,
    pub input_to_receipt_id: String,
    pub input_data_id: String,
}

impl ActionReceiptInputData {
    pub fn from_data_id(block_timestamp: u64, receipt_id: String, data_id: String) -> Self {
        Self {
            block_timestamp: block_timestamp.into(),
            input_to_receipt_id: receipt_id,
            input_data_id: data_id,
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.block_timestamp);
        args.add(&self.input_to_receipt_id);
        args.add(&self.input_data_id);
    }

    pub fn get_query(items_count: usize) -> anyhow::Result<String> {
        crate::models::create_query_with_placeholders(
            "INSERT IGNORE INTO action_receipt_input_data VALUES",
            items_count,
            ActionReceiptInputData::field_count(),
        )
    }
}

#[derive(Debug, sqlx::FromRow, FieldCount)]
pub struct ActionReceiptOutputData {
    pub block_timestamp: BigDecimal,
    pub output_from_receipt_id: String,
    pub output_data_id: String,
    pub receiver_account_id: String,
}

impl ActionReceiptOutputData {
    pub fn from_data_receiver(
        block_timestamp: u64,
        receipt_id: String,
        data_receiver: &near_indexer_primitives::views::DataReceiverView,
    ) -> Self {
        Self {
            block_timestamp: block_timestamp.into(),
            output_from_receipt_id: receipt_id,
            output_data_id: data_receiver.data_id.to_string(),
            receiver_account_id: data_receiver.receiver_id.to_string(),
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.block_timestamp);
        args.add(&self.output_from_receipt_id);
        args.add(&self.output_data_id);
        args.add(&self.receiver_account_id);
    }

    pub fn get_query(items_count: usize) -> anyhow::Result<String> {
        crate::models::create_query_with_placeholders(
            "INSERT IGNORE INTO action_receipt_output_data VALUES",
            items_count,
            ActionReceiptOutputData::field_count(),
        )
    }
}
