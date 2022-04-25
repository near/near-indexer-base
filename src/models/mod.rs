use futures::future::try_join_all;
use near_indexer_primitives::views::{
    AccessKeyPermissionView, ExecutionStatusView, StateChangeCauseView,
};
use sqlx::mysql::MySqlRow;
use sqlx::Arguments;

pub use execution_outcomes::{ExecutionOutcome, ExecutionOutcomeReceipt};
pub use near_lake_flows_into_sql::FieldCount;
pub use receipts::{ActionReceipt, ActionReceiptAction, ActionReceiptsOutput, DataReceipt};
pub use transactions::Transaction;

pub(crate) use serializers::extract_action_type_and_value_from_action_view;

pub(crate) mod account_changes;
pub(crate) mod blocks;
pub(crate) mod chunks;
pub(crate) mod execution_outcomes;
pub(crate) mod receipts;
mod serializers;
pub(crate) mod transactions;

pub trait FieldCount {
    /// Get the number of fields on a struct.
    fn field_count() -> usize;
}

pub trait MySqlMethods {
    fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments);

    fn get_query(count: usize) -> anyhow::Result<String>;

    fn name() -> String;
}

pub async fn chunked_insert<T: MySqlMethods + std::fmt::Debug>(
    pool: &sqlx::Pool<sqlx::MySql>,
    items: &[T],
    retry_count: usize,
) -> anyhow::Result<()> {
    let futures = items
        .chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT)
        .map(|items_part| insert_retry_or_panic(pool, items_part, retry_count));
    try_join_all(futures).await.map(|_| ())
}

async fn insert_retry_or_panic<T: MySqlMethods + std::fmt::Debug>(
    pool: &sqlx::Pool<sqlx::MySql>,
    items: &[T],
    retry_count: usize,
) -> anyhow::Result<()> {
    let mut interval = crate::INTERVAL;
    let mut retry_attempt = 0usize;
    let query = T::get_query(items.len())?;

    loop {
        if retry_attempt == retry_count {
            return Err(anyhow::anyhow!(
                "Failed to perform query to database after {} attempts. Stop trying.",
                retry_count
            ));
        }
        retry_attempt += 1;

        let mut args = sqlx::mysql::MySqlArguments::default();
        for item in items {
            item.add_to_args(&mut args);
        }

        match sqlx::query_with(&query, args).execute(pool).await {
            Ok(_) => break,
            Err(async_error) => {
                tracing::error!(
                         target: crate::INDEXER,
                         "Error occurred during {}:\n{} were not stored. \n{:#?} \n Retrying in {} milliseconds...",
                         async_error,
                         &T::name(),
                         &items,
                         interval.as_millis(),
                     );
                tokio::time::sleep(interval).await;
                if interval < crate::MAX_DELAY_TIME {
                    interval *= 2;
                }
            }
        }
    }
    Ok(())
}

pub async fn select_retry_or_panic(
    pool: &sqlx::Pool<sqlx::MySql>,
    query: &str,
    substitution_items: &[String],
    retry_count: usize,
) -> anyhow::Result<Vec<MySqlRow>> {
    let mut interval = crate::INTERVAL;
    let mut retry_attempt = 0usize;

    loop {
        if retry_attempt == retry_count {
            return Err(anyhow::anyhow!(
                "Failed to perform query to database after {} attempts. Stop trying.",
                retry_count
            ));
        }
        retry_attempt += 1;

        let mut args = sqlx::mysql::MySqlArguments::default();
        for item in substitution_items {
            args.add(item);
        }

        match sqlx::query_with(query, args).fetch_all(pool).await {
            Ok(res) => return Ok(res),
            Err(async_error) => {
                // todo we print here select with non-filled placeholders. It would be better to get the final select statement here
                tracing::error!(
                         target: crate::INDEXER,
                         "Error occurred during {}:\nFailed SELECT:\n{}\n Retrying in {} milliseconds...",
                         async_error,
                    query,
                         interval.as_millis(),
                     );
                tokio::time::sleep(interval).await;
                if interval < crate::MAX_DELAY_TIME {
                    interval *= 2;
                }
            }
        }
    }
}

fn create_query_with_placeholders(
    query: &str,
    mut items_count: usize,
    fields_count: usize,
) -> anyhow::Result<String> {
    if items_count < 1 {
        return Err(anyhow::anyhow!("At least 1 item expected"));
    }

    let placeholder = create_placeholder(fields_count)?;
    // Generates `INSERT INTO table VALUES (?, ?, ?), (?, ?, ?)`
    let mut res = query.to_owned() + " " + &placeholder;
    items_count -= 1;
    while items_count > 0 {
        res += ", ";
        res += &placeholder;
        items_count -= 1;
    }

    Ok(res)
}

// Generates `(?, ?, ?)`
pub fn create_placeholder(mut fields_count: usize) -> anyhow::Result<String> {
    if fields_count < 1 {
        return Err(anyhow::anyhow!("At least 1 field expected"));
    }
    let mut item = "(?".to_owned();
    fields_count -= 1;
    while fields_count > 0 {
        item += ", ?";
        fields_count -= 1;
    }
    item += ")";
    Ok(item)
}

pub(crate) trait PrintEnum {
    fn print(&self) -> &str;
}

impl PrintEnum for ExecutionStatusView {
    fn print(&self) -> &str {
        match self {
            ExecutionStatusView::Unknown => "UNKNOWN",
            ExecutionStatusView::Failure(_) => "FAILURE",
            ExecutionStatusView::SuccessValue(_) => "SUCCESS_VALUE",
            ExecutionStatusView::SuccessReceiptId(_) => "SUCCESS_RECEIPT_ID",
        }
    }
}

impl PrintEnum for AccessKeyPermissionView {
    fn print(&self) -> &str {
        match self {
            AccessKeyPermissionView::FunctionCall { .. } => "FUNCTION_CALL",
            AccessKeyPermissionView::FullAccess => "FULL_ACCESS",
        }
    }
}

impl PrintEnum for StateChangeCauseView {
    fn print(&self) -> &str {
        match self {
            StateChangeCauseView::NotWritableToDisk => {
                panic!("Unexpected variant {:?} received", self)
            }
            StateChangeCauseView::InitialState => panic!("Unexpected variant {:?} received", self),
            StateChangeCauseView::TransactionProcessing { .. } => "TRANSACTION_PROCESSING",
            StateChangeCauseView::ActionReceiptProcessingStarted { .. } => {
                "ACTION_RECEIPT_PROCESSING_STARTED"
            }
            StateChangeCauseView::ActionReceiptGasReward { .. } => "ACTION_RECEIPT_GAS_REWARD",
            StateChangeCauseView::ReceiptProcessing { .. } => "RECEIPT_PROCESSING",
            StateChangeCauseView::PostponedReceipt { .. } => "POSTPONED_RECEIPT",
            StateChangeCauseView::UpdatedDelayedReceipts => "UPDATED_DELAYED_RECEIPTS",
            StateChangeCauseView::ValidatorAccountsUpdate => "VALIDATOR_ACCOUNTS_UPDATE",
            StateChangeCauseView::Migration => "MIGRATION",
            StateChangeCauseView::Resharding => "RESHARDING",
        }
    }
}
