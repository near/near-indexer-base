-- https://docs.singlestore.com/db/v7.6/en/reference/sql-reference/data-definition-language-ddl/create-table.html--create-table
-- https://docs.singlestore.com/managed-service/en/reference/sql-reference/data-types/other-types.html

-- Short cheatsheet from the doc:
-- - all the tables in this project are columnstore tables
-- - Shard key is the way to identify which shard stores each row
-- - Unique key is a superset of the shard key
-- - There is only one sort key per table, and only one way to make fast range queries based on the sort
-- - Merging tables work fast if they have the same sort order -> we sort everything by timestamp
-- - `key ... using hash` gives fast queries by equality
-- - all the columns in hash keys should also be in shard key
-- - we don't use enum type because it' not allowed to use enums in keys

-- TODO rename the tables and the fields

-- update_reason options:
--     {
--         'TRANSACTION_PROCESSING',
--         'ACTION_RECEIPT_PROCESSING_STARTED',
--         'ACTION_RECEIPT_GAS_REWARD',
--         'RECEIPT_PROCESSING',
--         'POSTPONED_RECEIPT',
--         'UPDATED_DELAYED_RECEIPTS',
--         'VALIDATOR_ACCOUNTS_UPDATE',
--         'MIGRATION',
--         'RESHARDING'
--     }
CREATE TABLE account_changes
(
    affected_account_id                text           NOT NULL,
    changed_in_block_timestamp         numeric(20, 0) NOT NULL,
    changed_in_block_hash              text           NOT NULL,
    caused_by_transaction_hash         text,
    caused_by_receipt_id               text,
    update_reason                      text           NOT NULL,
    affected_account_nonstaked_balance numeric(45, 0) NOT NULL,
    affected_account_staked_balance    numeric(45, 0) NOT NULL,
    affected_account_storage_usage     numeric(20, 0) NOT NULL,
    index_in_block                     integer        NOT NULL,
    SHARD KEY (affected_account_id, changed_in_block_hash),
    SORT KEY (changed_in_block_timestamp, index_in_block),
    UNIQUE KEY (affected_account_id,
        changed_in_block_hash,
        caused_by_transaction_hash,
        caused_by_receipt_id,
        update_reason,
        affected_account_nonstaked_balance,
        affected_account_staked_balance,
        affected_account_storage_usage),
    KEY (affected_account_id) USING HASH,
    KEY (changed_in_block_hash) USING HASH,
    KEY (changed_in_block_timestamp) USING HASH,
    KEY (caused_by_receipt_id) USING HASH,
    KEY (caused_by_transaction_hash) USING HASH
);

CREATE TABLE create_account_action_receipts
(
    block_timestamp numeric(20, 0) NOT NULL,
    receipt_id                          text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_action_receipt             integer        NOT NULL,
    receipt_predecessor_account_id      text           NOT NULL,
    receipt_receiver_account_id         text           NOT NULL,

    -- no new fields here

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block                      integer        NOT NULL,

    SHARD KEY (receipt_id),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (receipt_id, index_in_action_receipt),
    KEY (receipt_predecessor_account_id) USING HASH,
    KEY (receipt_receiver_account_id) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (receipt_receiver_account_id, block_timestamp) USING HASH
);

CREATE TABLE deploy_contract_action_receipts
(
    block_timestamp numeric(20, 0) NOT NULL,
    receipt_id                          text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_action_receipt             integer        NOT NULL,
    receipt_predecessor_account_id      text           NOT NULL,
    receipt_receiver_account_id         text           NOT NULL,

    code_sha256 text NOT NULL,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block                      integer        NOT NULL,

    SHARD KEY (receipt_id),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (receipt_id, index_in_action_receipt),
    KEY (receipt_predecessor_account_id) USING HASH,
    KEY (receipt_receiver_account_id) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (receipt_receiver_account_id, block_timestamp) USING HASH
);

