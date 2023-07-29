//! Module containing the types for parsing q-entities files.

use bitflags::bitflags;
use core::fmt;
use core::slice;

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

/// The internal error enumeration for errors that can occur while parsing a q-entities file.
#[derive(Debug)]
enum ParseError {
    /// An I/O error occured.
    Io(io::Error),
    /// A C style comment was not terminated.
    UnterminatedCStyleComment(QEntitiesParserLocation),
    /// A quoted string was not terminated.
    UnterminatedQuotedString(QEntitiesParserLocation),
    /// An escape sequence is invalid.
    InvalidEscapeSequence(QEntitiesParserLocation),
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
    /// An escape sequence is invalid.
    InvalidEscapeSequence,
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
            ParseError::InvalidEscapeSequence { .. } => {
                QEntitiesParseErrorKind::InvalidEscapeSequence
            }
        }
    }

    /// Gets the location at which the error occured within the q-entities file.
    #[inline]
    pub fn location(&self) -> Option<&QEntitiesParserLocation> {
        match self.repr.as_ref() {
            ParseError::Io { .. } => None,
            ParseError::UnterminatedCStyleComment(loc) => Some(loc),
            ParseError::UnterminatedQuotedString(loc) => Some(loc),
            ParseError::InvalidEscapeSequence(loc) => Some(loc),
        }
    }
}

impl fmt::Display for QEntitiesParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr.as_ref() {
            ParseError::Io(e) => write!(f, "io error: {e}"),
            ParseError::UnterminatedCStyleComment(loc) => {
                write!(f, "unterminated c style comment: {loc}")
            }
            ParseError::UnterminatedQuotedString(loc) => {
                write!(f, "unterminated quoted string: {loc}")
            }
            ParseError::InvalidEscapeSequence(loc) => {
                write!(f, "invalid escape sequence: {loc}")
            }
        }
    }
}

impl error::Error for QEntitiesParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.repr.as_ref() {
            ParseError::Io(e) => Some(e),
            ParseError::UnterminatedCStyleComment(_) => None,
            ParseError::UnterminatedQuotedString(_) => None,
            ParseError::InvalidEscapeSequence(_) => None,
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
            repr: Box::new(ParseError::Io(value)),
        }
    }
}

