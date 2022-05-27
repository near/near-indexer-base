-- NEVER run this migration if you already have some data in the tables
-- You should do this manually because it could take endless amount of time and it's better to control the process
CREATE INDEX account_changes_account_idx ON account_changes (account_id);
CREATE INDEX account_changes_block_hash_idx ON account_changes (block_hash);
CREATE INDEX account_changes_block_timestamp_idx ON account_changes (block_timestamp);
-- CREATE INDEX account_changes_receipt_id_idx ON account_changes (caused_by_receipt_id);
-- CREATE INDEX account_changes_tx_hash_idx ON account_changes (caused_by_transaction_hash);

CREATE INDEX actions_action_kind_idx ON action_receipts__actions (action_kind);
CREATE INDEX actions_predecessor_idx ON action_receipts__actions (predecessor_account_id);
CREATE INDEX actions_receiver_idx ON action_receipts__actions (receiver_account_id);
CREATE INDEX actions_block_timestamp_idx ON action_receipts__actions (block_timestamp);
CREATE INDEX actions_args_function_call_idx ON action_receipts__actions ((args ->> 'method_name')) WHERE action_kind = 'FUNCTION_CALL';
-- CREATE INDEX actions_args_receiver_id_idx ON action_receipts__actions ((args -> 'args_json' ->> 'receiver_id')) WHERE action_kind = 'FUNCTION_CALL' AND (args ->> 'args_json') IS NOT NULL;
-- CREATE INDEX actions_receiver_and_timestamp_idx ON action_receipts__actions (receiver_account_id, block_timestamp);

CREATE INDEX outputs_output_data_id_idx ON action_receipts__outputs (output_data_id);
CREATE INDEX outputs_receipt_id_idx ON action_receipts__outputs (receipt_id);
CREATE INDEX outputs_receiver_account_id_idx ON action_receipts__outputs (receiver_account_id);

CREATE INDEX action_receipts_block_hash_idx ON action_receipts (block_hash);
-- CREATE INDEX action_receipts_chunk_hash_idx ON action_receipts (chunk_hash);
CREATE INDEX action_receipts_block_timestamp_idx ON action_receipts (block_timestamp);
CREATE INDEX action_receipts_predecessor_idx ON action_receipts (predecessor_account_id);
CREATE INDEX action_receipts_receiver_idx ON action_receipts (receiver_account_id);
CREATE INDEX action_receipts_transaction_hash_idx ON action_receipts (originated_from_transaction_hash);
CREATE INDEX action_receipts_signer_idx ON action_receipts (signer_account_id);

CREATE INDEX blocks_height_idx ON blocks (block_height);
-- CREATE INDEX blocks_prev_hash_idx ON blocks (prev_block_hash);
CREATE INDEX blocks_timestamp_idx ON blocks (block_timestamp);

CREATE INDEX chunks_block_timestamp_idx ON chunks (block_timestamp);
CREATE INDEX chunks_block_hash_idx ON chunks (block_hash);

CREATE INDEX data_receipts_block_hash_idx ON data_receipts (block_hash);
-- CREATE INDEX data_receipts_chunk_hash_idx ON data_receipts (chunk_hash);
CREATE INDEX data_receipts_block_timestamp_idx ON data_receipts (block_timestamp);
CREATE INDEX data_receipts_predecessor_idx ON data_receipts (predecessor_account_id);
CREATE INDEX data_receipts_receiver_idx ON data_receipts (receiver_account_id);
CREATE INDEX data_receipts_transaction_hash_idx ON data_receipts (originated_from_transaction_hash);

CREATE INDEX execution_receipts_timestamp_idx ON execution_outcomes__receipts (block_timestamp);
CREATE INDEX execution_receipts_produced_receipt_idx ON execution_outcomes__receipts (produced_receipt_id);

CREATE INDEX execution_outcome_block_timestamp_idx ON execution_outcomes (block_timestamp);
CREATE INDEX execution_outcomes_block_hash_idx ON execution_outcomes (block_hash);
CREATE INDEX execution_outcomes_status_idx ON execution_outcomes (status);

CREATE INDEX transactions_receipt_id_idx ON transactions (converted_into_receipt_id);
CREATE INDEX transactions_block_hash_idx ON transactions (block_hash);
CREATE INDEX transactions_block_timestamp_idx ON transactions (block_timestamp);
-- CREATE INDEX transactions_chunk_hash_idx ON transactions (chunk_hash);
CREATE INDEX transactions_signer_idx ON transactions (signer_account_id);
-- CREATE INDEX transactions_signer_public_key_idx ON transactions (signer_public_key);
CREATE INDEX transactions_receiver_idx ON transactions (receiver_account_id);
-- CREATE INDEX transactions_sorting_idx ON transactions (block_timestamp, chunk_index_in_block, index_in_chunk);

-- CREATE INDEX access_keys_account_id_idx ON access_keys (account_id);
-- CREATE INDEX access_keys_last_update_block_height_idx ON access_keys (last_update_block_height);
-- CREATE INDEX access_keys_public_key_idx ON access_keys (public_key);
-- CREATE INDEX accounts_last_update_block_height_idx ON accounts (last_update_block_height);
