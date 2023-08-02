# q-entities
[![Crates.io](https://img.shields.io/crates/v/qentities)](https://crates.io/crates/qentities) [![Crates.io](https://img.shields.io/crates/l/qentities)](https://choosealicense.com/licenses/mpl-2.0/) [![docs.rs](https://img.shields.io/docsrs/qentities)](https://docs.rs/qentities/)

A Rust crate featuring utilities related to the q-entities format.

## The q-entities Format
The q-entities format is the unofficial name which this crate gives to the otherwise unnamed format used by id Software's Quake and its derivative titles to store a map's entities.

There exists no formal specification for this format and derivative titles have on occasion been known to modify how it is parsed which makes defining a complete specification that works under all titles unfeasible.
This crate attempts to solve this problem by defining a [baseline](https://github.com/IanE9/qentities/issues/1) for parsing the format which can be further modified using a simple builder pattern.

## Basic Usage
The crate's top level module features types used to store and access a q-entities collection (most notably `QEntities`).

The `parse` module features types used to parse a q-entities file into a q-entities collection (most notably `QEntitiesParseOptions`).

### Minimal Example
```rust
use qentities::parse::QEntitiesParseOptions;

const FILE_DATA: &'static [u8] = br#"
{
"classname" "worldspawn"
"wad" "mywad.wad"
}
{
"classname" "light"
"origin" "0 0 32"
}
{
"classname" "info_player_start"
"origin" "0 0 0"
}
"#;

fn main() {
    let entities = QEntitiesParseOptions::new().parse(&FILE_DATA[..]).unwrap();
    for entity in entities.iter() {
        println!("{{");
        for kv in entity.iter() {
            let key = String::from_utf8_lossy(kv.key());
            let value = String::from_utf8_lossy(kv.value());

            println!("{key:?} {value:?}");
        }
        println!("}}");
    }
}

```
