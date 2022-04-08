mod common;
use common::*;
use osmpbf::RelMemberType::*;

#[test]
fn deleted_nodes() {
    assert_file_content(
        "tests/deleted_nodes.osh.pbf",
        vec![
            ExpBlob::Header(ExpHeader {
                req: vec![REQ_SCHEMA_V6, REQ_DENSE_NODES, REQ_HIST_INFO],
                opt: vec![],
                bbox: None,
            }),
            ExpBlob::Data(vec![ExpGroup {
                nodes: vec![],
                d_nodes: vec![
                    ExpNode {
                        id: 1,
                        loc: (214748364700, 214748364700),
                        tags: vec![],
                    },
                    ExpNode {
                        id: 2,
                        loc: (1000000000, 1000000000),
                        tags: vec![],
                    },
                ],
                ways: vec![],
                rels: vec![],
            }]),
        ],
    )
}

#[test]
fn loc_on_ways() {
    assert_file_content(
        "tests/loc_on_ways.osm.pbf",
        vec![
            ExpBlob::Header(ExpHeader {
                req: vec![REQ_SCHEMA_V6, REQ_DENSE_NODES],
                opt: vec![OPT_LOC_ON_WAYS],
                bbox: None,
            }),
            ExpBlob::Data(vec![ExpGroup {
                nodes: vec![],
                d_nodes: vec![],
                ways: vec![ExpWay {
                    id: 107,
                    refs: vec![105, 106, 108, 105],
                    locs: vec![
                        (52122403100, 11628401700),
                        (52119923500, 11625644600),
                        (52119899100, 11631019200),
                        (52122403100, 11628401700),
                    ],
                    tags: to_tags(vec![("building", "yes"), ("name", "triangle")]),
                }],
                rels: vec![],
            }]),
            ExpBlob::Data(vec![ExpGroup {
                nodes: vec![],
                d_nodes: vec![],
                ways: vec![],
                rels: vec![ExpRel {
                    id: 120,
                    members: vec![(107, Way, "test_role".to_string())],
                    tags: to_tags(vec![("rel_key", "rel_value")]),
                }],
            }]),
        ],
    )
}

#[test]
fn test() {
    assert_file_content(
        "tests/test.osm.pbf",
        vec![
            ExpBlob::Header(ExpHeader {
                req: vec![REQ_SCHEMA_V6, REQ_DENSE_NODES],
                opt: vec![],
                bbox: None,
            }),
            ExpBlob::Data(vec![
                ExpGroup {
                    nodes: vec![],
                    d_nodes: vec![
                        ExpNode {
                            id: 105,
                            loc: (52122403100, 11628401700),
                            tags: vec![],
                        },
                        ExpNode {
                            id: 106,
                            loc: (52119923500, 11625644600),
                            tags: vec![],
                        },
                        ExpNode {
                            id: 108,
                            loc: (52119899100, 11631019200),
                            tags: vec![],
                        },
                    ],
                    ways: vec![],
                    rels: vec![],
                },
                ExpGroup {
                    nodes: vec![],
                    d_nodes: vec![],
                    ways: vec![ExpWay {
                        id: 107,
                        refs: vec![105, 106, 108, 105],
                        locs: vec![],
                        tags: to_tags(vec![("building", "yes"), ("name", "triangle")]),
                    }],
                    rels: vec![],
                },
                ExpGroup {
                    nodes: vec![],
                    d_nodes: vec![],
                    ways: vec![],
                    rels: vec![ExpRel {
                        id: 120,
                        members: vec![(107, Way, "test_role".to_string())],
                        tags: vec![("rel_key".to_string(), "rel_value".to_string())],
                    }],
                },
            ]),
        ],
    )
}

#[test]
fn test_nozlib() {
    assert_file_content(
        "tests/test_nozlib.osm.pbf",
        vec![
            ExpBlob::Header(ExpHeader {
                req: vec![REQ_SCHEMA_V6, REQ_DENSE_NODES],
                opt: vec![],
                bbox: None,
            }),
            ExpBlob::Data(vec![
                ExpGroup {
                    nodes: vec![],
                    d_nodes: vec![
                        ExpNode {
                            id: 105,
                            loc: (52122403100, 11628401700),
                            tags: vec![],
                        },
                        ExpNode {
                            id: 106,
                            loc: (52119923500, 11625644600),
                            tags: vec![],
                        },
                        ExpNode {
                            id: 108,
                            loc: (52119899100, 11631019200),
                            tags: vec![],
                        },
                    ],
                    ways: vec![],
                    rels: vec![],
                },
                ExpGroup {
                    nodes: vec![],
                    d_nodes: vec![],
                    ways: vec![ExpWay {
                        id: 107,
                        refs: vec![105, 106, 108, 105],
                        locs: vec![],
                        tags: to_tags(vec![("building", "yes"), ("name", "triangle")]),
                    }],
                    rels: vec![],
                },
                ExpGroup {
                    nodes: vec![],
                    d_nodes: vec![],
                    ways: vec![],
                    rels: vec![ExpRel {
                        id: 120,
                        members: vec![(107, Way, "test_role".to_string())],
                        tags: to_tags(vec![("rel_key", "rel_value")]),
                    }],
                },
            ]),
        ],
    )
}

#[test]
fn test_nozlib_nodense() {
    assert_file_content(
        "tests/test_nozlib_nodense.osm.pbf",
        vec![
            ExpBlob::Header(ExpHeader {
                req: vec![REQ_SCHEMA_V6],
                opt: vec![],
                bbox: None,
            }),
            ExpBlob::Data(vec![
                ExpGroup {
                    nodes: vec![
                        ExpNode {
                            id: 105,
                            loc: (52122403100, 11628401700),
                            tags: vec![],
                        },
                        ExpNode {
                            id: 106,
                            loc: (52119923500, 11625644600),
                            tags: vec![],
                        },
                        ExpNode {
                            id: 108,
                            loc: (52119899100, 11631019200),
                            tags: vec![],
                        },
                    ],
                    d_nodes: vec![],
                    ways: vec![],
                    rels: vec![],
                },
                ExpGroup {
                    nodes: vec![],
                    d_nodes: vec![],
                    ways: vec![ExpWay {
                        id: 107,
                        refs: vec![105, 106, 108, 105],
                        locs: vec![],
                        tags: to_tags(vec![("building", "yes"), ("name", "triangle")]),
                    }],
                    rels: vec![],
                },
                ExpGroup {
                    nodes: vec![],
                    d_nodes: vec![],
                    ways: vec![],
                    rels: vec![ExpRel {
                        id: 120,
                        members: vec![(107, Way, "test_role".to_string())],
                        tags: to_tags(vec![("rel_key", "rel_value")]),
                    }],
                },
            ]),
        ],
    )
}
