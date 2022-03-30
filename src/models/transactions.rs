use std::fmt;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use sqlx::Arguments;

use crate::models::{FieldCount, PrintEnum};

#[derive(Debug, sqlx::FromRow, FieldCount)]
pub struct Transaction {
    pub transaction_hash: String,
    pub included_in_block_hash: String,
    pub included_in_chunk_hash: String,
    pub index_in_chunk: i32,
    pub block_timestamp: BigDecimal,
    pub signer_account_id: String,
    pub signer_public_key: String,
    pub nonce: BigDecimal,
    pub receiver_account_id: String,
    pub signature: String,
    // TODO it's enum, handle the enums properly
    // TODO we need to fail if we try to insert the value not in the enum, DB will not help us here
    pub status: String,
    pub converted_into_receipt_id: String,
    pub receipt_conversion_gas_burnt: BigDecimal,
    pub receipt_conversion_tokens_burnt: BigDecimal,
}

impl Transaction {
    pub fn from_indexer_transaction(
        tx: &near_indexer_primitives::IndexerTransactionWithOutcome,
        // hack for supporting duplicated transaction hashes
        transaction_hash: &str,
        converted_into_receipt_id: &str,
        block_hash: &near_indexer_primitives::CryptoHash,
        chunk_hash: &near_indexer_primitives::CryptoHash,
        block_timestamp: u64,
        index_in_chunk: i32,
    ) -> Self {
        Self {
            transaction_hash: transaction_hash.to_string(),
            included_in_block_hash: block_hash.to_string(),
            block_timestamp: block_timestamp.into(),
            index_in_chunk,
            nonce: tx.transaction.nonce.into(),
            signer_account_id: tx.transaction.signer_id.to_string(),
            signer_public_key: tx.transaction.public_key.to_string(),
            signature: tx.transaction.signature.to_string(),
            receiver_account_id: tx.transaction.receiver_id.to_string(),
            converted_into_receipt_id: converted_into_receipt_id.to_string(),
            included_in_chunk_hash: chunk_hash.to_string(),
            status: tx
                .outcome
                .execution_outcome
                .outcome
                .status
                .print()
                .to_string(),
            receipt_conversion_gas_burnt: tx.outcome.execution_outcome.outcome.gas_burnt.into(),
            receipt_conversion_tokens_burnt: BigDecimal::from_str(
                tx.outcome
                    .execution_outcome
                    .outcome
                    .tokens_burnt
                    .to_string()
                    .as_str(),
            )
            .expect("`token_burnt` must be u128"),
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.transaction_hash);
        args.add(&self.included_in_block_hash);
        args.add(&self.included_in_chunk_hash);
        args.add(&self.index_in_chunk);
        args.add(&self.block_timestamp);
        args.add(&self.signer_account_id);
        args.add(&self.signer_public_key);
        args.add(&self.nonce);
        args.add(&self.receiver_account_id);
        args.add(&self.signature);
        args.add(&self.status);
        args.add(&self.converted_into_receipt_id);
        args.add(&self.receipt_conversion_gas_burnt);
        args.add(&self.receipt_conversion_tokens_burnt);
    }

    pub fn get_query(transactions_count: usize) -> anyhow::Result<String> {
        return crate::models::create_query_with_placeholders(
            "INSERT IGNORE INTO transactions VALUES",
            transactions_count,
            Transaction::field_count(),
        );
    }
}
