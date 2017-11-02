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
// For a given cube or shared dim, it will simply pull out
// the tags and anything between the tags.
//
// In the future, there might be deeper inspection, but
// probably not worthwhile since the concatenated file
// can easily be checked by an actual Mondrian instance.

pub fn strip_schema_tags(schema: &str) -> &str {
    ""
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
