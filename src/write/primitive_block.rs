use proto::osmformat;
use proto::osmformat::{ PrimitiveGroup, Way};
use write::bbox::Bbox;
use write::strings::StringTableBuilder;

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
    pub fn new_with_opts(max_elems: Option<usize>,
                         granularity: Option<i32>,
                         date_granularity: Option<i32>) -> Self {
        let mut res = Self::default();
        if let Some(v) = max_elems {
            assert_ne!(v, 0);
            res.max_elems = v;
        }
        res.granularity = granularity;
        res.date_granularity = date_granularity;
        res
    }

    pub fn add_way(&mut self, value: Way) {
        let group = match self.ways.last_mut() {
            Some(v) if v.ways.len() < self.max_elems => { v.mut_ways() }
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
    use super::*;

    #[test]
    fn test_ways() {
        let mut w = PrimitiveBlock::default();
        let mut way = Way::default();
        way.set_id(42);
        way.set_refs(vec![1]);
        way.set_keys(vec![2]);
        way.set_vals(vec![3]);
        w.add_way(way)
    }
}
