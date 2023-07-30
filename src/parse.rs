//! Module containing the types for parsing q-entities files.

use super::byte_chunk::ByteChunksBuilder;
use super::{QEntities, QEntityInfo, QEntityKeyValueInfo};
use bitflags::bitflags;
use core::fmt;
use core::hash::BuildHasher;
use core::slice;
use hashbrown::hash_map::DefaultHashBuilder;

use std::{error, io};

/// Location within a q-entities file.
#[derive(Debug, Clone, Copy)]
pub struct QEntitiesParserLocation {
    /// Absolute offset from the beginning of the file.
    pub offset: u64,
    /// The line number within the file.
    pub line: u64,
    /// The column number within the line.
    pub column: u64,
}

impl std::fmt::Display for QEntitiesParserLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "@{} line#{} column#{}",
            self.offset, self.line, self.column,
        )
    }
}

/// An error describing an unexpected tokens within a q-entities file.
#[derive(Debug)]
pub struct QEntitiesUnexpectedTokenError {
    /// The unexpected token's kind.
    kind: QEntitiesTokenKind,
    /// The location of the unexpected token.
    location: QEntitiesParserLocation,
}

impl QEntitiesUnexpectedTokenError {
    /// Creates a new unexpected token error.
    #[inline]
    fn new(kind: QEntitiesTokenKind, location: QEntitiesParserLocation) -> Self {
        Self { kind, location }
    }

    /// Gets the location at which the unexpected token appeared.
    #[inline]
    pub fn location(&self) -> &QEntitiesParserLocation {
        &self.location
    }

    /// Gets the kind of token that was encountered.
    #[inline]
    pub fn kind(&self) -> QEntitiesTokenKind {
        self.kind
    }
}

impl fmt::Display for QEntitiesUnexpectedTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unexpected \"{}\" token {}", self.kind, self.location)
    }
}

impl error::Error for QEntitiesUnexpectedTokenError {}

/// The internal error enumeration for errors that can occur while parsing a q-entities file.
#[derive(Debug)]
enum ParseError {
    /// An I/O error occured.
    Io(io::Error),
    /// A C style comment was not terminated.
    UnterminatedCStyleComment(QEntitiesParserLocation),
    /// A quoted string was not terminated.
    UnterminatedQuotedString(QEntitiesParserLocation),
    /// An entity was not terminated.
    UnterminatedEntity(QEntitiesParserLocation),
    /// An escape sequence is invalid.
    InvalidEscapeSequence(QEntitiesParserLocation),
    /// An unexpected token was encountered.
    UnexpectedToken(QEntitiesUnexpectedTokenError),
}

impl From<io::Error> for ParseError {
    #[inline]
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<QEntitiesUnexpectedTokenError> for ParseError {
    #[inline]
    fn from(value: QEntitiesUnexpectedTokenError) -> Self {
        Self::UnexpectedToken(value)
    }
}

/// An error that can occur during parsing of a q-entities file.
#[derive(Debug)]
pub struct QEntitiesParseError {
    repr: Box<ParseError>,
}

/// A discriminant for a kind of error that can occur during parsing of a q-entities file.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum QEntitiesParseErrorKind {
    /// An I/O error occured.
    Io,
    /// A C style comment was not terminated.
    UnterminatedCStyleComment,
    /// A quoted string was not terminated.
    UnterminatedQuotedString,
    /// An entity was not terminated.
    UnterminatedEntity,
    /// An escape sequence is invalid.
    InvalidEscapeSequence,
    /// An unexpected token was encountered.
    UnexpectedToken,
}

impl QEntitiesParseError {
    /// Gets the error's kind.
    #[inline]
    pub fn kind(&self) -> QEntitiesParseErrorKind {
        match self.repr.as_ref() {
            ParseError::Io { .. } => QEntitiesParseErrorKind::Io,
            ParseError::UnterminatedCStyleComment { .. } => {
                QEntitiesParseErrorKind::UnterminatedCStyleComment
            }
            ParseError::UnterminatedQuotedString { .. } => {
                QEntitiesParseErrorKind::UnterminatedQuotedString
            }
            ParseError::UnterminatedEntity { .. } => QEntitiesParseErrorKind::UnterminatedEntity,
            ParseError::InvalidEscapeSequence { .. } => {
                QEntitiesParseErrorKind::InvalidEscapeSequence
            }
            ParseError::UnexpectedToken { .. } => QEntitiesParseErrorKind::UnexpectedToken,
        }
    }

