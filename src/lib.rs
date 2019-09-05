#![recursion_limit = "1024"]
// Copyright 2018 mondrian-schema-cat Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.


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

pub mod error;
use error::*;

// I assume tags follow the convention of CamelCase
const SCHEMA_TAG_OPEN: &str = r#"<Schema name=""#;
const SCHEMA_TAG_CLOSE: &str = r#"</Schema>"#;
const CUBE_TAG_OPEN: &str = "<Cube";
const SHAREDDIM_TAG_OPEN: &str = "<SharedDimension";
const DIM_TAG_OPEN: &str = "<Dimension";
const VIRTUALCUBE_TAG_OPEN: &str = r#"<VirtualCube"#;


/// Struct to hold the results of parsing
/// a string fragment of schema.
#[derive(Debug, PartialEq)]
pub struct Fragment<'a> {
    schema_name: Option<&'a str>,
    shared_dims: Option<&'a str>,
    cubes: Option<&'a str>,
    virtual_cubes: Option<&'a str>,
}

impl<'a> Fragment<'a> {

    /// Get the Schema name from one fragment
    /// None if there's no Schema tags
    /// Takes first schema tag and first name attr
    fn get_schema_name(fragment: &'a str) -> Result<Option<&'a str>> {
        let res = fragment
            .find(SCHEMA_TAG_OPEN)
            .map(|i| i + SCHEMA_TAG_OPEN.len())
            .and_then(|i| {
                fragment[i..]
                    .find('\"')
                    .and_then(|j| {
                        fragment.get(i..i+j)
                    })
            });
        Ok(res)
    }

    /// Get shared dims from one fragment
    fn get_shared_dims(fragment: &'a str) -> Result<Option<&'a str>> {
        // Finds the location of the first encount of the tag SharedDimension
        // If the first occurence is after the cube/ virtualcube will return an error
        let res;
        if let Some(cube_index) = fragment.find(SHAREDDIM_TAG_OPEN) {
            res = fragment
                .find(SHAREDDIM_TAG_OPEN)
                .and_then(|i| {
                    fragment[i..]
                        .find(CUBE_TAG_OPEN)
                        .or_else(|| fragment[i..].find(VIRTUALCUBE_TAG_OPEN))
                        .or_else(|| fragment[i..].find(SCHEMA_TAG_CLOSE))
                        .or(Some(fragment.len()-i))
                        .and_then(|j|{
                            match fragment[..j].find(CUBE_TAG_OPEN).or_else(|| fragment[..j].find(VIRTUALCUBE_TAG_OPEN)){
                                Some(_) =>{
                                    Some("-11")  // Falg used for Raising an error if the sahred dimension is defined between the cubes or at the end of the cubes
                                }
                                None => {
                                    fragment.get(i..i+j)
                                }
                            }
                        })
                });
        } else {
            res = fragment
                .find(CUBE_TAG_OPEN)
                .or_else(|| fragment.find(VIRTUALCUBE_TAG_OPEN))
                .or_else(|| fragment.find(SCHEMA_TAG_CLOSE))
                .or(Some(fragment.len()))
                .and_then(|i| {
                    fragment[..i]
                        .find(DIM_TAG_OPEN)
                        .and_then(|j|{
                            fragment.get(j..i)
                        })
                });
        }
        if res != Some("-11"){
            Ok(res)
        } else {
            return Err("Shared Dimension is in the wrong place".into())  // if the flag value is raised we generate an error in the program
        }
    }

    // Get cubes from one fragment
    fn get_cubes(fragment: &'a str) -> Result<Option<&'a str>> {
        // println!("{}", fragment.find(CUBE_TAG_CLOSE).unwrap());
        let res = fragment.find(CUBE_TAG_OPEN)
            .and_then(|i| {
                fragment[i..]
                    .find(VIRTUALCUBE_TAG_OPEN)
                    .or_else(|| fragment[i..].find(SCHEMA_TAG_CLOSE))
                    .or(Some(fragment.len()-i)) // eof
                    .and_then(|j| {
                        fragment.get(i..i+j)
                    })
            });
        Ok(res)
    }

    // Get virtual cubes from one fragment
    fn get_virtual_cubes(fragment: &'a str) -> Result<Option<&'a str>> {
        let res = fragment.find(VIRTUALCUBE_TAG_OPEN)
            .and_then(|i| {
                fragment[i..]
                    .find(SCHEMA_TAG_CLOSE)
                    .or(Some(fragment.len()-i)) // eof
                    .and_then(|j| {
                        fragment.get(i..i+j)
                    })
            });
        Ok(res)
    }

    pub fn process_fragment(fragment: &'a str) -> Result<Fragment<'a>> {
        // TODO make this work with string parse fn?

        let schema_name = Fragment::get_schema_name(fragment)?;
        let shared_dims = Fragment::get_shared_dims(fragment)?;
        let cubes = Fragment::get_cubes(fragment)?;
        let virtual_cubes = Fragment::get_virtual_cubes(fragment)?;
        Ok(Fragment {
            schema_name: schema_name,
            shared_dims: shared_dims,
            cubes: cubes,
            virtual_cubes: virtual_cubes,
        })
    }
}

/// Convenience method for turning unprocessed fragments
/// into one schema
pub fn fragments_to_schema(fragment: &[String]) -> Result<String> {
    // Get Schema names from all fragments
    // and check for non-duplicates (there should only
    // be one schema name). Error is returned if
    // there's more than one schema name
    //
    // Otherwise, process all fragments, then iterate through
    // 2 passes to first push all shared dims, then
    // to push all cubes.

    // process fragments
    let fragments: Vec<_>;
    match fragment.iter().map(|s| Fragment::process_fragment(&s)).collect() {
        Ok(f) => fragments = f,
        Err(e) => return Err(e)
    }

    // schema name handling
    let mut schema_name = None;
    for frag in &fragments {
        if let Some(current_name) = frag.schema_name {
            if let Some(stored_name) = schema_name {
                if stored_name != current_name {
                    return Err("More than one schema name found".into());
                }
            } else {
                schema_name = Some(current_name);
            }
        } else {
            continue
        }
    }

    // now push onto final str
    let mut final_schema = String::new();
    final_schema.push_str("<Schema name=\"");
    if let Some(name) = schema_name {
        final_schema.push_str(name);
        final_schema.push_str("\">\n");
    } else {
        return Err("No schema name found".into());
    }

    for frag in &fragments {
        if let Some(shared_dims) = frag.shared_dims {
            final_schema.push_str(shared_dims);
        }
    }
    for frag in &fragments {
        if let Some(cubes) = frag.cubes {
            final_schema.push_str(cubes);
        }
    }
    for frag in &fragments {
        if let Some(virtual_cubes) = frag.virtual_cubes {
            final_schema.push_str(virtual_cubes);
        }
    }

    final_schema.push_str("\n</Schema>");
    println!("{:?}", fragments[0]);

    Ok(final_schema)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_schema_name() {
        let fragment = r#"<Schema name="testname"></Schema>"#;
        assert_eq!(Fragment::get_schema_name(fragment).unwrap(), Some("testname"));
        let fragment = r#"<Cube name="testname"></Cube>"#;
        assert_eq!(Fragment::get_schema_name(fragment).unwrap(), None);
    }

    #[test]
    fn test_get_share_dims() {
        let fragment = r#"<Schema name="testname">
            <Cube name="testcube"></Cube></Schema>"#;
        assert_eq!(Fragment::get_shared_dims(fragment).unwrap(), None);

        // gets shareddims tag and dims tag  before cubes
        let fragment = r#"<Schema name="testname">
            <SharedDimension></SharedDimension><Cube name="testcube"></Cube></Schema>"#;
        assert_eq!(
            Fragment::get_shared_dims(fragment).unwrap(),
            Some("<SharedDimension></SharedDimension>")
        );

        let fragment = r#"<Schema name="testname">
            <Dimension></Dimension><Cube name="testcube"></Cube></Schema>"#;
        assert_eq!(
            Fragment::get_shared_dims(fragment).unwrap(),
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
            Fragment::get_shared_dims(fragment).unwrap(),
            None
        );

        // Test only shared dims, both with and without schema tag
        let fragment = r#"<Schema name="test">
            <Dimension name="a"></Dimension></Schema>"#;
        assert_eq!(
            Fragment::get_shared_dims(fragment).unwrap(),
            Some(r#"<Dimension name="a"></Dimension>"#)
        );

        let fragment = r#"<Schema name="test">
            <SharedDimension name="a"></SharedDimension></Schema>"#;
        assert_eq!(
            Fragment::get_shared_dims(fragment).unwrap(),
            Some(r#"<SharedDimension name="a"></SharedDimension>"#)
        );

        let fragment = r#"<SharedDimension name="a"></SharedDimension>"#;
        assert_eq!(
            Fragment::get_shared_dims(fragment).unwrap(),
            Some(r#"<SharedDimension name="a"></SharedDimension>"#)
        );

        let fragment = r#"<Dimension name="a"></Dimension>"#;
        assert_eq!(
            Fragment::get_shared_dims(fragment).unwrap(),
            Some(r#"<Dimension name="a"></Dimension>"#)
        );
    }

    #[test]
    fn test_get_cubes() {
        let fragment = r#"<Cube name="a"></Cube><VirtualCube name="vc1"></VirtualCube>"#;
        assert_eq!(
            Fragment::get_cubes(fragment).unwrap(),
            Some(r#"<Cube name="a"></Cube>"#)
        );

        let fragment = r#"<Schema name="b"><Cube name="a"></Cube></Schema>"#;
        assert_eq!(
            Fragment::get_cubes(fragment).unwrap(),
            Some(r#"<Cube name="a"></Cube>"#)
        );
    }

    #[test]
    fn test_get_virtual_cubes() {
        let fragment = r#"<Cube name="a"></Cube><VirtualCube name="vc1"></VirtualCube>"#;
        assert_eq!(
            Fragment::get_virtual_cubes(fragment).unwrap(),
            Some(r#"<VirtualCube name="vc1"></VirtualCube>"#)
        );

        let fragment = r#"<Schema name="s1"><VirtualCube name="vc1"></VirtualCube></Schema>"#;
        assert_eq!(
            Fragment::get_virtual_cubes(fragment).unwrap(),
            Some(r#"<VirtualCube name="vc1"></VirtualCube>"#)
        );
    }

    #[test]
    fn test_process_fragment() {
        let fragment = r#"<Schema name="testname">
            <Dimension name="shareddim"></Dimension><Cube name="testcube"><Dimension name="inner"></Dimension></Cube><Cube name="a"></Cube><VirtualCube name="testvirtualcube"><Dimension name="inner_virtual"></Dimension></VirtualCube><VirtualCube name="a"></VirtualCube></Schema>"#;
        assert_eq!(
            Fragment::process_fragment(fragment).unwrap(),
            Fragment {
                schema_name: Some("testname"),
                shared_dims: Some(r#"<Dimension name="shareddim"></Dimension>"#),
                cubes: Some(r#"<Cube name="testcube"><Dimension name="inner"></Dimension></Cube><Cube name="a"></Cube>"#),
                virtual_cubes: Some(r#"<VirtualCube name="testvirtualcube"><Dimension name="inner_virtual"></Dimension></VirtualCube><VirtualCube name="a"></VirtualCube>"#),
            }
        );
    }

    #[test]
    fn test_process_fragment_shareddimension() {
        let fragment = r#"<Schema name="testname">
            <SharedDimension name="shareddim"></SharedDimension><Cube name="testcube"><Dimension name="inner"></Dimension></Cube><Cube name="a"></Cube><VirtualCube name="testvirtualcube"><Dimension name="inner_virtual"></Dimension></VirtualCube><VirtualCube name="a"></VirtualCube></Schema>"#;
        assert_eq!(
            Fragment::process_fragment(fragment).unwrap(),
            Fragment {
                schema_name: Some("testname"),
                shared_dims: Some(r#"<SharedDimension name="shareddim"></SharedDimension>"#),
                cubes: Some(r#"<Cube name="testcube"><Dimension name="inner"></Dimension></Cube><Cube name="a"></Cube>"#),
                virtual_cubes: Some(r#"<VirtualCube name="testvirtualcube"><Dimension name="inner_virtual"></Dimension></VirtualCube><VirtualCube name="a"></VirtualCube>"#),
            }
        );
    }

    #[test]
    #[should_panic]
    fn test_fragments_to_schema_empty() {
        fragments_to_schema(&vec!["".to_owned()]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fragments_to_schema_no_schema() {
        fragments_to_schema(&vec!["<Cube></Cube>".to_owned()]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fragments_to_schema_different_names() {
        fragments_to_schema(&vec!["<Schema name=\"a\"></Schema>".to_owned(), "<Schema name=\"b\"></Schema>".to_owned()]).unwrap();
    }

    #[test]
    fn test_fragments_to_schema() {
        // First make sure that feeding through just one works
        let fragment = r#"<Schema name="testname"><SharedDimension name="shareddim"></SharedDimension><Cube name="testcube"><Dimension name="inner"></Dimension></Cube><Cube name="a"></Cube></Schema>"#.to_owned();
        let fragments = vec![fragment];
        assert_eq!(
            fragments_to_schema(&fragments).unwrap(),
            "<Schema name=\"testname\">\n<SharedDimension name=\"shareddim\"></SharedDimension><Cube name=\"testcube\"><Dimension name=\"inner\"></Dimension></Cube><Cube name=\"a\"></Cube>\n</Schema>"
        );

        let fragment = r#"<Schema name="testname"><Dimension name="shareddim"></Dimension><Cube name="testcube"><Dimension name="inner"></Dimension></Cube><Cube name="a"></Cube></Schema>"#.to_owned();
        let fragments = vec![fragment];
        assert_eq!(
            fragments_to_schema(&fragments).unwrap(),
            "<Schema name=\"testname\">\n<Dimension name=\"shareddim\"></Dimension><Cube name=\"testcube\"><Dimension name=\"inner\"></Dimension></Cube><Cube name=\"a\"></Cube>\n</Schema>"
        );

        // Now multiple
        let f1 = r#"<Schema name="testname"><SharedDimension name="shareddim"></SharedDimension><Cube name="testcube"><Dimension name="inner"></Dimension></Cube><Cube name="a"></Cube></Schema>"#.to_owned();
        let f2 = r#"<SharedDimension name="shareddim2"></SharedDimension><Cube name="cube2"><Dimension name="inner2"></Dimension></Cube><Cube name="b"></Cube>"#.to_owned();
        let fragments = vec![f1, f2];
        assert_eq!(
            fragments_to_schema(&fragments).unwrap(),
            "<Schema name=\"testname\">\n<SharedDimension name=\"shareddim\"></SharedDimension><SharedDimension name=\"shareddim2\"></SharedDimension><Cube name=\"testcube\"><Dimension name=\"inner\"></Dimension></Cube><Cube name=\"a\"></Cube><Cube name=\"cube2\"><Dimension name=\"inner2\"></Dimension></Cube><Cube name=\"b\"></Cube>\n</Schema>"
        );

        let f1 = r#"<Schema name="testname"><Dimension name="shareddim"></Dimension><Cube name="testcube"><Dimension name="inner"></Dimension></Cube><Cube name="a"></Cube></Schema>"#.to_owned();
        let f2 = r#"<Dimension name="shareddim2"></Dimension><Cube name="cube2"><Dimension name="inner2"></Dimension></Cube><Cube name="b"></Cube>"#.to_owned();
        let fragments = vec![f1, f2];
        assert_eq!(
            fragments_to_schema(&fragments).unwrap(),
            "<Schema name=\"testname\">\n<Dimension name=\"shareddim\"></Dimension><Dimension name=\"shareddim2\"></Dimension><Cube name=\"testcube\"><Dimension name=\"inner\"></Dimension></Cube><Cube name=\"a\"></Cube><Cube name=\"cube2\"><Dimension name=\"inner2\"></Dimension></Cube><Cube name=\"b\"></Cube>\n</Schema>"
        );
    }
}
