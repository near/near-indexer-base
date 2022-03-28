use std::fmt;

use bigdecimal::BigDecimal;

use avro_rs::{Schema, Writer};
use serde::Serialize;


#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Chunk {
    pub included_in_block_hash: String,
    pub chunk_hash: String,
    pub shard_id: BigDecimal,
    pub signature: String,
    pub gas_limit: BigDecimal,
    pub gas_used: BigDecimal,
    pub author_account_id: String,
}

impl Chunk {
    pub fn from_chunk_view(
        chunk_view: &near_indexer_primitives::IndexerChunkView,
        block_hash: &near_indexer_primitives::CryptoHash,
    ) -> Self {
        Self {
            included_in_block_hash: block_hash.to_string(),
            chunk_hash: chunk_view.header.chunk_hash.to_string(),
            shard_id: chunk_view.header.shard_id.into(),
            signature: chunk_view.header.signature.to_string(),
            gas_limit: chunk_view.header.gas_limit.into(),
            gas_used: chunk_view.header.gas_used.into(),
            author_account_id: chunk_view.author.to_string(),
        }
    }

    pub fn schema() -> Schema {
        let raw_schema = r#"
            {
                "type": "record",
                "name": "chunk",
                "fields": [
                    {"name": "included_in_block_hash", "type": "string"},
                    {"name": "chunk_hash", "type": "string"},
                    {"name": "shard_id", "type": "string"},
                    {"name": "signature", "type": "string"},
                    {"name": "gas_limit", "type": "string"},
                    {"name": "gas_used", "type": "string"},
                    {"name": "author_account_id", "type": "string"}
                ]
            }
        "#;
        avro_rs::Schema::parse_str(raw_schema).unwrap()
    }

    pub fn format() -> String {
        r#"
            included_in_block_hash <- %::included_in_block_hash,
            chunk_hash <- %::chunk_hash,
            shard_id <- %::shard_id,
            signature <- %::signature,
            gas_limit <- %::gas_limit,
            gas_used <- %::gas_used,
            author_account_id <- %::author_account_id
        "#.to_string()
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "('{}','{}','{}','{}','{}','{}','{}')",
            self.included_in_block_hash,
            self.chunk_hash,
            self.shard_id,
            self.signature,
            self.gas_limit,
            self.gas_used,
            self.author_account_id
        )
    }
}