    /// Gets the location at which the error occured within the q-entities file.
    #[inline]
    pub fn location(&self) -> Option<&QEntitiesParserLocation> {
        match self.repr.as_ref() {
            ParseError::Io { .. } => None,
            ParseError::UnterminatedCStyleComment(location) => Some(location),
            ParseError::UnterminatedQuotedString(location) => Some(location),
            ParseError::UnterminatedEntity(location) => Some(location),
            ParseError::InvalidEscapeSequence(location) => Some(location),
            ParseError::UnexpectedToken(e) => Some(&e.location),
        }
    }
}

impl fmt::Display for QEntitiesParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr.as_ref() {
            ParseError::Io(e) => write!(f, "io error: {e}"),
            ParseError::UnterminatedCStyleComment(location) => {
                write!(f, "unterminated c style comment {location}")
            }
            ParseError::UnterminatedQuotedString(location) => {
                write!(f, "unterminated quoted string {location}")
            }
            ParseError::UnterminatedEntity(location) => {
                write!(f, "unterminated entity string {location}")
            }
            ParseError::InvalidEscapeSequence(location) => {
                write!(f, "invalid escape sequence {location}")
            }
            ParseError::UnexpectedToken(e) => e.fmt(f),
        }
    }
}

impl error::Error for QEntitiesParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.repr.as_ref() {
            ParseError::Io(e) => Some(e),
            ParseError::UnterminatedCStyleComment(_) => None,
            ParseError::UnterminatedQuotedString(_) => None,
            ParseError::UnterminatedEntity(_) => None,
            ParseError::InvalidEscapeSequence(_) => None,
            ParseError::UnexpectedToken(e) => Some(e),
        }
    }
}

impl From<ParseError> for QEntitiesParseError {
    #[inline]
    fn from(value: ParseError) -> Self {
        Self {
            repr: Box::new(value),
        }
    }
}

impl From<io::Error> for QEntitiesParseError {
    #[inline]
    fn from(value: io::Error) -> Self {
        Self {
            repr: Box::new(ParseError::from(value)),
        }
    }
}

impl From<QEntitiesUnexpectedTokenError> for QEntitiesParseError {
    #[inline]
    fn from(value: QEntitiesUnexpectedTokenError) -> Self {
        Self {
            repr: Box::new(ParseError::from(value)),
        }
    }
}

/// An error that can occur when attempting to cast a [`QEntitiesParseError`] as an inner error
/// type.
#[derive(Debug)]
pub struct QEntitiesParseErrorCastError {
    /// Sealant to prevent users from constructing this type.
    _sealed: (),
}

impl QEntitiesParseErrorCastError {
    /// Creates a new [`QEntitiesParseErrorCastError`].
    #[inline(always)]
    fn new() -> Self {
        Self { _sealed: () }
    }
}

impl fmt::Display for QEntitiesParseErrorCastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "q-entities parse error cast failed")
    }
}

impl error::Error for QEntitiesParseErrorCastError {
    #[inline(always)]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl TryFrom<QEntitiesParseError> for io::Error {
    type Error = QEntitiesParseErrorCastError;

    #[inline]
    fn try_from(value: QEntitiesParseError) -> Result<Self, Self::Error> {
        if let ParseError::Io(e) = *value.repr {
            Ok(e)
        } else {
            Err(QEntitiesParseErrorCastError::new())
        }
    }
}

impl<'a> TryFrom<&'a QEntitiesParseError> for &'a io::Error {
    type Error = QEntitiesParseErrorCastError;

    #[inline]
    fn try_from(value: &'a QEntitiesParseError) -> Result<Self, Self::Error> {
        if let ParseError::Io(e) = value.repr.as_ref() {
            Ok(e)
        } else {
            Err(QEntitiesParseErrorCastError::new())
        }
    }
}