CREATE TABLE function_call_action_receipts
(
    block_timestamp numeric(20, 0) NOT NULL,
    receipt_id                          text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_action_receipt             integer        NOT NULL,
    receipt_predecessor_account_id      text           NOT NULL,
    receipt_receiver_account_id         text           NOT NULL,

    gas numeric(45, 0) NOT NULL,
    deposit numeric(45, 0) NOT NULL,
    args_json json NOT NULL,
    method_name text NOT NULL,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block                      integer        NOT NULL,

    SHARD KEY (receipt_id),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (receipt_id, index_in_action_receipt),
    KEY (receipt_predecessor_account_id) USING HASH,
    KEY (receipt_receiver_account_id) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (receipt_receiver_account_id, block_timestamp) USING HASH
);

CREATE TABLE transfer_action_receipts
(
    block_timestamp numeric(20, 0) NOT NULL,
    receipt_id                          text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_action_receipt             integer        NOT NULL,
    receipt_predecessor_account_id      text           NOT NULL,
    receipt_receiver_account_id         text           NOT NULL,

    deposit numeric(45, 0) NOT NULL,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block                      integer        NOT NULL,

    SHARD KEY (receipt_id),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (receipt_id, index_in_action_receipt),
    KEY (receipt_predecessor_account_id) USING HASH,
    KEY (receipt_receiver_account_id) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (receipt_receiver_account_id, block_timestamp) USING HASH
);

CREATE TABLE stake_action_receipts
(
    block_timestamp numeric(20, 0) NOT NULL,
    receipt_id                          text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_action_receipt             integer        NOT NULL,
    receipt_predecessor_account_id      text           NOT NULL,
    receipt_receiver_account_id         text           NOT NULL,

    stake numeric(45, 0) NOT NULL,
    public_key text NOT NULL,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block                      integer        NOT NULL,

    SHARD KEY (receipt_id),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (receipt_id, index_in_action_receipt),
    KEY (receipt_predecessor_account_id) USING HASH,
    KEY (receipt_receiver_account_id) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (receipt_receiver_account_id, block_timestamp) USING HASH
);

CREATE TABLE add_access_key_action_receipts
(
    block_timestamp numeric(20, 0) NOT NULL,
    receipt_id                          text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_action_receipt             integer        NOT NULL,
    receipt_predecessor_account_id      text           NOT NULL,
    receipt_receiver_account_id         text           NOT NULL,

    public_key text NOT NULL,
    permission_kind text NOT NULL,

    allowance numeric(45, 0),
    access_key_receiver_id text, -- null if permission_kind FULL_ACCESS
    method_names json,  -- null if permission_kind FULL_ACCESS
    -- nobody cares about nonce

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block                      integer        NOT NULL,

    SHARD KEY (receipt_id),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (receipt_id, index_in_action_receipt),
    KEY (receipt_predecessor_account_id) USING HASH,
    KEY (receipt_receiver_account_id) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (receipt_receiver_account_id, block_timestamp) USING HASH
);

CREATE TABLE delete_access_key_action_receipts
(
    block_timestamp numeric(20, 0) NOT NULL,
    receipt_id                          text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_action_receipt             integer        NOT NULL,
    receipt_predecessor_account_id      text           NOT NULL,
    receipt_receiver_account_id         text           NOT NULL,

    public_key text NOT NULL,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block                      integer        NOT NULL,

    SHARD KEY (receipt_id),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (receipt_id, index_in_action_receipt),
    KEY (receipt_predecessor_account_id) USING HASH,
    KEY (receipt_receiver_account_id) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (receipt_receiver_account_id, block_timestamp) USING HASH
);

CREATE TABLE delete_account_action_receipts
(
    block_timestamp numeric(20, 0) NOT NULL,
    receipt_id                          text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_action_receipt             integer        NOT NULL,
    receipt_predecessor_account_id      text           NOT NULL,
    receipt_receiver_account_id         text           NOT NULL,

    beneficiary_account_id text NOT NULL,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block                      integer        NOT NULL,

    SHARD KEY (receipt_id),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (receipt_id, index_in_action_receipt),
    KEY (receipt_predecessor_account_id) USING HASH,
    KEY (receipt_receiver_account_id) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (receipt_receiver_account_id, block_timestamp) USING HASH
);

