### How to create readonly user

```sql
create group readonly;
revoke create on schema public from group readonly;
grant usage on schema public to group readonly;
grant select on all tables in schema public to group readonly;
alter default privileges in schema public grant select on tables to group readonly;
create user public_readonly with password 'Password1';
alter group readonly add user public_readonly;
```

### How to connect Redshift to Aurora DB

1. There's no ready-to-go automated process to copy data from Aurora to Redshift
2. The instruction is close to https://aws.amazon.com/blogs/big-data/announcing-amazon-redshift-federated-querying-to-amazon-aurora-mysql-and-amazon-rds-for-mysql/
3. But it has some additional details. Notes for the future me:
    1. Our instances should be in the same VPC and subnet group (it's true for the current configuration)
    2. I need to create new secret (1), new policy (by raw json), new iam role for Redshift (2), add it to redshift cluster.
    3. The external schema could be created by the command

```sql
CREATE EXTERNAL SCHEMA apg
FROM POSTGRES
DATABASE 'indexer_mainnet'
URI 'take-me-from-redshift-reader-instance-page'
IAM_ROLE 'copy-me-from-(2)-arm-line'
SECRET_ARN 'copy-me-from-(1)';
```

There is some issues with data types.
https://docs.aws.amazon.com/redshift/latest/dg/federated-data-types.html

Issues:
- All the jsons and bytearrays will be cast to varchar(64000). TODO: check that we don't break binary data here.
- It's impossible to access any data from aurora table, if you have there `numeric(>38)`. Even if you don't access this bad column and only ask for the others. So, we have to alter table in aurora and change the types.

Then, the data should be transferred this way

```sql
-- Find the right timestamp in Aurora DB
select block_timestamp from blocks where block_height = 40000000; -- 1623683746564879190

-- Run these and similar insert statements in Redshift. Don't forget to add second border if you run it second time, Redshift doesn't have unique indexes
insert into chunks select * from apg.chunks where block_timestamp <= 1623683746564879190;
insert into data_receipts select * from apg.data_receipts where block_timestamp <= 1623683746564879190;
-- ...
```

We also may want to drop some data from Aurora.
I didn't dig into it too much, but in Postgres I'd prefer to create the other partition and drop the whole previous one instead of running `delete from table where timestamp < X`.