#![recursion_limit = "1024"]
// A simple concatenation utility for fragments of
// Mondrian schemas.
//
// Takes an arbitrary number of schema fragments
// - schema (containing cubes and shared dims)
// - shared dims
// - cubes
//
// and then concatenates the fragements in the correct
// order (schema wraps shared dims and then cubes,
// in that order).
//
// Fragments can be any of the above three in any combination,
// but must be in the same order as a full schema.
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
const SCHEMA_TAG_OPEN: &str = r#"<Schema name=""#;
const SCHEMA_TAG_CLOSE: &str = r#"</Schema>"#;
const CUBE_TAG_OPEN: &str = "<Cube";
const DIM_TAG_OPEN: &str = "<Dimension";


/// Struct to hold the results of parsing
/// a string fragment of schema.
#[derive(Debug, PartialEq)]
pub struct Fragment<'a> {
    schema_name: Option<&'a str>,
    shared_dims: Option<&'a str>,
    cubes: Option<&'a str>,
}

impl<'a> Fragment<'a> {

    /// Get the Schema name from one fragment
    /// None if there's no Schema tags
    /// Takes first schema tag and first name attr
    fn get_schema_name(fragment: &'a str) -> Option<&'a str> {
        fragment
            .find(SCHEMA_TAG_OPEN)
            .map(|i| i + SCHEMA_TAG_OPEN.len())
            .and_then(|i| {
                fragment[i..]
                    .find('\"')
                    .and_then(|j| {
                        fragment.get(i..i+j)
                    })
            })
    }

    /// Get shared dims from one fragment
    fn get_shared_dims(fragment: &'a str) -> Option<&'a str> {
        // Return everything between first Dim tag and first
        // Cube tag.
        //
        // To make sure that don't get internal dims which
        // happen to have a Cube tag after, need to make
        // sure that there isn't a Cube tag before the
        // dim tag.

        if let Some(first_cube_idx) = fragment.find(CUBE_TAG_OPEN) {
            // if there's a cube tag, get from dim_open_tag to cube tag,
            // as long as it's not an internal dim.
            fragment
                .find(DIM_TAG_OPEN)
                .and_then(|i| {
                    fragment[i..]
                        .find(CUBE_TAG_OPEN)
                        .and_then(|j| {
                            println!("{}, {}, {}", i, j, first_cube_idx);
                            // if this cube tag idx is the
                            // first cube idx, then it means that
                            // the dims are shared, so get frag.
                            // Otherwise it's an internal  dim.
                            if i+j == first_cube_idx {
                                fragment.get(i..i+j)
                            } else {
                                None
                            }
                        })
                })
        } else {
            // if there's no cube tag, then get from dim_open_tag to end
            fragment.find(DIM_TAG_OPEN)
                .and_then(|i| {
                    fragment[i..]
                        .find(SCHEMA_TAG_CLOSE).or(Some(fragment.len()-i))
                        .and_then(|j| {
                            fragment.get(i..i+j)
                        })
                })
        }
    }

    // Get cubes from one fragment
    fn get_cubes(fragment: &'a str) -> Option<&'a str> {
        fragment.find(CUBE_TAG_OPEN)
            .and_then(|i| {
                fragment[i..]
                    .find(SCHEMA_TAG_CLOSE).or(Some(fragment.len()-i))
                    .and_then(|j| {
                        fragment.get(i..i+j)
                    })
            })
    }

    pub fn process_fragment(fragment: &'a str) -> Fragment<'a> {
        // TODO make this work with string parse fn?

        let schema_name = Fragment::get_schema_name(fragment);
        let shared_dims = Fragment::get_shared_dims(fragment);
        let cubes = Fragment::get_cubes(fragment);

        Fragment {
            schema_name: schema_name,
            shared_dims: shared_dims,
            cubes: cubes,
        }
    }
}

// Get Schema names from all fragments
// and check for non-duplicates (there should only
// be one schema name).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_schema_name() {
        let fragment = r#"<Schema name="testname"></Schema>"#;
        assert_eq!(Fragment::get_schema_name(fragment), Some("testname"));
        let fragment = r#"<Cube name="testname"></Cube>"#;
        assert_eq!(Fragment::get_schema_name(fragment), None);
    }

    #[test]
    fn test_get_share_dims() {
        let fragment = r#"<Schema name="testname">
            <Cube name="testcube"></Cube></Schema>"#;
        assert_eq!(Fragment::get_shared_dims(fragment), None);

        // gets shared dims before cubes
        let fragment = r#"<Schema name="testname">
            <Dimension></Dimension><Cube name="testcube"></Cube></Schema>"#;
        assert_eq!(
            Fragment::get_shared_dims(fragment),
            Some("<Dimension></Dimension>")
        );

        // does not get internal dims within cube
        // (this test has an extra Cube to make sure
        // that adding a Cube tag match after the dim
        // in this case doesn't trigger getting the
        // intermal dim
        let fragment = r#"<Schema name="testname">
            <Cube name="testcube"><Dimension></Dimension></Cube>
            <Cube name="a"></Cube>
            </Schema>"#;
        assert_eq!(
            Fragment::get_shared_dims(fragment),
            None
        );

        // Test only shared dims, both with and without schema tag
        let fragment = r#"<Schema name="test">
            <Dimension name="a"></Dimension></Schema>"#;
        assert_eq!(
            Fragment::get_shared_dims(fragment),
            Some(r#"<Dimension name="a"></Dimension>"#)
        );

        let fragment = r#"<Dimension name="a"></Dimension>"#;
        assert_eq!(
            Fragment::get_shared_dims(fragment),
            Some(r#"<Dimension name="a"></Dimension>"#)
        );
    }

    #[test]
    fn test_get_cubes() {
        let fragment = r#"<Cube name="a"></Cube>"#;
        assert_eq!(
            Fragment::get_cubes(fragment),
            Some(r#"<Cube name="a"></Cube>"#)
        );

        let fragment = r#"<Schema name="b"><Cube name="a"></Cube></Schema>"#;
        assert_eq!(
            Fragment::get_cubes(fragment),
            Some(r#"<Cube name="a"></Cube>"#)
        );
    }

    #[test]
    fn test_process_fragment() {
        let fragment = r#"<Schema name="testname">
            <Dimension name="shareddim"></Dimension><Cube name="testcube"><Dimension name="inner"></Dimension></Cube><Cube name="a"></Cube></Schema>"#;
        assert_eq!(
            Fragment::process_fragment(fragment),
            Fragment {
                schema_name: Some("testname"),
                shared_dims: Some(r#"<Dimension name="shareddim"></Dimension>"#),
                cubes: Some(r#"<Cube name="testcube"><Dimension name="inner"></Dimension></Cube><Cube name="a"></Cube>"#),
            }
        );
    }
}
