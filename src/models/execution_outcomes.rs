use std::fmt;
use std::str::FromStr;

use crate::models::PrintEnum;
use bigdecimal::BigDecimal;

#[derive(Debug, sqlx::FromRow)]
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
    // TODO enums
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

    pub fn get_successful_receipts(outcomes: &[near_indexer_primitives::IndexerExecutionOutcomeWithReceipt]) -> {
        let a = outcomes
            .iter()
            .filter(|outcome_with_receipt| {
                matches!(
                outcome_with_receipt.execution_outcome.outcome.status,
                near_indexer_primitives::views::ExecutionStatusView::SuccessValue(_)
                    | near_indexer_primitives::views::ExecutionStatusView::SuccessReceiptId(_)
            )
            })
            .map(|outcome_with_receipt| &outcome_with_receipt.receipt).collect();
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct ExecutionOutcomeReceipt {
    pub executed_receipt_id: String,
    pub index_in_execution_outcome: i32,
    pub produced_receipt_id: String,
}

impl fmt::Display for ExecutionOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "('{}','{}','{}','{}','{}','{}','{}','{}','{}')",
            self.receipt_id,
            self.executed_in_block_hash,
            self.executed_in_block_timestamp,
            self.index_in_chunk,
            self.gas_burnt,
            self.tokens_burnt,
            self.executor_account_id,
            self.status,
            self.shard_id,
        )
    }
}

impl fmt::Display for ExecutionOutcomeReceipt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "('{}','{}','{}')",
            self.executed_receipt_id, self.index_in_execution_outcome, self.produced_receipt_id,
        )
    }
}
