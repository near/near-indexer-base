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
-- block_hash is shard_id everywhere because it make most of the joins faster
-- UNIQUE KEY usually contains block_hash because of SingleStore limitations

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
-- todo naming
CREATE TABLE account_changes
(
    account_id                 text           NOT NULL,
    block_timestamp            numeric(20, 0) NOT NULL,
    block_hash                 text           NOT NULL,
    caused_by_transaction_hash text,
    caused_by_receipt_id       text,
    update_reason              text           NOT NULL,
    nonstaked_balance          numeric(45, 0) NOT NULL,
    staked_balance             numeric(45, 0) NOT NULL,
    storage_usage              numeric(20, 0) NOT NULL,
    chunk_index_in_block       integer        NOT NULL,
    index_in_chunk             integer        NOT NULL,
    SHARD KEY (block_hash),
    SORT KEY (block_timestamp, chunk_index_in_block, index_in_chunk),
    UNIQUE KEY (block_hash, chunk_index_in_block, index_in_chunk),
    KEY (account_id) USING HASH,
    KEY (block_hash) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (caused_by_receipt_id) USING HASH,
    KEY (caused_by_transaction_hash) USING HASH
);

-- action_kind options:
--      {
--         'CREATE_ACCOUNT',
--         'DEPLOY_CONTRACT',
--         'FUNCTION_CALL',
--         'TRANSFER',
--         'STAKE',
--         'ADD_KEY',
--         'DELETE_KEY',
--         'DELETE_ACCOUNT'
--      }
CREATE TABLE action_receipts__actions
(
    block_hash             text           NOT NULL,
    block_timestamp        numeric(20, 0) NOT NULL,
    receipt_id             text           NOT NULL,
    action_kind            text           NOT NULL,
    args                   json           NOT NULL,
    predecessor_account_id text           NOT NULL,
    receiver_account_id    text           NOT NULL,
    chunk_index_in_block   integer        NOT NULL,
    index_in_chunk         integer        NOT NULL,

--     method_name AS args::%method_name PERSISTED text,
--     KEY(method_name) USING HASH

    SHARD KEY (block_hash),
    SORT KEY (block_timestamp, chunk_index_in_block, index_in_chunk),
    UNIQUE KEY (block_hash, receipt_id, index_in_chunk), -- unique (receipt_id, index_in_chunk)
    KEY (action_kind) USING HASH,
    KEY (predecessor_account_id) USING HASH,
    KEY (receiver_account_id) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (receiver_account_id, block_timestamp) USING HASH

-- TODO json field! + discuss indexes on json fields
-- https://docs.singlestore.com/db/v7.6/en/create-your-database/physical-database-schema-design/procedures-for-physical-database-schema-design/using-json.html#indexing-data-in-json-columns
-- CREATE INDEX action_receipt_actions_args_receiver_id_idx ON action_receipt_actions ((args -> 'args_json' ->> 'receiver_id')) WHERE action_receipt_actions.action_kind = 'FUNCTION_CALL' AND
--           (action_receipt_actions.args ->> 'args_json') IS NOT NULL;
);

CREATE TABLE action_receipts__input_data
(
    block_hash           text           NOT NULL,
    block_timestamp      numeric(20, 0) NOT NULL,
    receipt_id           text           NOT NULL,
    input_data_id        text           NOT NULL,
    chunk_index_in_block integer        NOT NULL,
    index_in_chunk       integer        NOT NULL,

    SHARD KEY (block_hash),
    SORT KEY (block_timestamp, chunk_index_in_block, index_in_chunk),
    UNIQUE KEY (block_hash, receipt_id, input_data_id), -- unique (receipt_id, input_data_id)
    KEY (block_timestamp) USING HASH,
    KEY (input_data_id) USING HASH,
    KEY (receipt_id) USING HASH
);

CREATE TABLE action_receipts__output_data
(
    block_hash           text           NOT NULL,
    block_timestamp      numeric(20, 0) NOT NULL,
    receipt_id           text           NOT NULL,
    output_data_id       text           NOT NULL,
    receiver_account_id  text           NOT NULL,
    chunk_index_in_block integer        NOT NULL,
    index_in_chunk       integer        NOT NULL,

    SHARD KEY (block_hash),
    SORT KEY (block_timestamp, chunk_index_in_block, index_in_chunk),
    UNIQUE KEY (block_hash, receipt_id, output_data_id), -- unique (receipt_id, output_data_id)
    KEY (block_timestamp) USING HASH,
    KEY (output_data_id) USING HASH,
    KEY (receipt_id) USING HASH,
    KEY (receiver_account_id) USING HASH
);