impl TryFrom<QEntitiesParseError> for QEntitiesUnexpectedTokenError {
    type Error = QEntitiesParseErrorCastError;

    #[inline]
    fn try_from(value: QEntitiesParseError) -> Result<Self, Self::Error> {
        if let ParseError::UnexpectedToken(e) = *value.repr {
            Ok(e)
        } else {
            Err(QEntitiesParseErrorCastError::new())
        }
    }
}

impl<'a> TryFrom<&'a QEntitiesParseError> for &'a QEntitiesUnexpectedTokenError {
    type Error = QEntitiesParseErrorCastError;

    #[inline]
    fn try_from(value: &'a QEntitiesParseError) -> Result<Self, Self::Error> {
        if let ParseError::UnexpectedToken(e) = value.repr.as_ref() {
            Ok(e)
        } else {
            Err(QEntitiesParseErrorCastError::new())
        }
    }
}

bitflags! {
    /// Bit-flags describing the options for parsing a q-entities file.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct QEntitiesParseFlags: u8 {
        /// Whether or not C++ style comments are enabled.
        const CPP_STYLE_COMMENTS = 0x01;
        /// Whether or not C style comments are enabled.
        const C_STYLE_COMMENTS = 0x02;
        /// Whether or not control bytes can terminate unquoted strings.
        const CONTROLS_TERMINATE_UNQUOTED_STRINGS = 0x04;
        /// Whether or not comments can terminate unquoted strings.
        const COMMENTS_TERMINATE_UNQUOTED_STRINGS = 0x08;
        /// Whether or not escape sequences are enabled.
        const ESCAPE = 0x10;
        /// Whether or not double quotes can be escaped.
        const ESCAPE_DOUBLE_QUOTES = 0x20;

        /// Flags that are controlled by [`QEntitiesParseEscapeOptions`].
        const ESCAPE_OPTIONS = Self::ESCAPE.bits() | Self::ESCAPE_DOUBLE_QUOTES.bits();
    }
}

/// Options that describe the available escape sequences when parsing quoted strings within a
/// q-entities file.
#[derive(Clone)]
pub struct QEntitiesParseEscapeOptions {
    /// Bit-flag options.
    flags: QEntitiesParseFlags,
}

impl QEntitiesParseEscapeOptions {
    /// Creates a new escape parse options instance that only allows back-slashes to escape
    /// themselves (`\\`).
    #[inline]
    pub fn new() -> Self {
        Self {
            flags: QEntitiesParseFlags::ESCAPE,
        }
    }

    /// Changes whether or not double quotes (`"`) can be escaped.
    #[inline]
    pub fn double_quotes(&mut self, value: bool) -> &mut Self {
        self.flags
            .set(QEntitiesParseFlags::ESCAPE_DOUBLE_QUOTES, value);
        self
    }
}

impl Default for QEntitiesParseEscapeOptions {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Options that describe the how a q-entities file is parsed.
#[derive(Clone)]
pub struct QEntitiesParseOptions {
    /// Bit-flag options.
    flags: QEntitiesParseFlags,
}

impl QEntitiesParseOptions {
    /// Creates a new parse options instance with the only options enabled being those that satisfy
    /// the baseline grammar for a q-entities file.
    #[inline]
    pub fn new() -> Self {
        Self {
            flags: QEntitiesParseFlags::empty(),
        }
    }

    /// Creates a new parse options instance suitable for parsing q-entities found in _Quake_.
    ///
    /// This enables the following options:
    /// * C++ style comments
    #[inline]
    pub fn quake() -> Self {
        Self {
            flags: QEntitiesParseFlags::CPP_STYLE_COMMENTS,
        }
    }

    /// Creates a new parse options instance suitable for parsing q-entities found in _Quake II_.
    ///
    /// This enables the following additional options:
    /// * C++ style comments
    #[inline(always)]
    pub fn quake2() -> Self {
        Self::quake()
    }

