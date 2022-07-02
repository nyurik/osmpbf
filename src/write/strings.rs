use std::cmp::Ordering;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

use crate::proto::osmformat::StringTable;

/// A utility struct to construct a PBF string table.
#[derive(Debug, Clone, Default)]
pub struct StringTableBuilder {
    /// A map of strings -> string tracking info
    strings: HashMap<String, StrInfo>,
    /// approximate total size of the string table in bytes
    size: usize,
}

#[derive(Debug, Clone)]
pub struct StrInfo {
    /// a value that [`add_string`] returned.
    /// This is not the final value that will be used in the encoded PBF.
    index: usize,
    /// Number of times this string has been added.
    usage: u32,
}

impl StringTableBuilder {
    #[must_use]
    /// add a string value to the table, returning an index of the added/existing value
    pub fn add_string(&mut self, value: String) -> usize {
        let mut new_id = self.strings.len();
        if new_id == 0 {
            self.strings.insert(
                "".to_string(),
                StrInfo {
                    index: 0,
                    usage: u32::MAX / 2,
                },
            );
            new_id += 1;
        }

        let value_len = value.len();
        // FIXME: is this allowed? What if the role is empty, should this be handled explicitly?
        // assert!(value_len > 0);

        match self.strings.entry(value) {
            Occupied(mut entry) => {
                // 2 bytes for each lookup covers 2^(7+7)=16384 values, so a good approximation
                self.size += 2;
                let val = entry.get_mut();
                val.usage += 1;
                val.index
            }
            Vacant(entry) => {
                self.size += 2 + value_len;
                entry.insert(StrInfo {
                    index: new_id,
                    usage: 1,
                });
                new_id
            }
        }
    }

    /// Convert self into a ready-to-serialize PBF StringTable
    /// The strings are sorted by usage count (desc) followed by string value (asc)
    /// Second result is a vector of indexes, mapping the index returned by [`add_string`]
    /// to the new position within the string table. 0-th element is always an empty string.
    pub fn finalize(self) -> (StringTable, Vec<usize>) {
        let mut items = self.strings.into_iter().collect::<Vec<_>>();
        items.sort_unstable_by(|a, b| match b.1.usage.cmp(&a.1.usage) {
            Ordering::Equal => a.0.cmp(&b.0),
            v => v,
        });
        let mut res = StringTable::default();
        let strings = items
            .iter()
            .map(|(v, _)| v.to_string().into_bytes())
            .collect();
        res.set_s(RepeatedField::from_vec(strings));
        let indexes: Vec<_> = items.into_iter().map(|v| v.1.index).collect();
        (res, indexes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_str(bytes: &[u8]) -> String {
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    fn assert_eq_vec(actual: &[Vec<u8>], expected: Vec<&str>) {
        let actual = actual.iter().map(|v| to_str(v)).collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_string_table_builder() {
        let tbl = StringTableBuilder::default();
        let (pbf, idx) = tbl.finalize();
        assert_eq!(pbf.get_s().len(), 0);
        assert_eq!(idx.len(), 0);

        let mut tbl = StringTableBuilder::default();
        assert_eq!(tbl.add_string("a".to_string()), 1);
        let (pbf, idx) = tbl.finalize();
        assert_eq_vec(pbf.get_s(), vec!["", "a"]);
        assert_eq!(idx, [0, 1]);

        let mut tbl = StringTableBuilder::default();
        assert_eq!(tbl.add_string("a".to_string()), 1);
        assert_eq!(tbl.add_string("b".to_string()), 2);
        assert_eq!(tbl.add_string("b".to_string()), 2);
        let (pbf, idx) = tbl.finalize();
        let pbf = pbf.get_s();
        assert_eq_vec(pbf, vec!["", "b", "a"]);
        assert_eq!(idx, [0, 2, 1]);
        assert_eq!(to_str(&pbf[idx[0]]), "");
        assert_eq!(to_str(&pbf[idx[1]]), "a");
        assert_eq!(to_str(&pbf[idx[2]]), "b");

        let mut tbl = StringTableBuilder::default();
        assert_eq!(tbl.add_string("a3".to_string()), 1);
        assert_eq!(tbl.add_string("a2".to_string()), 2);
        assert_eq!(tbl.add_string("a1".to_string()), 3);
        let (pbf, idx) = tbl.finalize();
        let pbf = pbf.get_s();
        assert_eq_vec(pbf, vec!["", "a1", "a2", "a3"]);
        assert_eq!(idx, [0, 3, 2, 1]);
        assert_eq!(to_str(&pbf[idx[0]]), "");
        assert_eq!(to_str(&pbf[idx[1]]), "a3");
        assert_eq!(to_str(&pbf[idx[2]]), "a2");
        assert_eq!(to_str(&pbf[idx[3]]), "a1");
    }
}