CREATE TABLE action_receipt_input_data
(
    input_data_id       text           NOT NULL,
    input_to_receipt_id text           NOT NULL,

-- TODO should we add hash keys on the new columns?
    -- TODO add the column
    block_timestamp     numeric(20, 0) NOT NULL,
    -- TODO add the column
    index_in_block      integer        NOT NULL,

    SHARD KEY (input_to_receipt_id),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (input_data_id, input_to_receipt_id),
    KEY (input_data_id) USING HASH,
    KEY (input_to_receipt_id) USING HASH
);

CREATE TABLE action_receipt_output_data
(
    output_data_id         text           NOT NULL,
    output_from_receipt_id text           NOT NULL,
    receiver_account_id    text           NOT NULL,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    block_timestamp        numeric(20, 0) NOT NULL,
    -- TODO add the column
    index_in_block         integer        NOT NULL,

    SHARD KEY (output_from_receipt_id),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (output_data_id, output_from_receipt_id),
    KEY (output_data_id) USING HASH,
    KEY (output_from_receipt_id) USING HASH,
    KEY (receiver_account_id) USING HASH
);

CREATE TABLE action_receipts
(
    receipt_id                       text           NOT NULL,
    included_in_block_hash           text           NOT NULL,
    included_in_chunk_hash           text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_chunk                   integer        NOT NULL,
    included_in_block_timestamp      numeric(20, 0) NOT NULL,
    predecessor_account_id           text           NOT NULL,
    receiver_account_id              text           NOT NULL,
    originated_from_transaction_hash text           NOT NULL,
    signer_account_id                text           NOT NULL,
    signer_public_key                text           NOT NULL,
    gas_price                        numeric(45, 0) NOT NULL,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block                   integer        NOT NULL,

    SHARD KEY (receipt_id),
    SORT KEY (included_in_block_timestamp, index_in_block),
    UNIQUE KEY (receipt_id),
    KEY (included_in_block_hash) USING HASH,
    KEY (included_in_chunk_hash) USING HASH,
    KEY (predecessor_account_id) USING HASH,
    KEY (receiver_account_id) USING HASH,
    KEY (included_in_block_timestamp) USING HASH,
    KEY (originated_from_transaction_hash) USING HASH,
    KEY (signer_account_id) USING HASH
);

CREATE TABLE blocks
(
    block_height      numeric(20, 0) NOT NULL,
    block_hash        text           NOT NULL,
    prev_block_hash   text           NOT NULL,
    block_timestamp   numeric(20, 0) NOT NULL,
    total_supply      numeric(45, 0) NOT NULL,
    gas_price         numeric(45, 0) NOT NULL,
    author_account_id text           NOT NULL,

    SHARD KEY (block_hash),
    SORT KEY (block_timestamp),
    UNIQUE KEY (block_hash),
    KEY (block_height) USING HASH,
    KEY (prev_block_hash) USING HASH,
    KEY (block_timestamp) USING HASH
);

CREATE TABLE chunks
(
    included_in_block_hash text           NOT NULL,
    chunk_hash             text           NOT NULL,
    shard_id               numeric(20, 0) NOT NULL,
    signature              text           NOT NULL,
    gas_limit              numeric(20, 0) NOT NULL,
    gas_used               numeric(20, 0) NOT NULL,
    author_account_id      text           NOT NULL,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    block_timestamp        numeric(20, 0) NOT NULL,
    -- TODO add the column
    index_in_block         integer        NOT NULL,

    SHARD KEY (chunk_hash),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (chunk_hash),
    KEY (included_in_block_hash) USING HASH
);