    /// Creates a new parse options instance suitable for parsing q-entities found in _Quake III:
    /// Arena_.
    ///
    /// This enables the following additional options:
    /// * C++ style comments
    /// * C style comments
    #[inline]
    pub fn quake3() -> Self {
        Self {
            flags: QEntitiesParseFlags::CPP_STYLE_COMMENTS | QEntitiesParseFlags::C_STYLE_COMMENTS,
        }
    }

    /// Creates a new parse options instance suitable for parsing q-entities found in most _Source
    /// Engine_ titles.
    ///
    /// This enables the following additional options:
    /// * C++ style comments
    /// * Controls terminate unquoted strings
    #[inline(always)]
    pub fn source_engine() -> Self {
        Self {
            flags: QEntitiesParseFlags::CPP_STYLE_COMMENTS
                | QEntitiesParseFlags::CONTROLS_TERMINATE_UNQUOTED_STRINGS,
        }
    }

    /// Creates a new parse options instance suitable for parsing q-entities found in _Vampire The
    /// Masquerade: Bloodlines_.
    ///
    /// This enables the following additional options:
    /// * C++ style comments
    /// * Controls terminate unquoted strings
    /// * Escape sequences for
    ///   * Double-quotes
    #[inline]
    pub fn vtmb() -> Self {
        Self {
            flags: QEntitiesParseFlags::CPP_STYLE_COMMENTS
                | QEntitiesParseFlags::CONTROLS_TERMINATE_UNQUOTED_STRINGS
                | QEntitiesParseFlags::ESCAPE
                | QEntitiesParseFlags::ESCAPE_DOUBLE_QUOTES,
        }
    }

    /// Changes whether or not C++ style single-line comments are enabled.
    #[inline]
    pub fn cpp_style_comments(&mut self, value: bool) -> &mut Self {
        self.flags
            .set(QEntitiesParseFlags::CPP_STYLE_COMMENTS, value);
        self
    }

    /// Changes whether or not C style multi-line comments are enabled.
    #[inline]
    pub fn c_style_comments(&mut self, value: bool) -> &mut Self {
        self.flags.set(QEntitiesParseFlags::C_STYLE_COMMENTS, value);
        self
    }

    /// Changes whether or control bytes terminate unquoted strings.
    #[inline]
    pub fn controls_terminate_unquoted_strings(&mut self, value: bool) -> &mut Self {
        self.flags.set(
            QEntitiesParseFlags::CONTROLS_TERMINATE_UNQUOTED_STRINGS,
            value,
        );
        self
    }

    /// Changes whether or comments terminate unquoted strings.
    #[inline]
    pub fn comments_terminate_unquoted_strings(&mut self, value: bool) -> &mut Self {
        self.flags.set(
            QEntitiesParseFlags::COMMENTS_TERMINATE_UNQUOTED_STRINGS,
            value,
        );
        self
    }

    /// Changes the escape sequence options use when parsing quoted strings.
    ///
    /// A value of [`Some`] always implies that a back-slash can escape another back-slash (`\\`).
    ///
    /// A value of [`None`] will disable escape sequences entirely.
    #[inline]
    pub fn escape_options(&mut self, value: Option<QEntitiesParseEscapeOptions>) -> &mut Self {
        self.flags.remove(QEntitiesParseFlags::ESCAPE_OPTIONS);
        if let Some(escape_option_flags) = value.map(|value| value.flags) {
            debug_assert!(escape_option_flags.contains(QEntitiesParseFlags::ESCAPE));
            debug_assert!(!escape_option_flags.contains(!QEntitiesParseFlags::ESCAPE_OPTIONS));
            self.flags.insert(escape_option_flags);
        }
        self
    }

    /// Parse a reader as a q-entities file.
    #[inline]
    pub fn parse<R: io::Read>(&self, reader: R) -> Result<QEntities, QEntitiesParseError> {
        self.parse_with_hasher(reader, DefaultHashBuilder::default())
    }

