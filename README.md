# q-entities
A Rust crate featuring utilities related to the q-entities format.

## The q-entities format
The q-entities format is the unofficial name which this crate gives to the otherwise unnamed format found in id Software's Quake and its derivative titles.

There exists no formal specification for this format and derivative titles have on occasion been known to modify how it is parsed which makes defining a complete specification that works under all titles unfeasible. This crate attempts to solve this problem by defining a _baseline grammar_ for parsing the format which can be further modified using a simple builder pattern.
