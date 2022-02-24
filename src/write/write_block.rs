#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use crate::create_block::create_blob;
use crate::proto::fileformat::Blob;
use crate::proto::osmformat::{Node, PrimitiveGroup, Relation, Relation_MemberType, Way};
use crate::proto::{fileformat, osmformat};
use crate::write::bbox::Bbox;
use crate::write::create_block;
use crate::write::strings::StringTableBuilder;
use crate::{elements, BlobHeader, DeltaEnc, DeltaEncoder, RelMemberType};
use byteorder::{BigEndian, WriteBytesExt};
use protobuf::{Message, RepeatedField};
use std::convert::TryInto;

#[derive(Debug, Clone, Copy, Default)]
pub struct PrimitiveBlockOptions {
    pub max_group_size: Option<usize>,
    pub granularity: Option<i32>,
    pub date_granularity: Option<i32>,
    pub lat_offset: Option<i64>,
    pub lon_offset: Option<i64>,
}

impl PrimitiveBlockOptions {
    pub fn create(&self) -> PrimitiveBlock {
        PrimitiveBlock::new(&self)
    }
}

#[derive(Debug)]
pub struct PrimitiveBlock {
    max_group_size: usize,

    block: osmformat::PrimitiveBlock,
    bbox: Bbox,
    strings: StringTableBuilder,
    granularity: i32,
    date_granularity: i32,
    lat_offset: i64,
    lon_offset: i64,
    nodes: Vec<PrimitiveGroup>,
    ways: Vec<PrimitiveGroup>,
    rels: Vec<PrimitiveGroup>,
}

impl Default for PrimitiveBlock {
    fn default() -> Self {
        let block = osmformat::PrimitiveBlock::default();
        let granularity = block.get_granularity();
        let date_granularity = block.get_date_granularity();
        let lat_offset = block.get_lat_offset();
        let lon_offset = block.get_lon_offset();
        Self {
            max_group_size: 8_000,
            block,
            bbox: Default::default(),
            strings: Default::default(),
            granularity,
            date_granularity,
            lat_offset,
            lon_offset,
            nodes: vec![],
            ways: vec![],
            rels: vec![],
        }
    }
}

impl PrimitiveBlock {
    fn new(opts: &PrimitiveBlockOptions) -> Self {
        let mut res = Self::default();
        if let Some(v) = opts.max_group_size {
            res.max_group_size = v;
        }
        if let Some(v) = opts.granularity {
            res.block.set_granularity(v);
        }
        if let Some(v) = opts.date_granularity {
            res.block.set_date_granularity(v);
        }
        if let Some(v) = opts.lat_offset {
            res.block.set_lat_offset(v);
        }
        if let Some(v) = opts.lon_offset {
            res.block.set_lon_offset(v);
        }
        res.granularity = res.block.get_granularity();
        res.date_granularity = res.block.get_date_granularity();
        res.lat_offset = res.block.get_lat_offset();
        res.lon_offset = res.block.get_lon_offset();
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
        let mut prim_grps = self.nodes;
        prim_grps.reserve(self.ways.len() + self.rels.len());
        prim_grps.extend(self.ways);
        prim_grps.extend(self.rels);
        self.block
            .set_primitivegroup(RepeatedField::from_vec(prim_grps));
        let raw_data = self.block.write_to_bytes().unwrap();
        create_blob(raw_data, "OSMData")
    }

    pub fn add_node<TTags>(&mut self, id: i64, tags: TTags, lat: i64, lon: i64)
    where
        TTags: IntoIterator<Item = (String, String)>,
    {
        self.bbox.add_node(lat, lon);
        let mut node = Node::default();
        node.set_id(id);

        let (keys, vals) = self.add_tags(tags);
        node.set_keys(keys);
        node.set_vals(vals);

        node.set_lat(self.encode_lat(lat));
        node.set_lon(self.encode_lon(lon));

        // try to add new node to the last existing group of nodes if it has room
        // otherwise create a new group and add to that
        let group = match self.nodes.last_mut() {
            Some(v) if v.nodes.len() < self.max_group_size => v.mut_nodes(),
            _ => {
                self.nodes.push(PrimitiveGroup::default());
                self.nodes.last_mut().unwrap().mut_nodes()
            }
        };
        group.push(node);
    }

    fn encode_lat(&self, lat: i64) -> i64 {
        (lat - self.lat_offset) / self.granularity as i64
    }

    fn encode_lon(&self, lon: i64) -> i64 {
        (lon - self.lon_offset) / self.granularity as i64
    }

