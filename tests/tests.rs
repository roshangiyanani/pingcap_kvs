// use assert_cmd::prelude::*;
// use kvs::{HashMapKvs, KvStore, Result};
// use predicates::ord::eq;
// use predicates::str::{contains, is_empty, PredicateStrExt};
// use std::process::Command;
// use tempfile::TempDir;
// use walkdir::WalkDir;

// // Insert data until total size of the directory decreases.
// // Test data correctness after compaction.
// #[test]
// fn compaction() -> Result<()> {
//     let temp_dir =
//         TempDir::new().expect("unable to create temporary working
// directory");     let mut store = HashMapKvs::open(temp_dir.path())?;

//     let dir_size = || {
//         let entries = WalkDir::new(temp_dir.path()).into_iter();
//         let len: walkdir::Result<u64> = entries
//             .map(|res| {
//                 res.and_then(|entry| entry.metadata())
//                     .map(|metadata| metadata.len())
//             })
//             .sum();
//         len.expect("fail to get directory size")
//     };

//     let mut current_size = dir_size();
//     for iter in 0..1000 {
//         for key_id in 0..1000 {
//             let key = format!("key{}", key_id);
//             let value = format!("{}", iter);
//             store.set(key, value)?;
//         }

//         let new_size = dir_size();
//         if new_size > current_size {
//             current_size = new_size;
//             continue;
//         }
//         // Compaction triggered

//         drop(store);
//         // reopen and check content
//         let mut store = HashMapKvs::open(temp_dir.path())?;
//         for key_id in 0..1000 {
//             let key = format!("key{}", key_id);
//             assert_eq!(store.get(key)?, Some(format!("{}", iter)));
//         }
//         return Ok(());
//     }

//     panic!("No compaction detected");
// }
