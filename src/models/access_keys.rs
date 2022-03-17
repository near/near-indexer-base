use crate::models::PrintEnum;
use bigdecimal::BigDecimal;
use std::fmt;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccessKey {
    pub public_key: String,
    pub account_id: String,
    pub created_by_receipt_id: Option<String>,
    pub deleted_by_receipt_id: Option<String>,
    // TODO
    pub permission_kind: String, //AccessKeyPermission,
    pub last_update_block_height: BigDecimal,
}

impl AccessKey {
    pub fn from_action_view(
        public_key: &near_crypto::PublicKey,
        account_id: &near_indexer_primitives::types::AccountId,
        access_key: &near_indexer_primitives::views::AccessKeyView,
        create_by_receipt_id: &near_indexer_primitives::CryptoHash,
        last_update_block_height: near_indexer_primitives::types::BlockHeight,
    ) -> Self {
        Self {
            public_key: public_key.to_string(),
            account_id: account_id.to_string(),
            created_by_receipt_id: Some(create_by_receipt_id.to_string()),
            deleted_by_receipt_id: None,
            permission_kind: access_key.permission.print().to_string(),
            last_update_block_height: last_update_block_height.into(),
        }
    }

    // pub fn from_genesis(
    //     public_key: &near_crypto::PublicKey,
    //     account_id: &near_indexer::near_primitives::types::AccountId,
    //     access_key: &near_indexer::near_primitives::account::AccessKey,
    //     last_update_block_height: near_indexer::near_primitives::types::BlockHeight,
    // ) -> Self {
    //     Self {
    //         public_key: public_key.to_string(),
    //         account_id: account_id.to_string(),
    //         created_by_receipt_id: None,
    //         deleted_by_receipt_id: None,
    //         permission_kind: (&access_key.permission).into(),
    //         last_update_block_height: last_update_block_height.into(),
    //     }
    // }
}

impl fmt::Display for AccessKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "('{}','{}','{}','{}','{}','{}')",
            self.public_key,
            self.account_id,
            // TODO !!! bug !!! we now store null as a string 'NULL'
            // TODO fix it when you will think about escaping
            self.created_by_receipt_id
                .as_ref()
                .unwrap_or(&"NULL".to_string()),
            self.deleted_by_receipt_id
                .as_ref()
                .unwrap_or(&"NULL".to_string()),
            self.permission_kind,
            self.last_update_block_height,
        )
    }
}
