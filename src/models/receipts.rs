use std::str::FromStr;

use bigdecimal::BigDecimal;
use sqlx::Arguments;

use crate::models::{FieldCount, PrintEnum};

#[derive(Debug, sqlx::FromRow, FieldCount)]
pub struct Receipt {
    pub receipt_id: String,
    pub included_in_block_hash: String,
    pub included_in_chunk_hash: String,
    pub index_in_chunk: i32,
    pub included_in_block_timestamp: BigDecimal,
    pub predecessor_account_id: String,
    pub receiver_account_id: String,
    pub receipt_kind: String,
    pub originated_from_transaction_hash: String,
}

impl Receipt {
    pub fn from_receipt_view(
        receipt: &near_indexer_primitives::views::ReceiptView,
        block_hash: &near_indexer_primitives::CryptoHash,
        transaction_hash: &str,
        chunk_hash: &near_indexer_primitives::CryptoHash,
        index_in_chunk: i32,
        block_timestamp: u64,
    ) -> Self {
        Self {
            receipt_id: receipt.receipt_id.to_string(),
            included_in_block_hash: block_hash.to_string(),
            included_in_chunk_hash: chunk_hash.to_string(),
            predecessor_account_id: receipt.predecessor_id.to_string(),
            receiver_account_id: receipt.receiver_id.to_string(),
            receipt_kind: receipt.receipt.print().to_string(),
            originated_from_transaction_hash: transaction_hash.to_string(),
            index_in_chunk,
            included_in_block_timestamp: block_timestamp.into(),
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.receipt_id);
        args.add(&self.included_in_block_hash);
        args.add(&self.included_in_chunk_hash);
        args.add(&self.index_in_chunk);
        args.add(&self.included_in_block_timestamp);
        args.add(&self.predecessor_account_id);
        args.add(&self.receiver_account_id);
        args.add(&self.receipt_kind);
        args.add(&self.originated_from_transaction_hash);
    }

    pub fn get_query(receipt_count: usize) -> anyhow::Result<String> {
        crate::models::create_query_with_placeholders(
            "INSERT IGNORE INTO receipts VALUES",
            receipt_count,
            Receipt::field_count(),
        )
    }
}

#[derive(Debug, sqlx::FromRow, FieldCount)]
pub struct DataReceipt {
    pub data_id: String,
    pub receipt_id: String,
    pub data: Option<Vec<u8>>,
}

impl DataReceipt {
    pub fn try_from_data_receipt_view(
        receipt_view: &near_indexer_primitives::views::ReceiptView,
    ) -> anyhow::Result<Self> {
        if let near_indexer_primitives::views::ReceiptEnumView::Data { data_id, data } =
            &receipt_view.receipt
        {
            Ok(Self {
                receipt_id: receipt_view.receipt_id.to_string(),
                data_id: data_id.to_string(),
                data: data.clone(),
            })
        } else {
            Err(anyhow::anyhow!("Given ReceiptView is not of Data variant"))
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.data_id);
        args.add(&self.receipt_id);
        // TODO handle blobs correctly
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
    pub signer_account_id: String,
    pub signer_public_key: String,
    pub gas_price: BigDecimal,
}

impl ActionReceipt {
    pub fn try_from_action_receipt_view(
        receipt_view: &near_indexer_primitives::views::ReceiptView,
    ) -> anyhow::Result<Self> {
        if let near_indexer_primitives::views::ReceiptEnumView::Action {
            signer_id,
            signer_public_key,
            gas_price,
            ..
        } = &receipt_view.receipt
        {
            Ok(Self {
                receipt_id: receipt_view.receipt_id.to_string(),
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
    pub args: serde_json::Value,
    pub receipt_predecessor_account_id: String,
    pub receipt_receiver_account_id: String,
    pub receipt_included_in_block_timestamp: BigDecimal,
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
        let (action_kind, args) =
            crate::models::extract_action_type_and_value_from_action_view(action_view);

        Self {
            receipt_id,
            index_in_action_receipt: index,
            args,
            action_kind,
            receipt_predecessor_account_id: predecessor_account_id,
            receipt_receiver_account_id: receiver_account_id,
            receipt_included_in_block_timestamp: block_timestamp.into(),
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.receipt_id);
        args.add(&self.index_in_action_receipt);
        args.add(&self.action_kind);
        args.add(&self.args.to_string());
        args.add(&self.receipt_predecessor_account_id);
        args.add(&self.receipt_receiver_account_id);
        args.add(&self.receipt_included_in_block_timestamp);
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
    pub input_to_receipt_id: String,
    pub input_data_id: String,
}

impl ActionReceiptInputData {
    pub fn from_data_id(receipt_id: String, data_id: String) -> Self {
        Self {
            input_to_receipt_id: receipt_id,
            input_data_id: data_id,
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
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
    pub output_from_receipt_id: String,
    pub output_data_id: String,
    pub receiver_account_id: String,
}

impl ActionReceiptOutputData {
    pub fn from_data_receiver(
        receipt_id: String,
        data_receiver: &near_indexer_primitives::views::DataReceiverView,
    ) -> Self {
        Self {
            output_from_receipt_id: receipt_id,
            output_data_id: data_receiver.data_id.to_string(),
            receiver_account_id: data_receiver.receiver_id.to_string(),
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
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
