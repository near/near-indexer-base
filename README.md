This is a tool to move the data from S3 to SingleStore in a structured way.

## Migration

```bash
# Add the new migration
sqlx migrate add migration_name

# Apply migrations
sqlx migrate run
```


### TODOs that should be discussed

1. SQL injections (batch insert is not supported by sqlx)
   1. https://github.com/launchbadge/sqlx/issues/291
   2. https://stackoverflow.com/questions/4922345/how-many-bind-variables-can-i-use-in-a-sql-query-in-mysql-5
2. New structure for `accounts` table, get rid of insert + update, store created-deleted-created accounts. We should write the blocks one by one, in the natural order (not only about the accounts, though)
3. Get rid of insert + update in `access_keys`, proposal of the same changes in the structure as in `accounts`
4. Massive fields and tables renaming (want your ideas)
5. Our plan to store `account_changes`
6. `transactions` and compound primary key
7. How to store genesis
8. How to handle enums better