-- TODO do we want to use MEDIUMBLOB or VARBINARY?
-- https://docs.singlestore.com/managed-service/en/reference/sql-reference/data-types/blob-types.html
CREATE TABLE data_receipts
(
    receipt_id                       text           NOT NULL,
    included_in_block_hash           text           NOT NULL,
    included_in_chunk_hash           text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_chunk                   integer        NOT NULL,
    included_in_block_timestamp      numeric(20, 0) NOT NULL,
    predecessor_account_id           text           NOT NULL,
    receiver_account_id              text           NOT NULL,
    originated_from_transaction_hash text           NOT NULL,
    data_id                          text           NOT NULL,
    data                             blob,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block                   integer        NOT NULL,

    SHARD KEY (receipt_id),
    SORT KEY (included_in_block_timestamp, index_in_block),
    UNIQUE KEY (data_id),
    KEY (receipt_id) USING HASH,
    KEY (included_in_block_hash) USING HASH,
    KEY (included_in_chunk_hash) USING HASH,
    KEY (predecessor_account_id) USING HASH,
    KEY (receiver_account_id) USING HASH,
    KEY (included_in_block_timestamp) USING HASH,
    KEY (originated_from_transaction_hash) USING HASH
);

CREATE TABLE execution_outcome_receipts
(
    executed_receipt_id        text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_execution_outcome integer        NOT NULL,
    produced_receipt_id        text           NOT NULL,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    block_timestamp            numeric(20, 0) NOT NULL,
    -- TODO add the column
    index_in_block             integer        NOT NULL,

    SHARD KEY (executed_receipt_id),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (executed_receipt_id, index_in_execution_outcome, produced_receipt_id),
    KEY (produced_receipt_id) USING HASH
);

-- status options:
--      {
--         'UNKNOWN',
--         'FAILURE',
--         'SUCCESS_VALUE',
--         'SUCCESS_RECEIPT_ID'
--      }
CREATE TABLE execution_outcomes
(
    receipt_id                  text           NOT NULL,
    executed_in_block_hash      text           NOT NULL,
    executed_in_block_timestamp numeric(20, 0) NOT NULL,
--     TODO we can drop it since we have index_in_block
    index_in_chunk              integer        NOT NULL,
    gas_burnt                   numeric(20, 0) NOT NULL,
    tokens_burnt                numeric(45, 0) NOT NULL,
    executor_account_id         text           NOT NULL,
    status                      text           NOT NULL,
    shard_id                    numeric(20, 0) NOT NULL,

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block              integer        NOT NULL,

    SHARD KEY (receipt_id),
    SORT KEY (executed_in_block_timestamp, index_in_block),
    UNIQUE KEY (receipt_id),
    KEY (executed_in_block_timestamp) USING HASH,
    KEY (executed_in_block_hash) USING HASH,
    KEY (status) USING HASH
);

-- status options:
--      {
--         'UNKNOWN',
--         'FAILURE',
--         'SUCCESS_VALUE',
--         'SUCCESS_RECEIPT_ID'
--      }
CREATE TABLE transactions
(
    transaction_hash                text           NOT NULL,
    included_in_block_hash          text           NOT NULL,
    included_in_chunk_hash          text           NOT NULL,
    --     TODO we can drop it since we have index_in_block
    index_in_chunk                  integer        NOT NULL,
    block_timestamp                 numeric(20, 0) NOT NULL,
    signer_account_id               text           NOT NULL,
    signer_public_key               text           NOT NULL,
    nonce                           numeric(20, 0) NOT NULL,
    receiver_account_id             text           NOT NULL,
    signature                       text           NOT NULL,
    status                          text           NOT NULL,
    converted_into_receipt_id       text           NOT NULL,
    receipt_conversion_gas_burnt    numeric(20, 0),
    receipt_conversion_tokens_burnt numeric(45, 0),

    -- TODO should we add hash keys on the new columns?
    -- TODO add the column
    index_in_block                  integer        NOT NULL,

    SHARD KEY (transaction_hash),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (transaction_hash),
    KEY (converted_into_receipt_id) USING HASH,
    KEY (included_in_block_hash) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (included_in_chunk_hash) USING HASH,
    KEY (signer_account_id) USING HASH,
    KEY (signer_public_key) USING HASH,
    KEY (receiver_account_id) USING HASH
);
