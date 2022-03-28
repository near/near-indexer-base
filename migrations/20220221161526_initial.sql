-- Changes from postgres migrations:
-- - All the migrations are merged into one
-- - Single Store does not support FKs: all FKs are deleted
-- - All indexes are commented: it's better to make the research about indexes.
-- - All PKs are created right inside the corresponding table
-- - Dropped circulating supply, FT, NFT (they will be in a separate project)
-- - Dropped transaction_actions (we wanted to do that)


# Enums in single store:
# be careful with nulls, there's always a possiblity to have default value (empty string), no way to create it separately
# https://docs.singlestore.com/managed-service/en/reference/sql-reference/data-types/other-types.html

# TODO rename the table and the id
# TODO ERROR 1167 ER_WRONG_KEY_COLUMN: The used storage engine can't index column 'update_reason'
CREATE TABLE account_changes
(
    affected_account_id                text           NOT NULL, # shard id
    changed_in_block_timestamp         numeric(20, 0) NOT NULL, # sort
    changed_in_block_hash              text           NOT NULL, # shard id (?)
    caused_by_transaction_hash         text,
    caused_by_receipt_id               text,
    update_reason ENUM (
    'TRANSACTION_PROCESSING',
    'ACTION_RECEIPT_PROCESSING_STARTED',
    'ACTION_RECEIPT_GAS_REWARD',
    'RECEIPT_PROCESSING',
    'POSTPONED_RECEIPT',
    'UPDATED_DELAYED_RECEIPTS',
    'VALIDATOR_ACCOUNTS_UPDATE',
    'MIGRATION',
    'RESHARDING'
    ) NOT NULL,
    affected_account_nonstaked_balance numeric(45, 0) NOT NULL,
    affected_account_staked_balance    numeric(45, 0) NOT NULL,
    affected_account_storage_usage     numeric(20, 0) NOT NULL,
    index_in_block integer NOT NULL,
    SHARD KEY (affected_account_id, changed_in_block_hash),
    SORT KEY (changed_in_block_timestamp, index_in_block),
    UNIQUE KEY (affected_account_id,
                changed_in_block_hash,
                caused_by_transaction_hash,
                caused_by_receipt_id,
                update_reason,
                affected_account_nonstaked_balance,
                affected_account_staked_balance,
                affected_account_storage_usage)
);
# shard sort unique - here it's better to set it manually
# can put all the columns to unique, it's OK
#
# sort has no restrictions

# todo
CREATE TABLE action_receipt_actions
(
    receipt_id                          text           NOT NULL,
    index_in_action_receipt             integer        NOT NULL,
    action_kind                         ENUM (
        'CREATE_ACCOUNT',
        'DEPLOY_CONTRACT',
        'FUNCTION_CALL',
        'TRANSFER',
        'STAKE',
        'ADD_KEY',
        'DELETE_KEY',
        'DELETE_ACCOUNT')                              NOT NULL,
    args                                json           NOT NULL,
    receipt_predecessor_account_id      text           NOT NULL,
    receipt_receiver_account_id         text           NOT NULL,
    receipt_included_in_block_timestamp numeric(20, 0) NOT NULL,
    SHARD KEY (receipt_id),
    UNIQUE KEY (receipt_id, index_in_action_receipt),
    SORT KEY (receipt_included_in_block_timestamp, index_in_action_receipt)
);

# todo
CREATE TABLE action_receipt_input_data
(
    input_data_id       text NOT NULL,
    input_to_receipt_id text NOT NULL,
    PRIMARY KEY (input_data_id, input_to_receipt_id)
    SHARD KEY (input_to_receipt_id),
    UNIQUE KEY (input_data_id, input_to_receipt_id),
#     todo
    SORT KEY (receipt_included_in_block_timestamp, index_in_action_receipt)
);

# todo
CREATE TABLE action_receipt_output_data
(
    output_data_id         text NOT NULL,
    output_from_receipt_id text NOT NULL,
    receiver_account_id    text NOT NULL,
    PRIMARY KEY (output_data_id, output_from_receipt_id)
);

CREATE TABLE action_receipts
(
    receipt_id        text           NOT NULL,
    signer_account_id text           NOT NULL,
    signer_public_key text           NOT NULL,
    gas_price         numeric(45, 0) NOT NULL,
    PRIMARY KEY (receipt_id)
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
    PRIMARY KEY (block_hash)
);

CREATE TABLE chunks
(
    included_in_block_hash text           NOT NULL,
#     add the sorting column here
    chunk_hash             text           NOT NULL,
    shard_id               numeric(20, 0) NOT NULL,
    signature              text           NOT NULL,
    gas_limit              numeric(20, 0) NOT NULL,
    gas_used               numeric(20, 0) NOT NULL,
    author_account_id      text           NOT NULL,
    PRIMARY KEY (chunk_hash)
);