    /// Parse a reader as a q-entities file using the given hasher.
    #[inline]
    pub fn parse_with_hasher<R: io::Read, S: BuildHasher>(
        &self,
        reader: R,
        hash_builder: S,
    ) -> Result<QEntities, QEntitiesParseError> {
        Parser::new(reader, self.clone()).parse(hash_builder)
    }
}

impl Default for QEntitiesParseOptions {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// The kinds of tokens that can appear within a q-entities file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QEntitiesTokenKind {
    /// An open brace (`{`).
    OpenBrace = b'{' as _,
    /// A close brace (`}`).
    CloseBrace = b'}' as _,
    /// A quoted string (`"foo bar"`).
    QuotedString = b'"' as _,
    /// An unquoted string (`foo_bar`).
    UnquotedString = 0,
}

impl fmt::Display for QEntitiesTokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenBrace => write!(f, "{{"),
            Self::CloseBrace => write!(f, "}}"),
            Self::QuotedString => write!(f, "quoted string"),
            Self::UnquotedString => write!(f, "unquoted string"),
        }
    }
}

/// State that a [`PeekByte`] can be in.
enum PeekByteState {
    /// The byte is unavailable and a needs to be updated from the reader.
    Spoiled,
    /// The byte available and represents the most recent state of the reader.
    Fresh,
    /// The reader has indicated that no more bytes are available.
    Unavailable,
}

/// Type that handles the abstraction of peeking bytes for [`Parser`].
struct PeekByte {
    state: PeekByteState,
    byte: u8,
}

impl fmt::Debug for PeekByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.state {
            PeekByteState::Spoiled => write!(f, "Spoiled"),
            PeekByteState::Fresh => write!(f, "Fresh({})", self.byte),
            PeekByteState::Unavailable => write!(f, "Unavailable"),
        }
    }
}

impl PeekByte {
    /// Create a new empty peek-byte.
    #[inline]
    pub fn new() -> Self {
        Self {
            state: PeekByteState::Spoiled,
            byte: 0,
        }
    }

    /// Assume that any previously peeked-byte has spoiled and perform a fresh read from the reader.
    pub fn peek_from_spoiled<R: io::Read>(
        &mut self,
        reader: &mut R,
    ) -> Result<Option<u8>, io::Error> {
        if reader.read(slice::from_mut(&mut self.byte))? == 0 {
            self.state = PeekByteState::Unavailable;
            Ok(None)
        } else {
            self.state = PeekByteState::Fresh;
            Ok(Some(self.byte))
        }
    }

    /// Either return the previously peeked-byte or read it out of the provided reader.
    pub fn peek_from<R: io::Read>(&mut self, reader: &mut R) -> Result<Option<u8>, io::Error> {
        match self.state {
            PeekByteState::Spoiled => self.peek_from_spoiled(reader),
            PeekByteState::Fresh => Ok(Some(self.byte)),
            PeekByteState::Unavailable => Ok(None),
        }
    }

    /// Attempt to take the inner byte and subsequently spoil it, or if the byte is already spoiled
    /// read the next from the given reader.
    pub fn take_from<R: io::Read>(&mut self, reader: &mut R) -> Result<Option<u8>, io::Error> {
        match self.state {
            PeekByteState::Spoiled => {
                if reader.read(slice::from_mut(&mut self.byte))? == 0 {
                    self.state = PeekByteState::Unavailable;
                    Ok(None)
                } else {
                    Ok(Some(self.byte))
                }
            }
            PeekByteState::Fresh => {
                self.state = PeekByteState::Spoiled;
                Ok(Some(self.byte))
            }
            PeekByteState::Unavailable => Ok(None),
        }
    }

    /// Assume that there exists a previously peeked byte that is still fresh and take it.
    ///
    /// This is intended to be used in scenarios where the user knows that there is a freshly peeked
    /// byte, but the compiler may have a difficult time proving such.
    ///
    /// # Panics
    /// In debug builds this function will panic if there does not actually exist a freshly peeked
    /// byte, while in release builds this function will merely return an erroneous but initialized
    /// result.
    #[inline]
    #[must_use]
    pub fn take_fresh(&mut self) -> u8 {
        debug_assert!(matches!(self.state, PeekByteState::Fresh));
        self.state = PeekByteState::Spoiled;
        self.byte
    }
}

