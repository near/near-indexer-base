use near_indexer_primitives::views::{AccessKeyPermissionView, ReceiptEnumView};

// pub use access_keys::AccessKey;
// pub use account_changes::AccountChange;
// pub use accounts::Account;
// pub use blocks::Block;
// pub use chunks::Chunk;
// pub use execution_outcomes::{ExecutionOutcome, ExecutionOutcomeReceipt};
pub use receipts::{
    ActionReceipt, ActionReceiptAction, ActionReceiptInputData, ActionReceiptOutputData,
    DataReceipt, Receipt,
};
pub use transactions::Transaction;

pub(crate) use serializers::extract_action_type_and_value_from_action_view;

pub(crate) mod access_keys;
pub(crate) mod accounts;
pub(crate) mod blocks;
pub(crate) mod chunks;
pub(crate) mod execution_outcomes;
pub(crate) mod receipts;
mod serializers;
pub(crate) mod transactions;

pub(crate) trait PrintEnum {
    fn print(&self) -> &str;
}

impl PrintEnum for near_indexer_primitives::views::ExecutionStatusView {
    fn print(&self) -> &str {
        match self {
            near_indexer_primitives::views::ExecutionStatusView::Unknown => "UNKNOWN",
            near_indexer_primitives::views::ExecutionStatusView::Failure(_) => "FAILURE",
            near_indexer_primitives::views::ExecutionStatusView::SuccessValue(_) => "SUCCESS_VALUE",
            near_indexer_primitives::views::ExecutionStatusView::SuccessReceiptId(_) => {
                "SUCCESS_RECEIPT_ID"
            }
        }
    }
}

impl PrintEnum for near_indexer_primitives::views::ReceiptEnumView {
    fn print(&self) -> &str {
        match self {
            ReceiptEnumView::Action { .. } => "ACTION",
            ReceiptEnumView::Data { .. } => "DATA",
        }
    }
}

impl PrintEnum for near_indexer_primitives::views::AccessKeyPermissionView {
    fn print(&self) -> &str {
        match self {
            AccessKeyPermissionView::FunctionCall { .. } => "FUNCTION_CALL",
            AccessKeyPermissionView::FullAccess => "FULL_ACCESS",
        }
    }
}