    pub fn add_way<TTags>(
        &mut self,
        id: i64,
        refs: Vec<i64>,
        tags: TTags,
        mut lats: Vec<i64>,
        mut lons: Vec<i64>,
    ) where
        TTags: IntoIterator<Item = (String, String)>,
    {
        assert_eq!(
            lats.len(),
            lons.len(),
            "number of lats and lons must be the same"
        );
        assert!(
            lats.len() == 0 || refs.len() == 0 || lats.len() == refs.len(),
            "number of lats and lons or refs must be zero, or they must be the same"
        );
        self.bbox.add_node_list(&lats, &lons);
        let mut way = Way::default();
        way.set_id(id);

        let (keys, vals) = self.add_tags(tags);
        way.set_keys(keys);
        way.set_vals(vals);

        let vec = DeltaEncoder::encode(refs.iter());
        way.set_refs(vec);

        // FIXME: this seems ugly
        for i in 0..lats.len() {
            lats[i] = self.encode_lat(lats[i]);
        }
        for i in 0..lons.len() {
            lons[i] = self.encode_lon(lons[i]);
        }

        way.set_lat(DeltaEncoder::encode(lats.iter()));
        way.set_lon(DeltaEncoder::encode(lons.iter()));

        // try to add new way to the last existing group of ways if it has room
        // otherwise create a new group and add to that
        let group = match self.ways.last_mut() {
            Some(v) if v.ways.len() < self.max_group_size => v.mut_ways(),
            _ => {
                self.ways.push(PrimitiveGroup::default());
                self.ways.last_mut().unwrap().mut_ways()
            }
        };
        group.push(way);
    }

    pub fn add_relation<TTags, TMembers>(&mut self, id: i64, tags: TTags, members: TMembers)
    where
        TTags: IntoIterator<Item = (String, String)>,
        TMembers: IntoIterator<
            Item = (
                String,        // role
                i64,           // member_id - delta-encoded
                RelMemberType, // member_type
            ),
        >,
    {
        let mut rel = Relation::default();
        rel.set_id(id);

        let (keys, vals) = self.add_tags(tags);
        rel.set_keys(keys);
        rel.set_vals(vals);

        let mut enc = DeltaEnc::default();
        let iter = members.into_iter();
        let (lwr, _) = iter.size_hint();
        let mut role_sids = Vec::with_capacity(lwr);
        let mut member_ids = Vec::with_capacity(lwr);
        let mut member_types = Vec::with_capacity(lwr);
        for (role, member_id, member_type) in iter {
            role_sids.push(self.strings.add_string(role) as i32);
            member_ids.push(enc.encode(member_id));
            member_types.push(osmformat::Relation_MemberType::from(member_type));
        }
        rel.set_roles_sid(role_sids);
        rel.set_memids(member_ids);
        rel.set_types(member_types);

        // try to add new rel to the last existing group of rels if it has room
        // otherwise create a new group and add to that
        let group = match self.rels.last_mut() {
            Some(v) if v.relations.len() < self.max_group_size => v.mut_relations(),
            _ => {
                self.rels.push(PrimitiveGroup::default());
                self.rels.last_mut().unwrap().mut_relations()
            }
        };
        group.push(rel);
    }

    fn add_tags<TTags>(&mut self, tags: TTags) -> (Vec<u32>, Vec<u32>)
    where
        TTags: IntoIterator<Item = (String, String)>,
    {
        let tags = tags.into_iter();
        let (mut size_hint, _) = tags.size_hint();
        let mut keys = Vec::<u32>::with_capacity(size_hint);
        let mut vals = Vec::<u32>::with_capacity(size_hint);
        for (k, v) in tags {
            keys.push(self.strings.add_string(k) as u32);
            vals.push(self.strings.add_string(v) as u32);
        }
        (keys, vals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::fileformat::Blob;
    use crate::{BlobDecode, BlobReader, Element, ElementReader, HeaderBlock};
    use std::fs::read;
    use std::io::Cursor;

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
                _ => panic!(),
            }
        }
    }

    #[test]
    fn test_ways() {
        let mut pbf = HeaderBlock::default().finalize();
        let mut w = PrimitiveBlock::default();
        w.add_way(42, vec![1], vec![], vec![], vec![]);
        pbf.extend(w.finalize());

        let reader = ElementReader::new(Cursor::new(pbf));
        let mut count = 0;
        reader
            .for_each(|e| {
                count += 1;
                if let Element::Way(w) = e {
                    assert_eq!(w.id(), 42);
                } else {
                    panic!();
                }
            })
            .unwrap();
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