/// State for parsing the Quake entities format from an [`io::Read`].
///
/// Note that this encapsulates the concepts of both a lexer and parser. These concepts are
/// encapsulated into a single type primarily to avoid needing to parse out entire byte-chunks in
/// contexts where the apperance of a byte-chunk is always an error.
struct Parser<R: io::Read> {
    /// The inner reader from which bytes are read.
    reader: R,
    /// The byte peeked from the reader.
    peek_byte: PeekByte,
    /// The parser's current location within the reader.
    location: QEntitiesParserLocation,
    /// Flags used tor parsing.
    flags: QEntitiesParseFlags,
}

impl<R: io::Read> Parser<R> {
    /// Create a new parser for a reader.
    #[inline]
    fn new(reader: R, options: QEntitiesParseOptions) -> Self {
        Self {
            reader,
            peek_byte: PeekByte::new(),
            location: QEntitiesParserLocation {
                offset: 0,
                line: 0,
                column: 0,
            },
            flags: options.flags,
        }
    }

    /// Peek the next unconsumed byte within the reader.
    #[inline(always)]
    fn peek_byte(&mut self) -> Result<Option<u8>, io::Error> {
        self.peek_byte.peek_from(&mut self.reader)
    }

    /// Attempt to read the next byte.
    ///
    /// This will implicitly move the location of the parser forward upon success.
    fn next_byte(&mut self) -> Result<Option<u8>, io::Error> {
        let res = self.peek_byte.take_from(&mut self.reader)?;
        if let Some(byte) = res {
            self.advance_location(byte);
        }
        Ok(res)
    }

    /// Identical behavior to [`next_byte()`](Self::next_byte()) except that this function makes the
    /// assumption that a previous peek was successful and returns the byte from that operation.
    ///
    /// # Panics
    /// This function can panic under all the same circumstances that [`PeekByte::take_fresh()`] may
    /// panic under.
    #[inline]
    #[must_use]
    fn next_byte_fresh(&mut self) -> u8 {
        let byte = self.peek_byte.take_fresh();
        self.advance_location(byte);
        byte
    }

    /// Advance the parser's location dependent upon the input byte.
    fn advance_location(&mut self, byte: u8) {
        self.location.offset += 1;
        match byte {
            b'\n' | b'\r' => {
                self.location.line += 1;
                self.location.column = 0;
            }
            _ => {
                self.location.column += 1;
            }
        }
    }

    /// Consumes bytes until the first new-line or EOF is encountered.
    fn skip_cpp_style_comment(&mut self) -> Result<(), QEntitiesParseError> {
        while let Some(byte) = self.next_byte()? {
            if matches!(byte, b'\n' | b'\r') {
                break;
            }
        }
        Ok(())
    }

    /// Consumes bytes until the pattern `*/` is encountered.
    ///
    /// If no `*/` pattern is encountered before the EOF then an error is returned.
    fn skip_c_style_comment(&mut self) -> Result<(), QEntitiesParseError> {
        // Compute the start location so that it can be returned if no termination pattern is
        // encountered.
        let start_loc = QEntitiesParserLocation {
            offset: self.location.offset - 2,
            line: self.location.line,
            column: self.location.offset - 2,
        };

        while let Some(byte) = self.next_byte()? {
            if byte == b'*' && matches!(self.peek_byte()?, Some(b'/')) {
                let _ = self.next_byte_fresh();
                return Ok(());
            }
        }

        Err(ParseError::UnterminatedCStyleComment(start_loc).into())
    }

    /// Consumes bytes until a byte that is neither whitespace nor part of a comment is encountered
    /// and returns that byte as well as its location.
    fn next_significant_byte(
        &mut self,
    ) -> Result<Option<(u8, QEntitiesParserLocation)>, QEntitiesParseError> {
        while let Some(byte) = self.peek_byte()? {
            let token_loc = self.location;
            let _ = self.next_byte_fresh();
            match byte {
                // Discard whitespace.
                _ if byte.is_ascii_whitespace() => (),

                // `/` may be part of a comment.
                b'/' => match self.peek_byte()? {
                    // `//` is a C++ style comment.
                    Some(b'/') if self.flags.contains(QEntitiesParseFlags::CPP_STYLE_COMMENTS) => {
                        let _ = self.next_byte_fresh();
                        self.skip_cpp_style_comment()?;
                    }

                    // `/*` is a C style comment.
                    Some(b'*') if self.flags.contains(QEntitiesParseFlags::C_STYLE_COMMENTS) => {
                        let _ = self.next_byte_fresh();
                        self.skip_c_style_comment()?;
                    }

                    // All other patterns are not comments.
                    _ => return Ok(Some((byte, token_loc))),
                },

                // Everything else is a significant byte.
                _ => {
                    return Ok(Some((byte, token_loc)));
                }
            }
        }
        Ok(None)
    }

