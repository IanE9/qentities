# q-entities
A Rust crate featuring utilities related to the q-entities format.

## The q-entities format
The q-entities format is the unofficial name which this crate gives to the otherwise unnamed format found in id Software's Quake and its derivative titles.

There exists no formal specification for this format and derivative titles have on occasion been known to modify how it is parsed which makes defining a complete specification that works under all titles unfeasible.
This crate attempts to solve this problem by defining a _baseline_ for parsing the format which can be further modified using a simple builder pattern.

### Baseline
The format is (most often) a human readable format which may be encoded in any superset of ASCII.

The format specifies three control characters:
* `{` - Open Brace
* `}` - Close Brace
* `"` - Double Quote

The format specifies six whitespace characters:
* `0x20` - Space
* `0x0C` - Form Feed
* `0x0A` - Line Feed
* `0x0D` - Carriage Return
* `0x09` - Horizontal Tab
* `0x0B` - Vertical Tab

A file consists of any number of _entities_ which further consist of any number of _key-value pairs_.

An _entity_ is an anonymous collection of _key-value pairs_. The beginning of an _entity_ is denoted by an opening brace `{` and the _entity_ is later terminated by a closing brace `}`. The nesting of _entities_ is not permitted.

Between the braces that delimit an _entity_ are any number of _key-value pairs_ for that _entity_. A _key-value pair_ is a pair of two _strings_ that appear consecutively where the _key_ is _string_ on the left and the _value_ is the _string_ on the right.

The _strings_ that compose _key-value pairs_ may be written in either the _quoted_ or _unquoted_ form.

The beginning of a _quoted string_ is denoted by an opening quote `"` and later terminated by a closing quote `"`. The _string_ is the collection of all characters that appear between these quotes.

The beginning of an _unquoted string_ is denoted by the appearance of any character that is neither a control nor whitespace character and is later terminated by the apperance of any whitespace character. The _string_ is the collection formed by the first delimiting character and all characters up too but not including the terminating delimiting character.