/// An error that can occur when attempting to cast a [`QEntitiesParseError`] as an inner error
/// type.
#[derive(Debug)]
pub struct QEntitiesParseErrorCastError {
    /// Sealant to prevent user's from constructing this type.
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

bitflags! {
    /// Bit-flags describing the options for parsing a q-entities file.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct QEntitiesParseFlags: u8 {
        /// Whether or not C++ style comments are enabled.
        const CPP_STYLE_COMMENTS = 0x01;
        /// Whether or not C style comments are enabled.
        const C_STYLE_COMMENTS = 0x02;
        /// Whether or not escape sequences are enabled.
        const ESCAPE = 0x04;
        /// Whether or not double quotes can be escaped.
        const ESCAPE_DOUBLE_QUOTES = 0x08;

        /// Flags that are controlled by [`QEntitiesParseEscapeOptions`].
        const ESCAPE_OPTIONS_MASK = Self::ESCAPE_DOUBLE_QUOTES.bits();
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
            flags: !QEntitiesParseFlags::ESCAPE_OPTIONS_MASK,
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
///
/// Due to the q-entities format lacking any formal specification, there exist no default options
/// for parsing a q-entities file. Because of this, when the user initially needs to create an
/// options instance they must use one of the following title specific functions:
/// * [`quake()`](Self::quake)
/// * [`quake2()`](Self::quake2)
/// * [`quake3()`](Self::quake3)
/// * [`source_engine()`](Self::source_engine)
/// * [`vtmb()`](Self::vtmb)
#[derive(Clone)]
pub struct QEntitiesParseOptions {
    /// Bit-flag options.
    flags: QEntitiesParseFlags,
}

impl QEntitiesParseOptions {
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
    /// This enables the following options:
    /// * C++ style comments
    #[inline(always)]
    pub fn quake2() -> Self {
        Self::quake()
    }

    /// Creates a new parse options instance suitable for parsing q-entities found in _Quake III:
    /// Arena_.
    ///
    /// This enables the following options:
    /// * C++ style comments
    /// * C style comments
    #[inline]
    pub fn quake3() -> Self {
        Self {
            flags: QEntitiesParseFlags::CPP_STYLE_COMMENTS | QEntitiesParseFlags::C_STYLE_COMMENTS,
        }
    }

    /// Creates a new parse options instance suitable for parsing q-entities found in the _Source
    /// Engine_.
    ///
    /// This enables the following options:
    /// * C++ style comments
    #[inline(always)]
    pub fn source_engine() -> Self {
        Self::quake()
    }

    /// Creates a new parse options instance suitable for parsing q-entities found in _Vampire The
    /// Masquerade: Bloodlines_.
    ///
    /// This enables the following options:
    /// * C++ style comments
    /// * Escape sequences for
    ///   * Double-quotes
    #[inline]
    pub fn vtmb() -> Self {
        Self {
            flags: QEntitiesParseFlags::CPP_STYLE_COMMENTS
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

    /// Changes the escape sequence options use when parsing quoted strings.
    ///
    /// A value of [`Some`] always implies that a back-slash can escape another back-slash (`\\`).
    ///
    /// A value of [`None`] will disable escape sequences entirely.
    #[inline]
    pub fn escape_options(&mut self, value: Option<QEntitiesParseEscapeOptions>) -> &mut Self {
        if let Some(escape_option_flags) = value.map(|value| value.flags) {
            // The escape options are set by unioning the bit-flags carried by the options value.
            // For this to be sound the carried escape flags must always have all other flags set.
            debug_assert_eq!(
                escape_option_flags & !QEntitiesParseFlags::ESCAPE_OPTIONS_MASK,
                !QEntitiesParseFlags::ESCAPE_OPTIONS_MASK,
            );

            self.flags |= QEntitiesParseFlags::ESCAPE | escape_option_flags;
        } else {
            self.flags &= !QEntitiesParseFlags::ESCAPE;
        }
        self
    }
}

/// The kinds of tokens that can appear within a q-entities file.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum QEntitiesTokenKind {
    /// An open brace (`{`).
    OpenBrace,
    /// A close brace (`}`).
    CloseBrace,
    /// A quoted string (`"foo bar"`).
    QuotedString,
    /// An unquoted string (`foo_bar`).
    UnquotedString,
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

    /// Assume that there exists a previously peeked byte that is still fresh and return it.
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
    pub fn peek_fresh(&self) -> u8 {
        debug_assert!(matches!(self.state, PeekByteState::Fresh));
        self.byte
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

    /// Peek the next byte within the reader with the hint that any previously peeked byte is now
    /// spoiled.
    #[inline(always)]
    fn peek_byte_spoiled(&mut self) -> Result<Option<u8>, io::Error> {
        self.peek_byte.peek_from_spoiled(&mut self.reader)
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
            if byte == b'\n' {
                break;
            }
        }
        Ok(())
    }

    /// Consumes bytes until the pattern `*/` is encountered.
    ///
    /// If no `*/` pattern is encountered before the EOF then an error is returned.
    fn skip_c_style_comment(&mut self) -> Result<(), QEntitiesParseError> {
        // Compute the start location so that it can be returned if termination pattern is
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
        while let Some(byte) = self.next_byte()? {
            let token_loc = self.location;
            match byte {
                // Discard whitespace.
                b' ' | b'\n' => (),

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

    /// Reads bytes from the inner reader into given writer until a terminating `"` byte is
    /// encountered and return the number of bytes written.
    fn parse_quoted_string(
        &mut self,
        mut writer: impl io::Write,
    ) -> Result<usize, QEntitiesParseError> {
        // Compute the start location so that it can be returned if a termination quote is not
        // encountered.
        let start_loc = QEntitiesParserLocation {
            offset: self.location.offset - 1,
            line: self.location.line,
            column: self.location.offset - 1,
        };

        let mut written = 0;
        while let Some(byte) = self.next_byte()? {
            match byte {
                // `"` terminates the string.
                b'"' => {
                    return Ok(written);
                }

                // `\` can be used to escape other bytes.
                b'\\' if self.flags.contains(QEntitiesParseFlags::ESCAPE) => match self
                    .peek_byte()?
                {
                    Some(b'\\') => {
                        let _ = self.next_byte_fresh();
                        writer.write_all(slice::from_ref(&b'\\'))?;
                    }
                    Some(b'"')
                        if self
                            .flags
                            .contains(QEntitiesParseFlags::ESCAPE_DOUBLE_QUOTES) =>
                    {
                        let _ = self.next_byte_fresh();
                        writer.write_all(slice::from_ref(&b'"'))?;
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
                    writer.write_all(slice::from_ref(&byte))?;
                    written += 1;
                }
            }
        }
        Err(ParseError::UnterminatedQuotedString(start_loc).into())
    }

    /// Reads bytes from the inner reader into given writer until some terminating whitespace or a
    /// control byte is encountered and return the number of bytes written.
    fn parse_unquoted_string(
        &mut self,
        head_byte: u8,
        mut writer: impl io::Write,
    ) -> Result<usize, QEntitiesParseError> {
        writer.write_all(slice::from_ref(&head_byte))?;
        let mut written = 1;

        while let Some(byte) = self.peek_byte()? {
            match byte {
                // Consume whitespace since it is not significant.
                b' ' | b'\n' => {
                    let _ = self.next_byte_fresh();
                    break;
                }

                // Explicit control bytes just break so that they can be re-parsed.
                b'{' | b'}' | b'"' => break,

                // `/` is special because it can be a comment. If it is a comment then we'll consume
                // the comment and break, but otherwise the `/` is part of the string.
                b'/' => {
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
                            writer.write_all(slice::from_ref(&byte))?;
                            written += 1;
                        }
                    }
                }

                // All other bytes are part of the string.
                _ => {
                    let _ = self.next_byte_fresh();
                    writer.write_all(slice::from_ref(&byte))?;
                    written += 1;
                }
            }
        }
        Ok(written)
    }
}