# https://docs.singlestore.com/managed-service/en/reference/sql-reference/data-types/blob-types.html
CREATE TABLE data_receipts
(
    data_id    text NOT NULL,
    receipt_id text NOT NULL,
    data       blob,
    PRIMARY KEY (data_id)
);

# TODO
CREATE TABLE execution_outcome_receipts
(
    executed_receipt_id        text    NOT NULL,
    index_in_execution_outcome integer NOT NULL,
    produced_receipt_id        text    NOT NULL,
    PRIMARY KEY (executed_receipt_id, index_in_execution_outcome, produced_receipt_id)
);

CREATE TABLE execution_outcomes
(
    receipt_id                  text           NOT NULL,
    executed_in_block_hash      text           NOT NULL,
    executed_in_block_timestamp numeric(20, 0) NOT NULL,
    index_in_chunk              integer        NOT NULL,
    gas_burnt                   numeric(20, 0) NOT NULL,
    tokens_burnt                numeric(45, 0) NOT NULL,
    executor_account_id         text           NOT NULL,
    status                      ENUM (
        'UNKNOWN',
        'FAILURE',
        'SUCCESS_VALUE',
        'SUCCESS_RECEIPT_ID'
        )                                      NOT NULL,
    shard_id                    numeric(20, 0) NOT NULL,
    PRIMARY KEY (receipt_id)
);

CREATE TABLE receipts
(
    receipt_id                       text           NOT NULL,
    included_in_block_hash           text           NOT NULL,
    included_in_chunk_hash           text           NOT NULL,
    index_in_chunk                   integer        NOT NULL,
    included_in_block_timestamp      numeric(20, 0) NOT NULL,
    predecessor_account_id           text           NOT NULL,
    receiver_account_id              text           NOT NULL,
    receipt_kind                     ENUM (
        'ACTION',
        'DATA'
        )                                           NOT NULL,
    originated_from_transaction_hash text           NOT NULL,
    PRIMARY KEY (receipt_id)
);

# TODO decided to use compound primary key here, need to discuss it
CREATE TABLE transactions
(
    transaction_hash                text           NOT NULL,
    included_in_block_hash          text           NOT NULL,
    included_in_chunk_hash          text           NOT NULL,
    index_in_chunk                  integer        NOT NULL,
    block_timestamp                 numeric(20, 0) NOT NULL,
    signer_account_id               text           NOT NULL,
    signer_public_key               text           NOT NULL,
    nonce                           numeric(20, 0) NOT NULL,
    receiver_account_id             text           NOT NULL,
    signature                       text           NOT NULL,
    status                          ENUM (
        'UNKNOWN',
        'FAILURE',
        'SUCCESS_VALUE',
        'SUCCESS_RECEIPT_ID'
        )                                          NOT NULL,
    converted_into_receipt_id       text           NOT NULL,
    receipt_conversion_gas_burnt    numeric(20, 0),
    receipt_conversion_tokens_burnt numeric(45, 0),
    PRIMARY KEY (transaction_hash)
);

# TODO make the research about indexes
# index: non-unique hash key on column store - no restrictions on that
# any type of strict equality
# does not help with ranges

# range scans: sort key
# one sort key per table

# or, split into 2 tables vertically and sort them separately
# in this case we need to store the same data twice



