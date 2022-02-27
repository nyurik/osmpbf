use protobuf::{Message, RepeatedField};
use crate::proto::osmformat;
use crate::proto::osmformat::{PrimitiveGroup, Way};
use crate::write::bbox::Bbox;
use crate::write::strings::StringTableBuilder;

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
        let blk = self.block.write_to_bytes();
        blk.unwrap()
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
    use std::io::Cursor;
    use crate::{Element, ElementReader};
    use super::*;

    #[test]
    fn test_ways() {
        let mut w = PrimitiveBlock::default();
        let mut way = Way::default();
        way.set_id(42);
        way.set_refs(vec![1]);
        // way.set_keys(vec![2]);
        // way.set_vals(vec![3]);
        w.add_way(way);
        let pbf = w.finalize();

        let reader = ElementReader::new(Cursor::new(pbf));
        let mut count = 0;
        reader.for_each(|e| {
            count += 1;
            assert_eq!(count, 1);
            if let Element::Way(w) = e {
                assert_eq!(w.id(), 42);
            } else {
                assert!(false);
            }
        }).unwrap();
        assert_eq!(count, 1);
    }
}
