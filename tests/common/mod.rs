#![allow(dead_code)]

use osmpbf::bbox::Bbox;
use osmpbf::{BlobDecode, BlobReader, PrimitiveGroup, RelMemberType};
use std::io::Read;

pub struct ExpHeader {
    pub req: Vec<&'static str>,
    pub opt: Vec<&'static str>,
    pub bbox: Option<Bbox>,
}

pub struct ExpGroup {
    pub nodes: Vec<ExpNode>,
    pub d_nodes: Vec<ExpNode>,
    pub ways: Vec<ExpWay>,
    pub rels: Vec<ExpRel>,
}

#[derive(PartialEq, Debug)]
pub struct ExpNode {
    pub id: i64,
    pub loc: (i64, i64),
    pub tags: Vec<(String, String)>,
}

#[derive(PartialEq, Debug)]
pub struct ExpWay {
    pub id: i64,
    pub refs: Vec<i64>,
    pub locs: Vec<(i64, i64)>,
    pub tags: Vec<(String, String)>,
}

#[derive(PartialEq, Debug)]
pub struct ExpRel {
    pub id: i64,
    pub members: Vec<(i64, RelMemberType, String)>,
    pub tags: Vec<(String, String)>,
}

pub enum ExpBlob {
    Header(ExpHeader),
    Data(Vec<ExpGroup>),
}

pub fn assert_file_content(path: &str, expected_vals: &[ExpBlob]) {
    assert_content(BlobReader::from_path(path).unwrap(), expected_vals)
}

pub fn assert_bytes<R: Read + Send>(reader: R, expected_vals: &[ExpBlob]) {
    assert_content(BlobReader::new(reader), expected_vals)
}

pub fn assert_content<R: Read + Send>(reader: BlobReader<R>, expected_vals: &[ExpBlob]) {
    let mut expected_iter = expected_vals.into_iter();
    for (block_id, actual) in reader.into_iter().enumerate() {
        let expected = expected_iter
            .next()
            .unwrap_or_else(|| panic!("More blobs available than expected"));
        match actual.unwrap().decode().unwrap() {
            BlobDecode::OsmHeader(hdr) => {
                if let ExpBlob::Header(ExpHeader { req, opt, bbox }) = expected {
                    sort_compare("Required features", hdr.required_features(), &req);
                    sort_compare("Optional features", hdr.optional_features(), &opt);
                    assert_eq!(&hdr.bbox(), bbox, "Bounding box")
                } else {
                    panic!("Was expecting a header blob");
                }
            }
            BlobDecode::OsmData(data) => {
                if let ExpBlob::Data(groups) = expected {
                    let mut expected_groups = groups.into_iter();
                    for (group_id, group) in data.groups().enumerate() {
                        let expected_group = expected_groups.next().unwrap_or_else(|| {
                            panic!("More groups available than expected in block {}", block_id)
                        });
                        assert_group(group, expected_group, block_id, group_id);
                    }
                    assert!(
                        expected_groups.next().is_none(),
                        "Less groups available than expected in block {}",
                        block_id
                    );
                } else {
                    panic!("Was expecting a data blob");
                }
            }
            BlobDecode::Unknown(unk) => panic!("Unknown: {}", unk),
        }
    }
    assert!(
        expected_iter.next().is_none(),
        "Less blobs available than expected"
    );
}

fn assert_group(
    actual_group: PrimitiveGroup,
    expected: &ExpGroup,
    block_id: usize,
    group_id: usize,
) {
    let actual: Vec<_> = actual_group
        .nodes()
        .map(|v| ExpNode {
            id: v.id(),
            loc: (v.nano_lat(), v.nano_lon()),
            tags: to_tags(v.tags()),
        })
        .collect();
    assert_eq!(actual, expected.nodes, "nodes {}/{}", block_id, group_id);

    let actual: Vec<_> = actual_group
        .dense_nodes()
        .map(|v| ExpNode {
            id: v.id(),
            loc: (v.nano_lat(), v.nano_lon()),
            tags: to_tags(v.tags()),
        })
        .collect();
    assert_eq!(
        actual, expected.d_nodes,
        "dense nodes {}/{}",
        block_id, group_id
    );

    let actual: Vec<_> = actual_group
        .ways()
        .map(|v| ExpWay {
            id: v.id(),
            refs: v.refs().collect(),
            locs: v
                .node_locations()
                .map(|v| (v.nano_lat(), v.nano_lon()))
                .collect(),
            tags: to_tags(v.tags()),
        })
        .collect();
    assert_eq!(actual, expected.ways, "ways {}/{}", block_id, group_id);

    let actual: Vec<_> = actual_group
        .relations()
        .map(|v| ExpRel {
            id: v.id(),
            members: v
                .members()
                .map(|v| (v.member_id, v.member_type, v.role().unwrap().to_string()))
                .collect(),
            tags: to_tags(v.tags()),
        })
        .collect();
    assert_eq!(
        actual, expected.rels,
        "relations in block {}/{}",
        block_id, group_id
    );
}

fn sort_compare(name: &str, actual: &[String], expected: &[&str]) {
    assert!(
        is_same_unordered(actual, expected),
        "{} {:?} does not match expected {:?}",
        name,
        actual,
        expected
    );
}

pub fn to_tags<'t, TTags>(tags: TTags) -> Vec<(String, String)>
where
    TTags: IntoIterator<Item = (&'t str, &'t str)>,
{
    tags.into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

pub fn to_header<T1, T2>(req: T1, opt: T2, bbox: Option<Bbox>) -> ExpBlob
where
    T1: IntoIterator<Item = &'static str>,
    T2: IntoIterator<Item = &'static str>,
{
    ExpBlob::Header(ExpHeader {
        req: req.into_iter().collect(),
        opt: opt.into_iter().collect(),
        bbox: bbox,
    })
}

/// Ensure two vectors have the same values, ignoring their order
pub fn is_same_unordered(actual: &[String], expected: &[&str]) -> bool {
    if actual.len() == expected.len() {
        let mut a = actual.to_vec();
        let mut e = expected.to_vec();
        a.sort_unstable();
        e.sort_unstable();
        a.iter().zip(e.iter()).filter(|&(a, e)| a == e).count() == a.len()
    } else {
        false
    }
}
