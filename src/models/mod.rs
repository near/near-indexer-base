use near_indexer_primitives::views::{AccessKeyPermissionView, ExecutionStatusView, ReceiptEnumView, StateChangeCauseView};

pub use receipts::{
    ActionReceipt, ActionReceiptAction, ActionReceiptInputData, ActionReceiptOutputData,
    DataReceipt, Receipt,
};
pub use transactions::Transaction;

pub(crate) use serializers::extract_action_type_and_value_from_action_view;

pub(crate) mod account_changes;
pub(crate) mod blocks;
pub(crate) mod chunks;
pub(crate) mod execution_outcomes;
pub(crate) mod receipts;
mod serializers;
pub(crate) mod transactions;

pub(crate) trait PrintEnum {
    fn print(&self) -> &str;
}

impl PrintEnum for ExecutionStatusView {
    fn print(&self) -> &str {
        match self {
            ExecutionStatusView::Unknown => "UNKNOWN",
            ExecutionStatusView::Failure(_) => "FAILURE",
            ExecutionStatusView::SuccessValue(_) => "SUCCESS_VALUE",
            ExecutionStatusView::SuccessReceiptId(_) => {
                "SUCCESS_RECEIPT_ID"
            }
        }
    }
}

impl PrintEnum for ReceiptEnumView {
    fn print(&self) -> &str {
        match self {
            ReceiptEnumView::Action { .. } => "ACTION",
            ReceiptEnumView::Data { .. } => "DATA",
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
            StateChangeCauseView::NotWritableToDisk => panic!("Unexpected variant {:?} received", self),
            StateChangeCauseView::InitialState => panic!("Unexpected variant {:?} received", self),
            StateChangeCauseView::TransactionProcessing { .. } => "TRANSACTION_PROCESSING",
            StateChangeCauseView::ActionReceiptProcessingStarted { .. } => "ACTION_RECEIPT_PROCESSING_STARTED",
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
