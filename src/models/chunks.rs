use bigdecimal::BigDecimal;
use sqlx::Arguments;

use crate::models::FieldCount;

#[derive(Debug, sqlx::FromRow, FieldCount)]
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

    pub fn add_to_args(&self, args: &mut sqlx::mysql::MySqlArguments) {
        args.add(&self.included_in_block_hash);
        args.add(&self.chunk_hash);
        args.add(&self.shard_id);
        args.add(&self.signature);
        args.add(&self.gas_limit);
        args.add(&self.gas_used);
        args.add(&self.author_account_id);
    }

    pub fn get_query(chunks_count: usize) -> anyhow::Result<String> {
        return crate::models::create_query_with_placeholders(
            "INSERT IGNORE INTO chunks VALUES",
            chunks_count,
            Chunk::field_count(),
        );
    }
}
