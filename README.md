# magro

[![Build Status](https://gitlab.com/lo48576/magro/badges/develop/pipeline.svg)](https://gitlab.com/lo48576/magro/pipelines/)
![Minimum supported rustc version: 1.46](https://img.shields.io/badge/rustc-1.46+-lightgray.svg)

magro: MAnage Git RepOsitories.

## Concepts

### Collections

Collection is a set of local repositories.
Every collection has a name and base directory.

Users can use collection as a filter.
For example, "show repositories in foo collection and bar collection."

#### Default collection

If a default collection is set, target collection can be omitted on clone.

### Collections cache

Magro remembers paths of repositories in collections.
By this cache, repository listing and lookup can be done very fast.

Cache can be automatically updated on adding and removing repos from `magro` command,
using `--refresh` flag (see the usage below and `--help`).
`magro refresh` command is also available to refresh the cache unconditionally.

## Subcommands

Use `--help` option for detail.

* `clone`: Clones a repository into a collection.
* `collection`: Manages collections.
    + `set-default`: Sets or unsets a default collection.
    + `add`: Creates a new collection.
    + `del`: Deletes collections.
    + `show`: Show collections.
    + `rename`: Rename a collection.
    + `get-path`: Shows the path to the collection directory.
    + `set-path`: Sets the path to the collection directory.
* `list`: Shows repositories in collections.
* `refresh`: Refreshes collections cache.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE.txt](LICENSE-APACHE.txt) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT.txt](LICENSE-MIT.txt) or
  <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
