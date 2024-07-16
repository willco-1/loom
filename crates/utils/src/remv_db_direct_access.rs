use std::convert::Infallible;

use alloy_primitives::{Address, keccak256, U256};
use eyre::{eyre, Result};
use log::debug;
use revm::{DatabaseRef, InMemoryDB};

pub fn calc_hashmap_cell<U0: Into<U256>, U1: Into<U256>>(offset: U0, cell: U1) -> U256 {
    let offset: U256 = offset.into();
    let cell: U256 = cell.into();
    let mut buf: Vec<u8> = Vec::new();
    buf.extend(cell.to_be_bytes_vec());
    buf.extend(offset.to_be_bytes_vec());
    debug!("Reading cell : {} {} {:?}", offset, cell, buf);

    keccak256(buf).into()
}

pub fn try_write_cell(
    db: &mut InMemoryDB,
    account: &Address,
    cell: U256,
    value: U256,
) -> Result<()> {
    match db.accounts.get_mut(account) {
        Some(account) => {
            account.storage.insert(cell, value);
            Ok(())
        }
        None => Err(eyre!("NO_ACCOUNT")),
    }
}

pub fn try_read_cell<DB: DatabaseRef<Error=Infallible>>(db: DB, account: &Address, cell: &U256) -> Result<U256> {
    db.storage_ref(*account, *cell).map_err(|_| eyre!("READ_CELL_FAILED"))
}

pub fn try_read_hashmap_cell<DB: DatabaseRef<Error=Infallible>>(
    db: DB,
    account: &Address,
    hashmap_offset: &U256,
    item: &U256,
) -> Result<U256> {
    let mut buf = item.to_be_bytes::<32>().to_vec();
    buf.append(&mut hashmap_offset.to_be_bytes::<32>().to_vec());
    let cell: U256 = keccak256(buf.as_slice()).into();
    db.storage_ref(*account, cell).map_err(|_| eyre!("READ_HASHMAP_CELL_ERROR"))
}
