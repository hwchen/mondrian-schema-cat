#![recursion_limit = "1024"]
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



pub struct Fragment<'a> {
    schema_name: Option<&'a str>,
    shared_dims: Vec<&'a str>,
    cubes: Vec<&'a str>,
}

impl<'a> Fragment<'a> {

    /// Get the Schema name from one fragment
    /// None if there's no Schema tags
    /// Errors if there's multiple schema tags
    /// TODO error if there's multiple name attributes
    pub fn get_schema_name(fragment: &'a str) -> Result<Option<&'a str>> {
        Ok(Some(""))
    }

    // Get shared dims from one fragment
    pub fn get_shared_dims(fragment: &'a str) -> Result<Vec<&'a str>> {
        Ok(vec![])
    }

    // Get cubes from one fragment
    pub fn get_cubes(fragment: &'a str) -> Result<Vec<&'a str>> {
        Ok(vec![])
    }


    pub fn process_fragment(fragment: &'a str) -> Fragment<'a> {
    // Get Schema names from all fragments
    // and check for non-duplicates. There should only be one name.
    // Get shared Dim names from
        Fragment {
            schema_name: None,
            shared_dims: vec![],
            cubes: vec![],
        }
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
