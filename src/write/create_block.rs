use crate::proto::fileformat;
use byteorder::{BigEndian, WriteBytesExt};
use protobuf::Message;
use std::convert::TryInto;

pub(crate) fn create_blob(raw_data: Vec<u8>, typ: &str) -> Vec<u8> {
    let mut blob = fileformat::Blob::default();
    blob.set_raw_size(raw_data.len() as i32);
    blob.set_raw(raw_data);
    // let data = blob.write_to_bytes().unwrap();

    let mut header = fileformat::BlobHeader::default();
    header.set_field_type(String::from(typ));
    header.set_datasize(blob.compute_size().try_into().unwrap());
    // header.set_datasize(data.len() as i32);   //# 49

    let mut res: Vec<u8> = Vec::new();
    let size = header.compute_size().try_into().unwrap();
    res.write_i32::<BigEndian>(size).unwrap();
    header.write_to_vec(&mut res).unwrap();
    blob.write_to_vec(&mut res).unwrap();
    res
}
