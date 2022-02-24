/*!
A fast reader for the OpenStreetMap PBF file format (\*.osm.pbf).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
osmpbf = "0.2"
```

## Example: Count ways

Here's a simple example that counts all the OpenStreetMap way elements in a
file:

```rust
use osmpbf::{ElementReader, Element};

let reader = ElementReader::from_path("tests/test.osm.pbf")?;
let mut ways = 0_u64;

// Increment the counter by one for each way.
reader.for_each(|element| {
    if let Element::Way(_) = element {
        ways += 1;
    }
})?;

println!("Number of ways: {ways}");
# assert_eq!(ways, 1);
# Ok::<(), std::io::Error>(())
```

## Example: Count ways in parallel

In this second example, we also count the ways but make use of all cores by
decoding the file in parallel:

```rust
use osmpbf::{ElementReader, Element};

let reader = ElementReader::from_path("tests/test.osm.pbf")?;

// Count the ways
let ways = reader.par_map_reduce(
    |element| {
        match element {
            Element::Way(_) => 1,
            _ => 0,
        }
    },
    || 0_u64,      // Zero is the identity value for addition
    |a, b| a + b   // Sum the partial results
)?;

println!("Number of ways: {ways}");
# assert_eq!(ways, 1);
# Ok::<(), std::io::Error>(())
```
*/

#![recursion_limit = "1024"]

pub use blob::*;
pub use block::*;
pub use dense::*;
pub use elements::*;
pub use error::{BlobError, Error, ErrorKind, Result};
pub use indexed::*;
pub use mmap_blob::*;
pub use reader::*;
pub use write::*;

pub mod blob;
pub mod block;
pub mod dense;
pub mod elements;
mod error;
pub mod indexed;
pub mod mmap_blob;
mod proto;
pub mod reader;
pub mod write;

// pub mod proto {
//     include!(concat!(env!("OUT_DIR"), "/mod.rs"));
// }
