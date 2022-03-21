#![allow(dead_code)]
#![allow(unused_imports)]

use std::convert::TryInto;
use byteorder::{BigEndian, WriteBytesExt};
use protobuf::{Message, RepeatedField};
use crate::BlobHeader;
use crate::proto::fileformat::Blob;
use crate::proto::{fileformat, osmformat};
use crate::proto::osmformat::{PrimitiveGroup, Way};
use crate::write::bbox::Bbox;
use crate::write::strings::StringTableBuilder;

fn create_blob(raw_data: Vec<u8>, typ: &str) -> Vec<u8> {
    let mut blob = fileformat::Blob::default();
    blob.set_raw_size(raw_data.len() as i32);
    blob.set_raw(raw_data);
    // let data = blob.write_to_bytes().unwrap();

    let mut header = fileformat::BlobHeader::default();
    header.set_field_type(String::from(typ));
    header.set_datasize(blob.compute_size().try_into().unwrap());
    // header.set_datasize(data.len() as i32);   //# 49

    let mut result: Vec<u8> = Vec::new();
    result.write_i32::<BigEndian>(header.compute_size().try_into().unwrap());
    header.write_to_vec(&mut result);
    blob.write_to_vec(&mut result);
    result
}

#[derive(Debug)]
pub struct PrimitiveBlock {
    max_elems: usize,

    block: osmformat::PrimitiveBlock,
    bbox: Bbox,
    strings: StringTableBuilder,
    granularity: Option<i32>,
    date_granularity: Option<i32>,
    lat_offset: Option<i64>,
    lon_offset: Option<i64>,
    nodes: Vec<PrimitiveGroup>,
    ways: Vec<PrimitiveGroup>,
    rels: Vec<PrimitiveGroup>,
}

#[derive(Debug, Default)]
pub struct HeaderBlock {
    block: osmformat::HeaderBlock,
}

impl HeaderBlock {
    pub fn as_mut(&mut self) -> &mut osmformat::HeaderBlock {
        &mut self.block
    }

    pub fn finalize(mut self) -> Vec<u8> {
        let raw_data = self.block.write_to_bytes().unwrap();
        create_blob(raw_data, "OSMHeader")
    }
}

impl Default for PrimitiveBlock {
    fn default() -> Self {
        Self {
            max_elems: 8_000,
            block: Default::default(),
            bbox: Default::default(),
            strings: Default::default(),
            granularity: None,
            date_granularity: None,
            lat_offset: None,
            lon_offset: None,
            nodes: vec![],
            ways: vec![],
            rels: vec![],
        }
    }
}

impl PrimitiveBlock {
    pub fn new_with_opts(
        max_elems: Option<usize>,
        granularity: Option<i32>,
        date_granularity: Option<i32>,
    ) -> Self {
        let mut res = Self::default();
        if let Some(v) = max_elems {
            assert_ne!(v, 0);
            res.max_elems = v;
        }
        res.granularity = granularity;
        res.date_granularity = date_granularity;
        res
    }

    pub fn finalize(mut self) -> Vec<u8> {
        let (strings, map) = self.strings.finalize();
        self.block.set_stringtable(strings);
        for way_grp in self.ways.iter_mut() {
            for w in way_grp.ways.iter_mut() {
                for k in w.keys.iter_mut().chain(w.vals.iter_mut()) {
                    *k = map[*k as usize] as u32;
                }
            }
        }
        // self.nodes.iter().chain(self.ways.iter()).chain(self.rels().iter()).collect();
        let mut prim_grps = self.nodes;
        prim_grps.reserve(self.ways.len() + self.rels.len());
        prim_grps.extend(self.ways);
        prim_grps.extend(self.rels);
        self.block.set_primitivegroup(RepeatedField::from_vec(prim_grps));
        let raw_data = self.block.write_to_bytes().unwrap();
        create_blob(raw_data, "OSMData")
    }

    pub fn add_way(&mut self, value: Way) {
        // try to add new way to the last existing group of ways if it has room
        // otherwise create a new group and add to that
        let group = match self.ways.last_mut() {
            Some(v) if v.ways.len() < self.max_elems => v.mut_ways(),
            _ => {
                self.ways.push(PrimitiveGroup::default());
                self.ways.last_mut().unwrap().mut_ways()
            }
        };
        self.bbox.add_node_list(&value.lon, &value.lat);
        group.push(value);
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::io::Cursor;
    use crate::{BlobDecode, BlobReader, Element, ElementReader};
    use crate::proto::fileformat::Blob;
    use super::*;

    #[test]
    fn test_header() {
        let mut h = HeaderBlock::default();
        let mut pbf = h.finalize();
        let mut reader = BlobReader::new(Cursor::new(pbf));
        // let reader = ElementReader::new(Cursor::new(pbf));
        // reader.for_each(|_| assert!(false));
        for b in reader {
            match b.unwrap().decode().unwrap() {
                BlobDecode::OsmHeader(h) => {
                    assert_unordered("Req", &[], h.required_features());
                    assert_unordered("Opt", &[], h.optional_features());
                }
                _ => panic!()
            }
        }
    }

    #[test]
    fn test_ways() {
        let mut pbf = HeaderBlock::default().finalize();
        let mut w = PrimitiveBlock::default();
        let mut way = Way::default();
        way.set_id(42);
        way.set_refs(vec![1]);
        w.add_way(way);
        pbf.extend(w.finalize());

        let reader = ElementReader::new(Cursor::new(pbf));
        let mut count = 0;
        reader.for_each(|e| {
            count += 1;
            if let Element::Way(w) = e {
                assert_eq!(w.id(), 42);
            } else {
                assert!(false);
            }
        }).unwrap();
        assert_eq!(count, 1);
    }

    /// Assert that two arrays are the same, ignoring their order
    #[cfg(test)]
    pub(crate) fn assert_unordered(info: &str, expected: &[&str], actual: &[String]) {
        assert!(
            is_same_unordered(expected, actual),
            "{} features {:?} don't match expected {:?}",
            info,
            actual,
            expected
        );
    }

    /// Ensure two vectors have the same values, ignoring their order
    #[cfg(test)]
    pub(crate) fn is_same_unordered(a: &[&str], b: &[String]) -> bool {
        if a.len() == b.len() {
            let mut a = a.to_vec();
            let mut b = b.to_vec();
            a.sort_unstable();
            b.sort();
            a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count() == a.len()
        } else {
            false
        }
    }
}