    /// Reads bytes from the inner reader into given buffer until a terminating `"` byte is
    /// encountered.
    fn parse_quoted_string(&mut self, buf: &mut Vec<u8>) -> Result<(), QEntitiesParseError> {
        // Compute the start location so that it can be returned if a termination quote is not
        // encountered.
        let start_loc = QEntitiesParserLocation {
            offset: self.location.offset - 1,
            line: self.location.line,
            column: self.location.offset - 1,
        };

        while let Some(byte) = self.next_byte()? {
            match byte {
                // `"` terminates the string.
                b'"' => {
                    return Ok(());
                }

                // `\` can be used to escape other bytes.
                b'\\' if self.flags.contains(QEntitiesParseFlags::ESCAPE) => match self
                    .peek_byte()?
                {
                    Some(escape_byte @ b'\\') => {
                        let _ = self.next_byte_fresh();
                        buf.push(escape_byte);
                    }
                    Some(escape_byte @ b'"')
                        if self
                            .flags
                            .contains(QEntitiesParseFlags::ESCAPE_DOUBLE_QUOTES) =>
                    {
                        let _ = self.next_byte_fresh();
                        buf.push(escape_byte);
                    }
                    _ => {
                        return Err(ParseError::InvalidEscapeSequence(QEntitiesParserLocation {
                            offset: self.location.offset - 1,
                            line: self.location.line,
                            column: self.location.column - 1,
                        })
                        .into())
                    }
                },

                // All other bytes are part of the string.
                _ => {
                    buf.push(byte);
                }
            }
        }

        Err(ParseError::UnterminatedQuotedString(start_loc).into())
    }

    /// Reads bytes from the inner reader into given bufer until some terminating byte is
    /// encountered.
    fn parse_unquoted_string(
        &mut self,
        head_byte: u8,
        buf: &mut Vec<u8>,
    ) -> Result<(), QEntitiesParseError> {
        buf.push(head_byte);

        while let Some(byte) = self.peek_byte()? {
            match byte {
                // Consume whitespace since it is not significant.
                _ if byte.is_ascii_whitespace() => {
                    let _ = self.next_byte_fresh();
                    break;
                }

                // Explicit control bytes just break so that they can be re-parsed.
                b'{' | b'}' | b'"'
                    if self
                        .flags
                        .contains(QEntitiesParseFlags::CONTROLS_TERMINATE_UNQUOTED_STRINGS) =>
                {
                    break;
                }

                // `/` is special because it can be a comment. If it is a comment then we'll consume
                // the comment and break, but otherwise the `/` is part of the string.
                b'/' if self
                    .flags
                    .contains(QEntitiesParseFlags::COMMENTS_TERMINATE_UNQUOTED_STRINGS) =>
                {
                    let _ = self.next_byte_fresh();
                    match self.peek_byte()? {
                        // `//` is a C++ style comment.
                        Some(b'/')
                            if self.flags.contains(QEntitiesParseFlags::CPP_STYLE_COMMENTS) =>
                        {
                            let _ = self.next_byte_fresh();
                            self.skip_cpp_style_comment()?;
                            break;
                        }

                        // `/*` is a C style comment.
                        Some(b'*')
                            if self.flags.contains(QEntitiesParseFlags::C_STYLE_COMMENTS) =>
                        {
                            let _ = self.next_byte_fresh();
                            self.skip_c_style_comment()?;
                            break;
                        }

                        // All other patterns are not comments. Note that the second byte is not
                        // consumed because it may whitespace or a control byte.
                        _ => {
                            buf.push(byte);
                        }
                    }
                }

                // All other bytes are part of the string.
                _ => {
                    let _ = self.next_byte_fresh();
                    buf.push(byte);
                }
            }
        }

        Ok(())
    }

