
#[derive(Debug, DbEnum, Clone)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
#[DieselType = "State_change_reason_kind"]
#[PgType = "state_change_reason_kind"]
pub enum StateChangeReasonKind {
    TransactionProcessing,
    ActionReceiptProcessingStarted,
    ActionReceiptGasReward,
    ReceiptProcessing,
    PostponedReceipt,
    UpdatedDelayedReceipts,
    ValidatorAccountsUpdate,
    Migration,
    Resharding,
}

impl From<&near_indexer::near_primitives::views::StateChangeCauseView> for StateChangeReasonKind {
    fn from(
        state_change_cause_view: &near_indexer::near_primitives::views::StateChangeCauseView,
    ) -> Self {
        match state_change_cause_view {
            near_indexer::near_primitives::views::StateChangeCauseView::TransactionProcessing { .. } => Self::TransactionProcessing,
            near_indexer::near_primitives::views::StateChangeCauseView::ActionReceiptProcessingStarted { .. } => Self::ActionReceiptProcessingStarted,
            near_indexer::near_primitives::views::StateChangeCauseView::ActionReceiptGasReward { .. } => Self::ActionReceiptGasReward,
            near_indexer::near_primitives::views::StateChangeCauseView::ReceiptProcessing { .. } => Self::ReceiptProcessing,
            near_indexer::near_primitives::views::StateChangeCauseView::PostponedReceipt { .. } => Self::PostponedReceipt,
            near_indexer::near_primitives::views::StateChangeCauseView::UpdatedDelayedReceipts { .. } => Self::UpdatedDelayedReceipts,
            near_indexer::near_primitives::views::StateChangeCauseView::ValidatorAccountsUpdate { .. } => Self::ValidatorAccountsUpdate,
            near_indexer::near_primitives::views::StateChangeCauseView::Migration { .. } => Self::Migration,
            near_indexer::near_primitives::views::StateChangeCauseView::Resharding { .. } => Self::Resharding,
            near_indexer::near_primitives::views::StateChangeCauseView::NotWritableToDisk | near_indexer::near_primitives::views::StateChangeCauseView::InitialState => panic!("Unexpected variant {:?} received", state_change_cause_view),
        }
    }
}
