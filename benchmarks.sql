use indexer_16_partitions;

-- We have only one writer when I do these tests. It takes ~30% of CPU on each leaf node
-- No other processes are running during this time
-- DB size is 600 Gb
-- We use S1 tier (8 vCPUs | 64 GB Memory, 1 Tb SSD storage)

-- 200 ms
select count(*) from blocks;

-- 250 ms
select count(*) from blocks
where block_height > 3000000;

-- !!! 1 minute 10 seconds first time, 21s second time. Postgres: 150 ms
select account_id from account_changes
order by block_timestamp desc
limit 100;

-- !!! 25s. Postgres: 150 ms
select * from blocks
order by block_timestamp desc
limit 100;

-- 400 ms
select * from blocks
where block_timestamp > 50000000
order by block_timestamp
limit 100;

------------------
-- Random queries from Explorer, taken from
-- https://github.com/near/near-explorer/blob/master/backend/src/db-utils.ts

-- !!! ERROR 1858 ER_TOO_MANY_SORTED_RUNS: Leaf Error (node-49a0b120-c01e-4247-acc8-88be43f66613-leaf-ag2-0.svc-49a0b120-c01e-4247-acc8-88be43f66613:3306): There are too many distinct sorted row segment groups (223) for sorted iteration on ''table account_changes''. Run ''OPTIMIZE TABLE account_changes'' and then try again.
-- after I run optimize, it took 5 sec. But it's anyway not OK that optimize was required. We can say it took 40 minutes and 5 seconds.
select blocks.block_height, account_changes.account_id, account_changes.block_timestamp, account_changes.caused_by_transaction_hash,
       account_changes.caused_by_receipt_id, account_changes.update_reason
from account_changes join blocks on account_changes.block_hash = blocks.block_hash
order by account_changes.block_timestamp
limit 100;

-- 40 mins
OPTIMIZE TABLE account_changes;

-- 35s. Postgres: 13s
-- aurora: 2s
SELECT
    receiver_account_id,
    COUNT(*) AS transactions_count
FROM transactions
WHERE receiver_account_id IN ('cheese.zest.near',
                              'miguel.zest.near',
                              'zest.near',
                              'paras.near',
                              'diagnostics.tessab.near',
                              'contract.paras.near',
                              'plutus.paras.near',
                              'berryclub.ek.near',
                              'farm.berryclub.ek.near',
                              'berryclub.near',
                              'cards.berryclub.ek.near',
                              'giveaway.paras.near',
                              'bananaswap.near',
                              'jerry.near.zest',
                              'tessab.near',
                              'amm.counselor.near')
GROUP BY receiver_account_id
ORDER BY transactions_count DESC;

-- 2s
SELECT
    COUNT(transaction_hash) AS total
FROM transactions
WHERE
        block_timestamp > UNIX_TIMESTAMP(DATE_SUB(NOW(), INTERVAL 30 day)) * 1000 * 1000 * 1000;

use indexer_16_partitions;

-- 700ms. Postgres: 4s
-- aurora: 200ms
SELECT COUNT(DISTINCT transactions.transaction_hash) AS in_transactions_count
FROM transactions
         LEFT JOIN action_receipts ON action_receipts.originated_from_transaction_hash = transactions.transaction_hash
    AND transactions.block_timestamp >= 1650067200000000000 AND transactions.block_timestamp < 1650153600000000000
WHERE action_receipts.block_timestamp >= 1650067200000000000 AND action_receipts.block_timestamp < 1650153600000000000
  AND transactions.signer_account_id != 'aurora'
  AND action_receipts.receiver_account_id = 'aurora';

-- !!! killed it after 9 minutes of waiting.  Postgres: 500 ms
-- aurora can't give the answer :( killed after 4 minutes

SELECT round(account_changes.block_timestamp / (1000 * 1000 * 1000)) AS timestamp,
       account_changes.update_reason,
       account_changes.nonstaked_balance AS nonstaked_balance,
       account_changes.staked_balance AS staked_balance,
       account_changes.storage_usage AS storage_usage,
       action_receipts.receipt_id,
       action_receipts.predecessor_account_id AS receipt_signer_id,
       action_receipts.receiver_account_id AS receipt_receiver_id,
       transactions.signer_account_id AS transaction_signer_id,
       transactions.receiver_account_id AS transaction_receiver_id,
       action_receipts__actions.action_kind AS receipt_kind,
       action_receipts__actions.args AS receipt_args
FROM account_changes
         LEFT JOIN transactions ON transactions.transaction_hash = account_changes.caused_by_transaction_hash
         LEFT JOIN action_receipts ON action_receipts.receipt_id = account_changes.caused_by_receipt_id
         LEFT JOIN action_receipts__actions ON action_receipts__actions.receipt_id = action_receipts.receipt_id
