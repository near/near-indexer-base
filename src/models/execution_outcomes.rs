use std::str::FromStr;

use bigdecimal::BigDecimal;
use sqlx::Arguments;

use crate::models::{FieldCount, PrintEnum};

#[derive(Debug, sqlx::FromRow, FieldCount)]
pub struct ExecutionOutcome {
    pub receipt_id: String,
    // TODO do we want to add block_height additionally? It could be helpful and it's cheap
    // Height is a little bit more human readable than timestamp
    pub executed_in_block_hash: String,
    // TODO don't we want to rename all such fields so that they have the same naming? It will simplify the work for our users
    // Once a week I help others to get rid of unnecessary joins in their queries. Good consistent naming could help the users
    pub executed_in_block_timestamp: BigDecimal,
    pub index_in_chunk: i32,
    pub gas_burnt: BigDecimal,
    pub tokens_burnt: BigDecimal,
    pub executor_account_id: String,
    pub status: String,
    pub shard_id: BigDecimal,
}

impl ExecutionOutcome {
    pub fn from_execution_outcome(
        execution_outcome: &near_indexer_primitives::views::ExecutionOutcomeWithIdView,
        index_in_chunk: i32,
        executed_in_block_timestamp: u64,
        shard_id: u64,
    ) -> Self {
        Self {
            executed_in_block_hash: execution_outcome.block_hash.to_string(),
            executed_in_block_timestamp: executed_in_block_timestamp.into(),
            index_in_chunk,
            receipt_id: execution_outcome.id.to_string(),
            gas_burnt: execution_outcome.outcome.gas_burnt.into(),
            tokens_burnt: BigDecimal::from_str(
                execution_outcome.outcome.tokens_burnt.to_string().as_str(),
            )
            .expect("`tokens_burnt` expected to be u128"),
            executor_account_id: execution_outcome.outcome.executor_id.to_string(),
            status: execution_outcome.outcome.status.print().to_string(),
            shard_id: shard_id.into(),
        }
    }

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.receipt_id);
        args.add(&self.executed_in_block_hash);
        args.add(&self.executed_in_block_timestamp);
        args.add(&self.index_in_chunk);
        args.add(&self.gas_burnt);
        args.add(&self.tokens_burnt);
        args.add(&self.executor_account_id);
        args.add(&self.status);
        args.add(&self.shard_id);
    }

    pub fn get_query(execution_outcome_count: usize) -> anyhow::Result<String> {
        crate::models::create_query_with_placeholders(
            "INSERT IGNORE INTO execution_outcomes VALUES",
            execution_outcome_count,
            ExecutionOutcome::field_count(),
        )
    }
}

#[derive(Debug, sqlx::FromRow, FieldCount)]
pub struct ExecutionOutcomeReceipt {
    pub executed_receipt_id: String,
    pub index_in_execution_outcome: i32,
    pub produced_receipt_id: String,
}

impl ExecutionOutcomeReceipt {
    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.executed_receipt_id);
        args.add(&self.index_in_execution_outcome);
        args.add(&self.produced_receipt_id);
    }

    pub fn get_query(execution_outcome_receipt_count: usize) -> anyhow::Result<String> {
        crate::models::create_query_with_placeholders(
            "INSERT IGNORE INTO execution_outcome_receipts VALUES",
            execution_outcome_receipt_count,
            ExecutionOutcomeReceipt::field_count(),
        )
    }
}
