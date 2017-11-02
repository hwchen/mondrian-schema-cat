#[recursion_limit = "1024"]
// A simple concatenation utility for fragments of
// Mondrian schemas.
//
// Takes an arbitrary number of schema fragments
// - schema (containing cubes and shared dims)
// - cube
// - share dim
//
// and then concatenates the fragements in the correct
// order (schema wraps shared dims and then cubes,
// in that order).
//
// The simple implementation (first pass) is a simple text
// processor, with no deep knowledge of the schema.
// This means that there's no checking for dupliates or
// correctness and no rewriting of anything internal to
// a cube or shared dim (it just ignore schema tags
// basically).
//
// (Update: nevermind, I can still check for duplicate
// names at least, even with simple processing)
//
// For a given cube or shared dim, it will simply pull out
// the tags and anything between the tags.
//
// In the future, there might be deeper inspection, but
// probably not worthwhile since the concatenated file
// can easily be checked by an actual Mondrian instance.

#[macro_use]
extern crate error_chain;

mod error;

use error::*;

// I assume tags follow the convention of CamelCase
const SCHEMA_TAG_OPEN: &str = "<Schema";
const SCHEMA_TAG_CLOSE: &str = "</Schema";
const CUBE_TAG_OPEN: &str = "<Cube";
const CUBE_TAG_CLOSE: &str = "</Cube";
const DIM_TAG_OPEN: &str = "<Dimension";
const DIM_TAG_CLOSE: &str = "</Dimension";

// Get the Schema name from one fragment
pub fn get_schema_name(schema: &str) -> Result<&str> {
    Ok("")
}

// Get Schema names from all fragments
// and check for duplicates

// Get shared dims from one
pub fn get_shared_dims() {}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
