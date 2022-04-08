#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use osmpbf::*;
use std::fs;
use std::fs::File;
use std::io::BufWriter;

// #[test]
// fn write_blobs() {
//     let path = "tests/write_blobs.osm.pbf";
//     {
//         let f = File::create(path).unwrap();
//         let buf_writer = BufWriter::new(f);
//         let writer = BlobWriter::new(buf_writer);
//         writer.write_blob_header()
//     }
//     fs::remove_file(path).unwrap();
// }