CREATE TABLE action_receipts
(
    receipt_id                       text           NOT NULL,
    block_hash                       text           NOT NULL,
    chunk_hash                       text           NOT NULL,
    block_timestamp                  numeric(20, 0) NOT NULL,
    chunk_index_in_block             integer        NOT NULL,
    index_in_chunk                   integer        NOT NULL,
    predecessor_account_id           text           NOT NULL,
    receiver_account_id              text           NOT NULL,
    originated_from_transaction_hash text           NOT NULL,
    signer_account_id                text           NOT NULL,
    signer_public_key                text           NOT NULL,
--     todo what is it? asked Marcin about that
    gas_price                        numeric(45, 0) NOT NULL,

    SHARD KEY (block_hash),
    SORT KEY (block_timestamp, chunk_index_in_block, index_in_chunk),
    UNIQUE KEY (block_hash, receipt_id), -- receipt_id is unique
    KEY (block_hash) USING HASH,
    KEY (chunk_hash) USING HASH,
    KEY (predecessor_account_id) USING HASH,
    KEY (receiver_account_id) USING HASH,
    KEY (block_timestamp) USING HASH,
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
--     todo next_block_gas_price?
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
    block_timestamp   numeric(20, 0) NOT NULL,
    block_hash        text           NOT NULL,
    chunk_hash        text           NOT NULL,
    index_in_block    integer        NOT NULL,
    signature         text           NOT NULL,
    gas_limit         numeric(20, 0) NOT NULL,
    gas_used          numeric(20, 0) NOT NULL,
    author_account_id text           NOT NULL,

    SHARD KEY (block_hash),
    SORT KEY (block_timestamp, index_in_block),
    UNIQUE KEY (block_hash, chunk_hash), -- chunk_hash is unique
    KEY (block_timestamp) USING HASH,
    KEY (block_hash) USING HASH
);

CREATE TABLE data_receipts
(
    receipt_id                       text           NOT NULL,
    block_hash                       text           NOT NULL,
    chunk_hash                       text           NOT NULL,
    block_timestamp                  numeric(20, 0) NOT NULL,
    chunk_index_in_block             integer        NOT NULL,
    index_in_chunk                   integer        NOT NULL,
    predecessor_account_id           text           NOT NULL,
    receiver_account_id              text           NOT NULL,
    originated_from_transaction_hash text           NOT NULL,
    data_id                          text           NOT NULL,
    -- https://docs.singlestore.com/managed-service/en/reference/sql-reference/data-types/blob-types.html
    data                             longblob,

    SHARD KEY (block_hash),
    SORT KEY (block_timestamp, chunk_index_in_block, index_in_chunk),
    UNIQUE KEY (block_hash, receipt_id), -- receipt_id is unique
    KEY (receipt_id) USING HASH,
    KEY (data_id) USING HASH,
    KEY (block_hash) USING HASH,
    KEY (chunk_hash) USING HASH,
    KEY (predecessor_account_id) USING HASH,
    KEY (receiver_account_id) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (originated_from_transaction_hash) USING HASH
);

CREATE TABLE execution_outcomes__receipts
(
    block_hash           text           NOT NULL,
    block_timestamp      numeric(20, 0) NOT NULL,
    executed_receipt_id  text           NOT NULL,
    produced_receipt_id  text           NOT NULL,
    chunk_index_in_block integer        NOT NULL,
    index_in_chunk       integer        NOT NULL,

    SHARD KEY (block_hash),
    SORT KEY (block_timestamp, chunk_index_in_block, index_in_chunk),
    UNIQUE KEY (block_hash, executed_receipt_id, produced_receipt_id), -- unique (executed_receipt_id, produced_receipt_id)
    KEY (block_timestamp) USING HASH,
    KEY (produced_receipt_id) USING HASH
);

-- status options:
--      {
--         'UNKNOWN',
--         'FAILURE',
--         'SUCCESS_VALUE',
--         'SUCCESS_RECEIPT_ID'
--      }
-- todo we want to store more data for this table and maybe for the others
CREATE TABLE execution_outcomes
(
    receipt_id           text           NOT NULL,
    block_hash           text           NOT NULL,
    block_timestamp      numeric(20, 0) NOT NULL,
    chunk_index_in_block integer        NOT NULL,
    index_in_chunk       integer        NOT NULL,
    gas_burnt            numeric(20, 0) NOT NULL,
    tokens_burnt         numeric(45, 0) NOT NULL,
    executor_account_id  text           NOT NULL,
    status               text           NOT NULL,
    shard_id             numeric(20, 0) NOT NULL,

    SHARD KEY (block_hash),
    SORT KEY (block_timestamp, chunk_index_in_block, index_in_chunk),
    UNIQUE KEY (block_hash, receipt_id), -- receipt_id is unique
    KEY (block_timestamp) USING HASH,
    KEY (block_hash) USING HASH,
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
    block_hash                      text           NOT NULL,
    chunk_hash                      text           NOT NULL,
    block_timestamp                 numeric(20, 0) NOT NULL,
    chunk_index_in_block            integer        NOT NULL,
    index_in_chunk                  integer        NOT NULL,
    signer_account_id               text           NOT NULL,
    signer_public_key               text           NOT NULL,
    nonce                           numeric(20, 0) NOT NULL,
    receiver_account_id             text           NOT NULL,
    signature                       text           NOT NULL,
    status                          text           NOT NULL,
    converted_into_receipt_id       text           NOT NULL,
    receipt_conversion_gas_burnt    numeric(20, 0),
    receipt_conversion_tokens_burnt numeric(45, 0),

    SHARD KEY (block_hash),
    SORT KEY (block_timestamp, chunk_index_in_block, index_in_chunk),
    -- todo handle tx hash collisions. it's harder than we expected.
    -- transaction_hash is almost unique except https://github.com/near/near-indexer-for-explorer/issues/84
    UNIQUE KEY (block_hash, transaction_hash),
    KEY (converted_into_receipt_id) USING HASH,
    KEY (block_hash) USING HASH,
    KEY (block_timestamp) USING HASH,
    KEY (chunk_hash) USING HASH,
    KEY (signer_account_id) USING HASH,
    KEY (signer_public_key) USING HASH,
    KEY (receiver_account_id) USING HASH
);

CREATE ROWSTORE TABLE _blocks_to_rerun
(
    block_height      numeric(20, 0) NOT NULL,
    PRIMARY KEY (block_height)
);
