-- All `text` fields now have type `varchar(64000)`
-- We spend 4 bytes on it + the length of the string, so I just took bigger limit
-- https://docs.aws.amazon.com/redshift/latest/dg/r_Character_types.html#r_Character_types-varchar-or-character-varying

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
    account_id                 varchar(64000) NOT NULL,
    block_timestamp            numeric(20, 0) NOT NULL,
    block_hash                 varchar(64000) NOT NULL,
    caused_by_transaction_hash varchar(64000),
    caused_by_receipt_id       varchar(64000),
    update_reason              varchar(64000) NOT NULL,
    nonstaked_balance          numeric(38, 0) NOT NULL,
    staked_balance             numeric(38, 0) NOT NULL,
    storage_usage              numeric(20, 0) NOT NULL,
    chunk_index_in_block       integer        NOT NULL,
    index_in_chunk             integer        NOT NULL
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
    block_hash             varchar(64000) NOT NULL,
    block_timestamp        numeric(20, 0) NOT NULL,
    receipt_id             varchar(64000) NOT NULL,
    action_kind            varchar(64000) NOT NULL,
    -- https://docs.aws.amazon.com/redshift/latest/dg/json-functions.html
    -- https://docs.aws.amazon.com/redshift/latest/dg/super-overview.html
    -- todo check that redshift can load to super (docs said that the default is varchar(64000))
    args                   super          NOT NULL,
    predecessor_account_id varchar(64000) NOT NULL,
    receiver_account_id    varchar(64000) NOT NULL,
    chunk_index_in_block   integer        NOT NULL,
    index_in_chunk         integer        NOT NULL
);

CREATE TABLE action_receipts__outputs
(
    block_hash           varchar(64000) NOT NULL,
    block_timestamp      numeric(20, 0) NOT NULL,
    receipt_id           varchar(64000) NOT NULL,
    output_data_id       varchar(64000) NOT NULL,
    receiver_account_id  varchar(64000) NOT NULL,
    chunk_index_in_block integer        NOT NULL,
    index_in_chunk       integer        NOT NULL
);

CREATE TABLE action_receipts
(
    receipt_id                       varchar(64000) NOT NULL,
    block_hash                       varchar(64000) NOT NULL,
    chunk_hash                       varchar(64000) NOT NULL,
    block_timestamp                  numeric(20, 0) NOT NULL,
    chunk_index_in_block             integer        NOT NULL,
    receipt_index_in_chunk           integer        NOT NULL, -- goes both through action and data receipts
    predecessor_account_id           varchar(64000) NOT NULL,
    receiver_account_id              varchar(64000) NOT NULL,
    originated_from_transaction_hash varchar(64000) NOT NULL,
    signer_account_id                varchar(64000) NOT NULL,
    signer_public_key                varchar(64000) NOT NULL,
    gas_price                        numeric(38, 0) NOT NULL
);

CREATE TABLE blocks
(
    block_height      numeric(20, 0) NOT NULL,
    block_hash        varchar(64000) NOT NULL,
    prev_block_hash   varchar(64000) NOT NULL,
    block_timestamp   numeric(20, 0) NOT NULL,
    total_supply      numeric(38, 0) NOT NULL,
    gas_price         numeric(38, 0) NOT NULL,
    author_account_id varchar(64000) NOT NULL
);

CREATE TABLE chunks
(
    block_timestamp   numeric(20, 0) NOT NULL,
    block_hash        varchar(64000) NOT NULL,
    chunk_hash        varchar(64000) NOT NULL,
    index_in_block    integer        NOT NULL,
    signature         varchar(64000) NOT NULL,
    gas_limit         numeric(20, 0) NOT NULL,
    gas_used          numeric(20, 0) NOT NULL,
    author_account_id varchar(64000) NOT NULL
);

CREATE TABLE data_receipts
(
    receipt_id                       varchar(64000) NOT NULL,
    block_hash                       varchar(64000) NOT NULL,
    chunk_hash                       varchar(64000) NOT NULL,
    block_timestamp                  numeric(20, 0) NOT NULL,
    chunk_index_in_block             integer        NOT NULL,
    receipt_index_in_chunk           integer        NOT NULL, -- goes both through action and data receipts
    predecessor_account_id           varchar(64000) NOT NULL,
    receiver_account_id              varchar(64000) NOT NULL,
    originated_from_transaction_hash varchar(64000) NOT NULL,
    data_id                          varchar(64000) NOT NULL,
    -- todo it's the only working way to copy data from aurora i've found. need to check that we do not corrupt binary data in varchar type
    data                             varbyte(64000)--varchar(64000)
);

CREATE TABLE execution_outcomes__receipts
(
    block_hash           varchar(64000) NOT NULL,
    block_timestamp      numeric(20, 0) NOT NULL,
    executed_receipt_id  varchar(64000) NOT NULL,
    produced_receipt_id  varchar(64000) NOT NULL,
    chunk_index_in_block integer        NOT NULL,
    index_in_chunk       integer        NOT NULL
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
    receipt_id           varchar(64000) NOT NULL,
    block_hash           varchar(64000) NOT NULL,
    block_timestamp      numeric(20, 0) NOT NULL,
    chunk_index_in_block integer        NOT NULL,
    index_in_chunk       integer        NOT NULL,
    gas_burnt            numeric(20, 0) NOT NULL,
    tokens_burnt         numeric(38, 0) NOT NULL,
    executor_account_id  varchar(64000) NOT NULL,
    status               varchar(64000) NOT NULL
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
    transaction_hash                varchar(64000) NOT NULL,
    block_hash                      varchar(64000) NOT NULL,
    chunk_hash                      varchar(64000) NOT NULL,
    block_timestamp                 numeric(20, 0) NOT NULL,
    chunk_index_in_block            integer        NOT NULL,
    index_in_chunk                  integer        NOT NULL,
    signer_account_id               varchar(64000) NOT NULL,
    signer_public_key               varchar(64000) NOT NULL,
    nonce                           numeric(20, 0) NOT NULL,
    receiver_account_id             varchar(64000) NOT NULL,
    signature                       varchar(64000) NOT NULL,
    status                          varchar(64000) NOT NULL,
    converted_into_receipt_id       varchar(64000) NOT NULL,
    receipt_conversion_gas_burnt    numeric(20, 0),
    receipt_conversion_tokens_burnt numeric(38, 0)
);

CREATE TABLE _last_successful_load
(
    block_height    numeric(20, 0) NOT NULL,
    block_timestamp numeric(20, 0) NOT NULL
);