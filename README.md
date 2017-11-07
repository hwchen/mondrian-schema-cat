# Mondrian Schema Cat (moncat)

A utility for concatenating together fragments of a Mondrian schema.n\

Takes an arbitrary number of schema fragments containing:
- schema (containing cubes and shared dims)
- shared dims
- cubes

and then concatenates the fragement sections in the correct
order (schema wraps shared dims and then cubes, in that order).

Fragments can be any of the above three in any combination, but
each fragment's internals must be in the same order as a full schema.

As of now, the logic is pretty simple, just finding the approriate chunks of text.

A future implementation may or may not parse the xml, depending on future needs.

## Use

Install rust from [rustup](rustup.rs).

Clone repo:
```
$ git clone https://github.com/hwchen/mondrian-schema-cat
$ cd mondrian-schema-cat
```

Install utility:
```
$ cargo install
```

Run:
```
$ moncat -d schemas/ -o schema.xml
```
```
$ moncat frag1.xml frag2.xml -o schema.xml
```