WHERE account_changes.account_id = 'aurora' and account_changes.block_timestamp < 1647578052869592081
ORDER BY account_changes.block_timestamp DESC
LIMIT 100;


SELECT round(account_changes.block_timestamp / (1000 * 1000 * 1000)) AS timestamp,
       account_changes.update_reason,
       account_changes.nonstaked_balance,
       account_changes.staked_balance,
       account_changes.storage_usage,
       transactions.signer_account_id AS transaction_signer_id,
       transactions.receiver_account_id AS transaction_receiver_id
FROM account_changes
         LEFT JOIN transactions ON transactions.transaction_hash = account_changes.caused_by_transaction_hash
WHERE account_changes.account_id = 'aurora' and account_changes.block_timestamp < 1647578052869592081
ORDER BY account_changes.block_timestamp DESC
LIMIT 100;

------------------
-- Analytical queries. Postgres is able to perform each of them in 30 s .. 7 minutes

-- https://github.com/near/near-analytics/blob/main/aggregations/db_tables/deployed_contracts.py
-- !!! I've found another problem here: we slow down `INSERT`s when running such heavy queries.
-- !!! after 12 minutes of executing, I got
-- ERROR 2470 UNKNOWN_ERR_CODE: Leaf Error (node-49a0b120-c01e-4247-acc8-88be43f66613-leaf-ag1-0.svc-49a0b120-c01e-4247-acc8-88be43f66613:3306): Leaf Error (node-49a0b120-c01e-4247-acc8-88be43f66613-leaf-ag1-0.svc-49a0b120-c01e-4247-acc8-88be43f66613:3306): Leaf Error (node-49a0b120-c01e-4247-acc8-88be43f66613-leaf-ag2-0.svc-49a0b120-c01e-4247-acc8-88be43f66613:3306): 'indexer_16_partitions_14': Failed to GetObject `prd/49a0b120-c01e-4247-acc8-88be43f66613/dbaaddab/9285098404147030757_4000/partition_14/blobs/00000000000/018/0x60ac_928509
SELECT
    action_receipts__actions.args::%code_sha256 as contract_code_sha256,
                action_receipts.receiver_account_id as deployed_to_account_id,
                action_receipts.receipt_id as deployed_by_receipt_id,
                action_receipts.block_timestamp as deployed_at_block_timestamp
FROM action_receipts__actions
    JOIN action_receipts ON action_receipts.receipt_id = action_receipts__actions.receipt_id
WHERE action_kind = 'DEPLOY_CONTRACT'
    -- random day from the last few weeks, that is fully written to the DB
    -- 16 april 2022 1650067200000000000
  AND action_receipts.block_timestamp >= 1650067200000000000
    -- 17 april 2022 1650153600000000000
  AND action_receipts.block_timestamp < 1650153600000000000
ORDER BY action_receipts.block_timestamp;

-- https://github.com/near/near-analytics/blob/main/aggregations/db_tables/daily_receipts_per_contract_count.py
-- 10s, good
SELECT
    action_receipts__actions.receiver_account_id,
    COUNT(action_receipts__actions.receipt_id) AS receipts_count
FROM action_receipts__actions
WHERE action_receipts__actions.action_kind = 'FUNCTION_CALL'
  AND action_receipts__actions.block_timestamp >= 1650067200000000000
  AND action_receipts__actions.block_timestamp < 1650153600000000000
GROUP BY action_receipts__actions.receiver_account_id;

-- https://github.com/near/near-analytics/blob/main/aggregations/db_tables/daily_ingoing_transactions_per_account_count.py
-- 7s, good
-- It has a cheat in JOIN clause, it is also helpful there. Wihout it, it runs 40s
SELECT
    action_receipts.receiver_account_id,
    COUNT(DISTINCT transactions.transaction_hash) AS ingoing_transactions_count
FROM transactions
         LEFT JOIN action_receipts ON action_receipts.originated_from_transaction_hash = transactions.transaction_hash
    AND transactions.block_timestamp >= 1650067200000000000
    AND transactions.block_timestamp < 1650153600000000000
WHERE action_receipts.block_timestamp >= 1650067200000000000
  AND action_receipts.block_timestamp < (1650153600000000000 + 600000000000)
  AND transactions.signer_account_id != action_receipts.receiver_account_id
GROUP BY action_receipts.receiver_account_id;

-- Others we can try:
-- https://github.com/near/near-analytics/blob/main/aggregations/db_tables/daily_new_contracts_count.py
-- https://github.com/near/near-analytics/blob/main/aggregations/db_tables/daily_deposit_amount.py
-- https://github.com/near/near-analytics/blob/main/aggregations/db_tables/daily_active_contracts_count.py
