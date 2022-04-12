// #![allow(dead_code)]
// #![allow(unused_imports)]
// #![allow(unused_mut)]
// #![allow(unused_variables)]

mod common;

use crate::common::{assert_bytes, to_header, ExpBlob, ExpGroup, ExpNode, ExpWay};
use osmpbf::bbox::Bbox;
use osmpbf::write_block::PrimitiveBlock;
use osmpbf::HeaderBlock;

#[test]
fn headers() {
    let header = HeaderBlock::default();
    assert_bytes(header.finalize().as_slice(), &[to_header([], [], None)]);

    let mut header = HeaderBlock::default();
    header.set_required_features(vec!["FOO".to_string()]);
    header.set_optional_features(vec!["BAR".to_string(), "BAT".to_string()]);
    assert_bytes(
        header.finalize().as_slice(),
        &[to_header(["FOO"], ["BAR", "BAT"], None)],
    );

    let mut header = HeaderBlock::default();
    header.set_required_features(vec!["FOO".to_string(), "BAR".to_string()]);
    let bbox = Some(Bbox {
        left: -10,
        right: 20,
        top: 30,
        bottom: -40,
    });
    header.set_bbox(bbox.clone());
    assert_bytes(
        header.finalize().as_slice(),
        &[to_header(["FOO", "BAR"], [], bbox)],
    );
}

#[test]
fn empty_header() {
    assert_bytes(
        HeaderBlock::default().finalize().as_slice(),
        &[to_header([], [], None)],
    );
}

#[test]
fn one_way() {
    let mut block = PrimitiveBlock::default();
    block.add_way(
        Some(42),
        vec![1, 3, 5],
        vec![
            ("abc".to_string(), "def".to_string()),
            ("abc".to_string(), "xyz".to_string()),
        ],
        vec![100, 200, 300],
        vec![400, 500, 600],
    );
    assert_bytes(
        block.finalize().as_slice(),
        &[ExpBlob::Data(vec![{
            ExpGroup {
                nodes: vec![],
                d_nodes: vec![],
                ways: vec![ExpWay {
                    id: 42,
                    refs: vec![1, 3, 5],
                    locs: vec![(100, 400), (200, 500), (300, 600)],
                    tags: vec![
                        ("abc".to_string(), "def".to_string()),
                        ("abc".to_string(), "xyz".to_string()),
                    ],
                }],
                rels: vec![],
            }
        }])],
    );
}

#[test]
fn several_ways() {
    let mut block = PrimitiveBlock::default();
    block.add_way(
        Some(42),
        vec![1, 5],
        vec![
            ("abc".to_string(), "def".to_string()),
            ("abc".to_string(), "xyz".to_string()),
        ],
        vec![100, 300],
        vec![400, 600],
    );
    block.add_way(
        Some(69),
        vec![1],
        vec![("abc".to_string(), "def".to_string())],
        vec![],
        vec![],
    );
    assert_bytes(
        block.finalize().as_slice(),
        &[ExpBlob::Data(vec![{
            ExpGroup {
                nodes: vec![],
                d_nodes: vec![],
                ways: vec![
                    ExpWay {
                        id: 42,
                        refs: vec![1, 5],
                        locs: vec![(100, 400), (300, 600)],
                        tags: vec![
                            ("abc".to_string(), "def".to_string()),
                            ("abc".to_string(), "xyz".to_string()),
                        ],
                    },
                    ExpWay {
                        id: 69,
                        refs: vec![1],
                        locs: vec![],
                        tags: vec![("abc".to_string(), "def".to_string())],
                    },
                ],
                rels: vec![],
            }
        }])],
    );
}

#[test]
fn one_node() {
    let mut block = PrimitiveBlock::default();
    block.add_node(
        Some(42),
        vec![
            ("abc".to_string(), "def".to_string()),
            ("abc".to_string(), "xyz".to_string()),
        ],
        100200,
        300400,
    );
    assert_bytes(
        block.finalize().as_slice(),
        &[ExpBlob::Data(vec![{
            ExpGroup {
                nodes: vec![ExpNode {
                    id: 42,
                    loc: (100200, 300400),
                    tags: vec![
                        ("abc".to_string(), "def".to_string()),
                        ("abc".to_string(), "xyz".to_string()),
                    ],
                }],
                d_nodes: vec![],
                ways: vec![],
                rels: vec![],
            }
        }])],
    );
}
