use bigdecimal::BigDecimal;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Account {
    pub account_id: String,
    pub created_at_block_height: BigDecimal,
    pub deleted_at_block_height: Option<BigDecimal>,
    pub created_by_receipt_id: Option<String>,
    pub deleted_by_receipt_id: Option<String>,
}

impl Account {
    pub fn new_from_receipt(
        account_id: &near_indexer_primitives::types::AccountId,
        created_by_receipt_id: &near_indexer_primitives::CryptoHash,
        created_at_block_height: near_indexer_primitives::types::BlockHeight,
    ) -> Self {
        Self {
            account_id: account_id.to_string(),
            created_at_block_height: created_at_block_height.into(),
            deleted_at_block_height: None,
            created_by_receipt_id: Some(created_by_receipt_id.to_string()),
            deleted_by_receipt_id: None,
        }
    }

    // pub fn new_from_genesis(
    //     account_id: &near_indexer::near_primitives::types::AccountId,
    //     last_update_block_height: near_indexer::near_primitives::types::BlockHeight,
    // ) -> Self {
    //     Self {
    //         account_id: account_id.to_string(),
    //         created_by_receipt_id: None,
    //         deleted_by_receipt_id: None,
    //         last_update_block_height: last_update_block_height.into(),
    //     }
    // }
}