# CREATE INDEX access_keys_account_id_idx ON access_keys USING btree (account_id);
# CREATE INDEX access_keys_last_update_block_height_idx ON access_keys USING btree (last_update_block_height);
# CREATE INDEX access_keys_public_key_idx ON access_keys USING btree (public_key);
# CREATE INDEX account_changes_affected_account_id_idx ON account_changes USING btree (affected_account_id);
# CREATE INDEX account_changes_changed_in_block_hash_idx ON account_changes USING btree (changed_in_block_hash);
# CREATE INDEX account_changes_changed_in_block_timestamp_idx ON account_changes USING btree (changed_in_block_timestamp);
# CREATE INDEX account_changes_changed_in_caused_by_receipt_id_idx ON account_changes USING btree (caused_by_receipt_id);
# CREATE INDEX account_changes_changed_in_caused_by_transaction_hash_idx ON account_changes USING btree (caused_by_transaction_hash);
# CREATE INDEX accounts_last_update_block_height_idx ON accounts USING btree (last_update_block_height);
# CREATE INDEX action_receipt_input_data_input_data_id_idx ON action_receipt_input_data USING btree (input_data_id);
# CREATE INDEX action_receipt_input_data_input_to_receipt_id_idx ON action_receipt_input_data USING btree (input_to_receipt_id);
# CREATE INDEX action_receipt_output_data_output_data_id_idx ON action_receipt_output_data USING btree (output_data_id);
# CREATE INDEX action_receipt_output_data_output_from_receipt_id_idx ON action_receipt_output_data USING btree (output_from_receipt_id);
# CREATE INDEX action_receipt_output_data_receiver_account_id_idx ON action_receipt_output_data USING btree (receiver_account_id);
# CREATE INDEX action_receipt_signer_account_id_idx ON action_receipts USING btree (signer_account_id);
# CREATE INDEX blocks_height_idx ON blocks USING btree (block_height);
# CREATE INDEX blocks_prev_hash_idx ON blocks USING btree (prev_block_hash);
# CREATE INDEX blocks_timestamp_idx ON blocks USING btree (block_timestamp);
# CREATE INDEX chunks_included_in_block_hash_idx ON chunks USING btree (included_in_block_hash);
# CREATE INDEX data_receipts_receipt_id_idx ON data_receipts USING btree (receipt_id);
# CREATE INDEX execution_outcome_executed_in_block_timestamp ON execution_outcomes USING btree (executed_in_block_timestamp);
# CREATE INDEX execution_outcome_executed_in_chunk_hash_idx ON execution_outcomes USING btree (executed_in_chunk_hash);
# CREATE INDEX execution_outcome_receipts_produced_receipt_id ON execution_outcome_receipts USING btree (produced_receipt_id);
# CREATE INDEX execution_outcomes_block_hash_idx ON execution_outcomes USING btree (executed_in_block_hash);
# CREATE INDEX receipts_included_in_block_hash_idx ON receipts USING btree (included_in_block_hash);
# CREATE INDEX receipts_included_in_chunk_hash_idx ON receipts USING btree (included_in_chunk_hash);
# CREATE INDEX receipts_predecessor_account_id_idx ON receipts USING btree (predecessor_account_id);
# CREATE INDEX receipts_receiver_account_id_idx ON receipts USING btree (receiver_account_id);
# CREATE INDEX receipts_timestamp_idx ON receipts USING btree (included_in_block_timestamp);
# CREATE INDEX transactions_converted_into_receipt_id_dx ON transactions USING btree (converted_into_receipt_id);
# CREATE INDEX transactions_included_in_block_hash_idx ON transactions USING btree (included_in_block_hash);
# CREATE INDEX transactions_included_in_block_timestamp_idx ON transactions USING btree (block_timestamp);
# CREATE INDEX transactions_included_in_chunk_hash_idx ON transactions USING btree (included_in_chunk_hash);
# CREATE INDEX transactions_signer_account_id_idx ON transactions USING btree (signer_account_id);
# CREATE INDEX transactions_signer_public_key_idx ON transactions USING btree (signer_public_key);
# CREATE INDEX receipts_originated_from_transaction_hash_idx ON receipts (originated_from_transaction_hash);
# CREATE INDEX transactions_receiver_account_id_idx ON transactions (receiver_account_id);
# CREATE INDEX action_receipt_actions_action_kind_idx ON action_receipt_actions (action_kind);
# CREATE INDEX execution_outcomes_status_idx ON execution_outcomes (status);
# CREATE INDEX action_receipt_actions_receipt_predecessor_account_id_idx ON action_receipt_actions (receipt_predecessor_account_id);
# CREATE INDEX action_receipt_actions_receipt_receiver_account_id_idx ON action_receipt_actions (receipt_receiver_account_id);
# CREATE INDEX action_receipt_actions_receipt_included_in_block_timestamp_idx ON action_receipt_actions (receipt_included_in_block_timestamp);
# CREATE INDEX action_receipt_actions_args_function_call_idx ON action_receipt_actions ((args ->> 'method_name')) WHERE action_receipt_actions.action_kind = 'FUNCTION_CALL';
# CREATE INDEX action_receipt_actions_args_receiver_id_idx ON action_receipt_actions ((args -> 'args_json' ->> 'receiver_id')) WHERE action_receipt_actions.action_kind = 'FUNCTION_CALL' AND
#           (action_receipt_actions.args ->> 'args_json') IS NOT NULL;
# CREATE INDEX transactions_sorting_idx ON transactions (block_timestamp, index_in_chunk);
# CREATE INDEX account_changes_sorting_idx ON account_changes (changed_in_block_timestamp, index_in_block);
# CREATE INDEX action_receipt_actions_receiver_and_timestamp_idx
#     ON action_receipt_actions (receipt_receiver_account_id, receipt_included_in_block_timestamp);