    fn parse<S: BuildHasher>(&mut self, hash_builder: S) -> Result<QEntities, QEntitiesParseError> {
        /// State the parser can be in.
        #[derive(Debug, Clone, Copy)]
        enum ParseState {
            /// The parser is searching for the next entity.
            NextEntity,
            /// The parser is searching for a key.
            NextKey,
            /// The parser is searching for a value.
            NextValue,
        }

        // Location at which the last entity began. This is used to return an error if the EOF is
        // reached while still parsing an entity.
        let mut entity_start_loc = QEntitiesParserLocation {
            offset: 0,
            line: 0,
            column: 0,
        };

        // Intermediates for constructing the `QEntities` instance.
        let mut entities = Vec::new();
        let mut key_values = Vec::new();
        let mut byte_chunks = ByteChunksBuilder::with_hasher(hash_builder);
        let mut key_chunk = 0;

        // Scratch buffer which is used to store keys and values.
        let mut scratch = Vec::new();

        let mut state = ParseState::NextEntity;
        while let Some((token_head_byte, token_location)) = self.next_significant_byte()? {
            let token_kind = match token_head_byte {
                b'{' => QEntitiesTokenKind::OpenBrace,
                b'}' => QEntitiesTokenKind::CloseBrace,
                b'"' => QEntitiesTokenKind::QuotedString,
                _ => QEntitiesTokenKind::UnquotedString,
            };

            state = match state {
                ParseState::NextEntity => match token_kind {
                    QEntitiesTokenKind::OpenBrace => {
                        entity_start_loc = token_location;
                        entities.push(QEntityInfo {
                            first_kv: key_values.len(),
                            kvs_length: 0,
                        });

                        ParseState::NextKey
                    }

                    _ => {
                        return Err(
                            QEntitiesUnexpectedTokenError::new(token_kind, token_location).into(),
                        )
                    }
                },

                ParseState::NextKey => match token_kind {
                    QEntitiesTokenKind::CloseBrace => ParseState::NextEntity,

                    QEntitiesTokenKind::QuotedString => {
                        scratch.clear();
                        self.parse_quoted_string(&mut scratch)?;
                        key_chunk = byte_chunks.chunk(&scratch);
                        ParseState::NextValue
                    }

                    QEntitiesTokenKind::UnquotedString => {
                        scratch.clear();
                        self.parse_unquoted_string(token_head_byte, &mut scratch)?;
                        key_chunk = byte_chunks.chunk(&scratch);
                        ParseState::NextValue
                    }

                    _ => {
                        return Err(
                            QEntitiesUnexpectedTokenError::new(token_kind, token_location).into(),
                        )
                    }
                },

                ParseState::NextValue => {
                    let value_chunk = match token_kind {
                        QEntitiesTokenKind::QuotedString => {
                            scratch.clear();
                            self.parse_quoted_string(&mut scratch)?;
                            byte_chunks.chunk(&scratch)
                        }

                        QEntitiesTokenKind::UnquotedString => {
                            scratch.clear();
                            self.parse_unquoted_string(token_head_byte, &mut scratch)?;
                            byte_chunks.chunk(&scratch)
                        }

                        _ => {
                            return Err(QEntitiesUnexpectedTokenError::new(
                                token_kind,
                                token_location,
                            )
                            .into())
                        }
                    };

                    key_values.push(QEntityKeyValueInfo {
                        key_chunk,
                        value_chunk,
                    });
                    entities.last_mut().unwrap().kvs_length += 1;

                    ParseState::NextKey
                }
            };
        }

        match state {
            ParseState::NextEntity => Ok(QEntities {
                entities: entities.into(),
                key_values: key_values.into(),
                byte_chunks: byte_chunks.into(),
            }),
            _ => Err(ParseError::UnterminatedEntity(entity_start_loc).into()),
        }
    }
}