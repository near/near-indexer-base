use near_indexer_primitives::views::{
    AccessKeyPermissionView, ActionView, ExecutionStatusView, StateChangeCauseView,
};

pub use near_lake_flows_into_sql::FieldCount;
pub use receipts::{
    ActionReceipt, ActionReceiptAction, ActionReceiptInputData, ActionReceiptOutputData,
    DataReceipt,
};
pub use transactions::Transaction;

pub(crate) mod account_changes;
pub(crate) mod blocks;
pub(crate) mod chunks;
pub(crate) mod execution_outcomes;
pub(crate) mod receipts;
pub(crate) mod transactions;

pub trait FieldCount {
    /// Get the number of fields on a struct.
    fn field_count() -> usize;
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

impl PrintEnum for ActionView {
    fn print(&self) -> &str {
        match self {
            ActionView::CreateAccount => "CREATE_ACCOUNT",
            ActionView::DeployContract { .. } => "DEPLOY_CONTRACT",
            ActionView::FunctionCall { .. } => "FUNCTION_CALL",
            ActionView::Transfer { .. } => "TRANSFER",
            ActionView::Stake { .. } => "STAKE",
            ActionView::AddKey { .. } => "ADD_KEY",
            ActionView::DeleteKey { .. } => "DELETE_KEY",
            ActionView::DeleteAccount { .. } => "DELETE_ACCOUNT",
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
