use anyhow::anyhow;

pub(crate) mod account_changes;
pub(crate) mod blocks;
pub(crate) mod chunks;
pub(crate) mod execution_outcomes;
// pub(crate) mod genesis;
pub(crate) mod receipts;
pub(crate) mod transactions;
//
// const CHUNK_SIZE_FOR_BATCH_INSERT: usize = 500;

fn create_query_with_placeholders(query: &str, mut items_count: usize, mut fields_count: usize) -> anyhow::Result<String> {
    if items_count < 1 || fields_count < 1 {
        return Err(anyhow!("At least 1 item expected with at least 1 field inside"));
    }

    // Generating `(?, ?, ?)`
    let mut item = "(?".to_owned();
    fields_count -= 1;
    while fields_count > 0 {
        item += ", ?";
        fields_count -= 1;
    }
    item += ")";

    // Generating `INSERT INTO table VALUES (?, ?, ?), (?, ?, ?)`
    let mut res = query.to_owned() + " " + &item;
    items_count -= 1;
    while items_count > 0 {
        res += ", ";
        res += &item;
        items_count -= 1;
    }

    Ok(res)
}
