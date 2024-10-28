use std::env;
use std::path::PathBuf;

pub fn get_elf() -> &'static [u8] {
    include_bytes!(env!("SP1_ELF_zkdb_merkle"))
}
