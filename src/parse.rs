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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QEntitiesParserLocation {
    /// Absolute offset from the beginning of the file.
    offset: u64,
    /// The line number within the file.
    line: u64,
    /// The column number within the line.
    column: u64,
}

impl QEntitiesParserLocation {
    /// Gets the location's absolute offset from the beginning of the file.
    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Gets the location's line number within the file.
    #[inline]
    pub fn line(&self) -> u64 {
        self.line
    }

    /// Gets the location's column number within the line.
    #[inline]
    pub fn column(&self) -> u64 {
        self.column
    }
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

/// An error describing an unexpected token within a q-entities file.
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
    /// A key was too long.
    KeyTooLong(QEntitiesParserLocation),
    /// A value was too long,
    ValueTooLong(QEntitiesParserLocation),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// A key was too long.
    KeyTooLong,
    /// A value was too long,
    ValueTooLong,
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
            ParseError::KeyTooLong { .. } => QEntitiesParseErrorKind::KeyTooLong,
            ParseError::ValueTooLong { .. } => QEntitiesParseErrorKind::ValueTooLong,
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
            ParseError::KeyTooLong(location) => Some(&location),
            ParseError::ValueTooLong(location) => Some(&location),
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
            ParseError::KeyTooLong(location) => {
                write!(f, "key too long {location}")
            }
            ParseError::ValueTooLong(location) => {
                write!(f, "key too long {location}")
            }
        }
    }
}

impl error::Error for QEntitiesParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.repr.as_ref() {
            ParseError::Io(e) => Some(e),
            ParseError::UnterminatedCStyleComment { .. } => None,
            ParseError::UnterminatedQuotedString { .. } => None,
            ParseError::UnterminatedEntity { .. } => None,
            ParseError::InvalidEscapeSequence { .. } => None,
            ParseError::UnexpectedToken(e) => Some(e),
            ParseError::KeyTooLong { .. } => None,
            ParseError::ValueTooLong { .. } => None,
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
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// use qentities::parse::{QEntitiesParseEscapeOptions, QEntitiesParseOptions};
    ///
    /// let src = br#"
    /// {
    /// classname worldspawn
    /// script_fn "func(\"arg a\", \"arg b\")"
    /// }"#;
    ///
    /// let mut escape_options = QEntitiesParseEscapeOptions::new();
    /// escape_options.double_quotes(true);
    ///
    /// let entities = QEntitiesParseOptions::new()
    ///     .escape_options(Some(escape_options))
    ///     .parse(&src[..])
    ///     .unwrap();
    /// assert_eq!(entities.len(), 1);
    ///
    /// let entity = entities.get(0).unwrap();
    /// assert_eq!(entity.len(), 2);
    ///
    /// let (key_a, value_a) = entity.get(0).map(|kv| (kv.key(), kv.value())).unwrap();
    /// let (key_b, value_b) = entity.get(1).map(|kv| (kv.key(), kv.value())).unwrap();
    ///
    /// assert_eq!(key_a, b"classname");
    /// assert_eq!(value_a, b"worldspawn");
    ///
    /// assert_eq!(key_b, b"script_fn");
    /// assert_eq!(value_b, b"func(\"arg a\", \"arg b\")");
    /// ```
    #[inline]
    pub fn double_quotes(&mut self, value: bool) -> &mut Self {
        self.flags
            .set(QEntitiesParseFlags::ESCAPE_DOUBLE_QUOTES, value);
        self
    }

    /// Same as [`double_quotes()`](Self::double_quotes) but takes `self` by value.
    #[inline]
    pub fn with_double_quotes(mut self, value: bool) -> Self {
        self.double_quotes(value);
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
/// # Title Specific Presets
/// Several functions are provided that create an options instance suitable for parsing q-entities
/// found in a specific title. Users should understand that the intended purpose of these functions
/// does not extend beyond this goal and as such failure to support the q-entities of their
/// respective titles is considered a bug.
///
/// The options enabled by these functions are documented for the convenience of the user, but
/// **users should not depend on the options enabled by these functions remaining consistent between
/// non-major releases**. This caveat, though somewhat burdening to users, is necessary to enable
/// fixing bugs in these functions outside of major releases.
///
/// The following functions are subject to the aforementioned details:
/// * [`quake()`](Self::quake)
/// * [`quake2()`](Self::quake2)
/// * [`quake3()`](Self::quake3)
/// * [`source_engine()`](Self::source_engine)
/// * [`vtmb()`](Self::vtmb)
#[derive(Clone)]
pub struct QEntitiesParseOptions {
    /// Bit-flag options.
    flags: QEntitiesParseFlags,
    /// The maximum length that a key is allowed to be.
    max_key_length: usize,
    /// The maximum length that a value is allowed to be.
    max_value_length: usize,
}

impl QEntitiesParseOptions {
    /// Creates a new parse options instance with the only options enabled being those that satisfy
    /// the baseline grammar for a q-entities file.
    #[inline]
    pub fn new() -> Self {
        Self {
            flags: QEntitiesParseFlags::empty(),
            max_key_length: usize::MAX,
            max_value_length: usize::MAX,
        }
    }

    /// [Title Specific Preset](Self#title-specific-presets) for parsing q-entities found in
    /// _Quake_.
    ///
    /// # Current Release Options
    /// This function enables the following options in the current release:
    /// * C++ style comments
    #[inline]
    pub fn quake() -> Self {
        Self {
            flags: QEntitiesParseFlags::CPP_STYLE_COMMENTS,
            ..Self::new()
        }
    }

    /// [Title Specific Preset](Self#title-specific-presets) for parsing q-entities found in
    /// _Quake II_.
    ///
    /// # Current Release Options
    /// This function enables the following options in the current release:
    /// * C++ style comments
    #[inline(always)]
    pub fn quake2() -> Self {
        Self::quake()
    }

    /// [Title Specific Preset](Self#title-specific-presets) for parsing q-entities found in
    /// _Quake III: Arena_.
    ///
    /// # Current Release Options
    /// This function enables the following options in the current release:
    /// * C++ style comments
    /// * C style comments
    #[inline]
    pub fn quake3() -> Self {
        Self {
            flags: QEntitiesParseFlags::CPP_STYLE_COMMENTS | QEntitiesParseFlags::C_STYLE_COMMENTS,
            ..Self::new()
        }
    }

    /// [Title Specific Preset](Self#title-specific-presets) for parsing q-entities found in most
    /// _Source Engine_ titles.
    ///
    /// # Current Release Options
    /// This function enables the following options in the current release:
    /// * C++ style comments
    /// * Controls terminate unquoted strings
    #[inline(always)]
    pub fn source_engine() -> Self {
        Self {
            flags: QEntitiesParseFlags::CPP_STYLE_COMMENTS
                | QEntitiesParseFlags::CONTROLS_TERMINATE_UNQUOTED_STRINGS,
            ..Self::new()
        }
    }

    /// [Title Specific Preset](Self#title-specific-presets) for parsing q-entities found in
    /// _Vampire The Masquerade: Bloodlines_.
    ///
    /// # Current Release Options
    /// This function enables the following options in the current release:
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
            ..Self::new()
        }
    }

    /// Changes whether or not C++ style single-line comments are enabled.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// use qentities::parse::QEntitiesParseOptions;
    ///
    /// let src = br#"
    /// { // worldspawn
    /// classname worldspawn
    /// }"#;
    ///
    /// let entities = QEntitiesParseOptions::new()
    ///     .cpp_style_comments(true)
    ///     .parse(&src[..])
    ///     .unwrap();
    /// assert_eq!(entities.len(), 1);
    ///
    /// let entity = entities.get(0).unwrap();
    /// assert_eq!(entity.len(), 1);
    ///
    /// let (key, value) = entity.get(0).map(|kv| (kv.key(), kv.value())).unwrap();
    /// assert_eq!(key, b"classname");
    /// assert_eq!(value, b"worldspawn");
    /// ```
    #[inline]
    pub fn cpp_style_comments(&mut self, value: bool) -> &mut Self {
        self.flags
            .set(QEntitiesParseFlags::CPP_STYLE_COMMENTS, value);
        self
    }

    /// Same as [`cpp_style_comments()`](Self::cpp_style_comments) but takes `self` by value.
    #[inline]
    pub fn with_cpp_style_comments(mut self, value: bool) -> Self {
        self.cpp_style_comments(value);
        self
    }

    /// Changes whether or not C style multi-line comments are enabled.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// use qentities::parse::QEntitiesParseOptions;
    ///
    /// let src = br#"
    /// /**
    ///  * my cool entities
    ///  */
    ///
    /// { /* worldspawn */
    /// /* key */ classname /* value */ worldspawn
    /// }"#;
    ///
    /// let entities = QEntitiesParseOptions::new()
    ///     .c_style_comments(true)
    ///     .parse(&src[..])
    ///     .unwrap();
    /// assert_eq!(entities.len(), 1);
    ///
    /// let entity = entities.get(0).unwrap();
    /// assert_eq!(entity.len(), 1);
    ///
    /// let (key, value) = entity.get(0).map(|kv| (kv.key(), kv.value())).unwrap();
    /// assert_eq!(key, b"classname");
    /// assert_eq!(value, b"worldspawn");
    /// ```
    #[inline]
    pub fn c_style_comments(&mut self, value: bool) -> &mut Self {
        self.flags.set(QEntitiesParseFlags::C_STYLE_COMMENTS, value);
        self
    }

    /// Same as [`c_style_comments()`](Self::c_style_comments) but takes `self` by value.
    #[inline]
    pub fn with_c_style_comments(mut self, value: bool) -> Self {
        self.c_style_comments(value);
        self
    }

    /// Changes whether or control bytes terminate unquoted strings.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// use qentities::parse::QEntitiesParseOptions;
    ///
    /// let src = br#"{classname"worldspawn"wad mywad.wad}"#;
    ///
    /// let entities = QEntitiesParseOptions::new()
    ///     .controls_terminate_unquoted_strings(true)
    ///     .parse(&src[..])
    ///     .unwrap();
    /// assert_eq!(entities.len(), 1);
    ///
    /// let entity = entities.get(0).unwrap();
    /// assert_eq!(entity.len(), 2);
    ///
    /// let (key_a, value_a) = entity.get(0).map(|kv| (kv.key(), kv.value())).unwrap();
    /// let (key_b, value_b) = entity.get(1).map(|kv| (kv.key(), kv.value())).unwrap();
    ///
    /// assert_eq!(key_a, b"classname");
    /// assert_eq!(value_a, b"worldspawn");
    ///
    /// assert_eq!(key_b, b"wad");
    /// assert_eq!(value_b, b"mywad.wad");
    /// ```
    #[inline]
    pub fn controls_terminate_unquoted_strings(&mut self, value: bool) -> &mut Self {
        self.flags.set(
            QEntitiesParseFlags::CONTROLS_TERMINATE_UNQUOTED_STRINGS,
            value,
        );
        self
    }

    /// Same as [`controls_terminate_unquoted_strings()`](Self::controls_terminate_unquoted_strings)
    /// but takes `self` by value.
    #[inline]
    pub fn with_controls_terminate_unquoted_strings(mut self, value: bool) -> Self {
        self.controls_terminate_unquoted_strings(value);
        self
    }

    /// Changes whether or comments terminate unquoted strings.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// use qentities::parse::QEntitiesParseOptions;
    ///
    /// let src = b"{classname/**/worldspawn//\n}";
    ///
    /// let entities = QEntitiesParseOptions::new()
    ///     .cpp_style_comments(true)
    ///     .c_style_comments(true)
    ///     .comments_terminate_unquoted_strings(true)
    ///     .parse(&src[..])
    ///     .unwrap();
    /// assert_eq!(entities.len(), 1);
    ///
    /// let entity = entities.get(0).unwrap();
    /// assert_eq!(entity.len(), 1);
    ///
    /// let (key, value) = entity.get(0).map(|kv| (kv.key(), kv.value())).unwrap();
    /// assert_eq!(key, b"classname");
    /// assert_eq!(value, b"worldspawn");
    /// ```
    #[inline]
    pub fn comments_terminate_unquoted_strings(&mut self, value: bool) -> &mut Self {
        self.flags.set(
            QEntitiesParseFlags::COMMENTS_TERMINATE_UNQUOTED_STRINGS,
            value,
        );
        self
    }

    /// Same as [`comments_terminate_unquoted_strings()`](Self::comments_terminate_unquoted_strings)
    /// but takes `self` by value.
    #[inline]
    pub fn with_comments_terminate_unquoted_strings(mut self, value: bool) -> Self {
        self.comments_terminate_unquoted_strings(value);
        self
    }

    /// Changes the escape sequence options use when parsing quoted strings.
    ///
    /// A value of [`Some`] always implies that a back-slash can escape another back-slash (`\\`).
    ///
    /// A value of [`None`] will disable escape sequences entirely.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// use qentities::parse::{QEntitiesParseEscapeOptions, QEntitiesParseOptions};
    ///
    /// let src = br#"
    /// {
    /// classname worldspawn
    /// wad "wads\\mywad.wad"
    /// }"#;
    ///
    /// let entities = QEntitiesParseOptions::new()
    ///     .escape_options(Some(QEntitiesParseEscapeOptions::new()))
    ///     .parse(&src[..])
    ///     .unwrap();
    /// assert_eq!(entities.len(), 1);
    ///
    /// let entity = entities.get(0).unwrap();
    /// assert_eq!(entity.len(), 2);
    ///
    /// let (key_a, value_a) = entity.get(0).map(|kv| (kv.key(), kv.value())).unwrap();
    /// let (key_b, value_b) = entity.get(1).map(|kv| (kv.key(), kv.value())).unwrap();
    ///
    /// assert_eq!(key_a, b"classname");
    /// assert_eq!(value_a, b"worldspawn");
    ///
    /// assert_eq!(key_b, b"wad");
    /// assert_eq!(value_b, b"wads\\mywad.wad");
    /// ```
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

    /// Same as [`escape_options()`](Self::escape_options) but takes `self` by value.
    #[inline]
    pub fn with_escape_options(mut self, value: Option<QEntitiesParseEscapeOptions>) -> Self {
        self.escape_options(value);
        self
    }

    /// Changes the maximum allowed byte length of a parsed key.
    ///
    /// Using a value of [`None`] specifies that there should be no limit.
    #[inline]
    pub fn max_key_length(&mut self, value: Option<usize>) -> &mut Self {
        self.max_key_length = value.unwrap_or(usize::MAX);
        self
    }

    /// Same as [`max_key_length()`](Self::max_key_length) but takes `self` by value.
    #[inline]
    pub fn with_max_key_length(mut self, value: Option<usize>) -> Self {
        self.max_key_length(value);
        self
    }

    /// Changes the maximum allowed byte length of a parsed value.
    ///
    /// Using a value of [`None`] specifies that there should be no limit.
    #[inline]
    pub fn max_value_length(&mut self, value: Option<usize>) -> &mut Self {
        self.max_value_length = value.unwrap_or(usize::MAX);
        self
    }

    /// Same as [`max_value_length()`](Self::max_value_length) but takes `self` by value.
    #[inline]
    pub fn with_max_value_length(mut self, value: Option<usize>) -> Self {
        self.max_value_length(value);
        self
    }

    /// Parse a reader as a q-entities file.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// use qentities::parse::QEntitiesParseOptions;
    ///
    /// let entities = QEntitiesParseOptions::new().parse(&b"{ classname worldspawn }"[..]).unwrap();
    /// assert_eq!(entities.len(), 1);
    ///
    /// let entity = entities.get(0).unwrap();
    /// assert_eq!(entity.len(), 1);
    ///
    /// let (key, value) = entity.get(0).map(|kv| (kv.key(), kv.value())).unwrap();
    /// assert_eq!(key, b"classname");
    /// assert_eq!(value, b"worldspawn");
    /// ```
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

/// The kinds of sources strings can be parsed from within a q-entities file.
#[derive(Debug, Clone, Copy)]
enum StringSourceKind {
    /// The source is a key.
    Key,
    /// The source is a value.
    Value,
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
    /// options used for parsing.
    options: QEntitiesParseOptions,
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
                line: 1,
                column: 1,
            },
            options,
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
                self.location.column = 1;
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
            column: self.location.column - 2,
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
                    Some(b'/')
                        if self
                            .options
                            .flags
                            .contains(QEntitiesParseFlags::CPP_STYLE_COMMENTS) =>
                    {
                        let _ = self.next_byte_fresh();
                        self.skip_cpp_style_comment()?;
                    }

                    // `/*` is a C style comment.
                    Some(b'*')
                        if self
                            .options
                            .flags
                            .contains(QEntitiesParseFlags::C_STYLE_COMMENTS) =>
                    {
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

    /// Gets the maximum length for a string's source kind.
    fn string_source_max_length(&self, kind: StringSourceKind) -> usize {
        match kind {
            StringSourceKind::Key => self.options.max_key_length,
            StringSourceKind::Value => self.options.max_value_length,
        }
    }

    /// Helper to attempt pushing a byte to a buffer while additionally returning an error if doing
    /// so would cause the buffer to exceeed a length limit.
    fn push_string_buf(
        kind: StringSourceKind,
        buf: &mut Vec<u8>,
        byte: u8,
        max_length: usize,
        start_location: QEntitiesParserLocation,
    ) -> Result<(), QEntitiesParseError> {
        if buf.len() < max_length {
            buf.push(byte);
            Ok(())
        } else {
            Err(match kind {
                StringSourceKind::Key => ParseError::KeyTooLong(start_location),
                StringSourceKind::Value => ParseError::ValueTooLong(start_location),
            }
            .into())
        }
    }

    /// Reads bytes from the inner reader into given buffer until a terminating `"` byte is
    /// encountered.
    fn parse_quoted_string(
        &mut self,
        source_kind: StringSourceKind,
        buf: &mut Vec<u8>,
    ) -> Result<(), QEntitiesParseError> {
        buf.clear();

        let max_length = self.string_source_max_length(source_kind);

        // Compute the start location so that it can be returned if an error is encountered.
        let start_location = QEntitiesParserLocation {
            offset: self.location.offset - 1,
            line: self.location.line,
            column: self.location.column - 1,
        };

        while let Some(byte) = self.next_byte()? {
            match byte {
                // `"` terminates the string.
                b'"' => {
                    return Ok(());
                }

                // `\` can be used to escape other bytes.
                b'\\' if self.options.flags.contains(QEntitiesParseFlags::ESCAPE) => match self
                    .peek_byte()?
                {
                    Some(escape_byte @ b'\\') => {
                        let _ = self.next_byte_fresh();
                        Self::push_string_buf(
                            source_kind,
                            buf,
                            escape_byte,
                            max_length,
                            start_location,
                        )?;
                    }
                    Some(escape_byte @ b'"')
                        if self
                            .options
                            .flags
                            .contains(QEntitiesParseFlags::ESCAPE_DOUBLE_QUOTES) =>
                    {
                        let _ = self.next_byte_fresh();
                        Self::push_string_buf(
                            source_kind,
                            buf,
                            escape_byte,
                            max_length,
                            start_location,
                        )?;
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
                    Self::push_string_buf(source_kind, buf, byte, max_length, start_location)?;
                }
            }
        }

        Err(ParseError::UnterminatedQuotedString(start_location).into())
    }

    /// Reads bytes from the inner reader into given bufer until some terminating byte is
    /// encountered.
    fn parse_unquoted_string(
        &mut self,
        source_kind: StringSourceKind,
        head_byte: u8,
        buf: &mut Vec<u8>,
    ) -> Result<(), QEntitiesParseError> {
        buf.clear();

        let max_length = self.string_source_max_length(source_kind);

        // Compute the start location so that it can be returned if an error is encountered.
        let start_location = QEntitiesParserLocation {
            offset: self.location.offset - 1,
            line: self.location.line,
            column: self.location.column - 1,
        };

        Self::push_string_buf(source_kind, buf, head_byte, max_length, start_location)?;

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
                        .options
                        .flags
                        .contains(QEntitiesParseFlags::CONTROLS_TERMINATE_UNQUOTED_STRINGS) =>
                {
                    break;
                }

                // `/` is special because it can be a comment. If it is a comment then we'll consume
                // the comment and break, but otherwise the `/` is part of the string.
                b'/' if self
                    .options
                    .flags
                    .contains(QEntitiesParseFlags::COMMENTS_TERMINATE_UNQUOTED_STRINGS) =>
                {
                    let _ = self.next_byte_fresh();
                    match self.peek_byte()? {
                        // `//` is a C++ style comment.
                        Some(b'/')
                            if self
                                .options
                                .flags
                                .contains(QEntitiesParseFlags::CPP_STYLE_COMMENTS) =>
                        {
                            let _ = self.next_byte_fresh();
                            self.skip_cpp_style_comment()?;
                            break;
                        }

                        // `/*` is a C style comment.
                        Some(b'*')
                            if self
                                .options
                                .flags
                                .contains(QEntitiesParseFlags::C_STYLE_COMMENTS) =>
                        {
                            let _ = self.next_byte_fresh();
                            self.skip_c_style_comment()?;
                            break;
                        }

                        // All other patterns are not comments. Note that the second byte is not
                        // consumed because it may whitespace or a control byte.
                        _ => {
                            Self::push_string_buf(
                                source_kind,
                                buf,
                                byte,
                                max_length,
                                start_location,
                            )?;
                        }
                    }
                }

                // All other bytes are part of the string.
                _ => {
                    let _ = self.next_byte_fresh();
                    Self::push_string_buf(source_kind, buf, byte, max_length, start_location)?;
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
                        self.parse_quoted_string(StringSourceKind::Key, &mut scratch)?;
                        key_chunk = byte_chunks.chunk(&scratch);
                        ParseState::NextValue
                    }

                    QEntitiesTokenKind::UnquotedString => {
                        self.parse_unquoted_string(
                            StringSourceKind::Key,
                            token_head_byte,
                            &mut scratch,
                        )?;
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
                            self.parse_quoted_string(StringSourceKind::Value, &mut scratch)?;
                            byte_chunks.chunk(&scratch)
                        }

                        QEntitiesTokenKind::UnquotedString => {
                            scratch.clear();
                            self.parse_unquoted_string(
                                StringSourceKind::Value,
                                token_head_byte,
                                &mut scratch,
                            )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use bstr::BStr;

    /// Variant for an expected error in [`ExpectedError`].
    #[derive(Clone, Copy)]
    enum ExpectedErrorVariant {
        SimpleKind(QEntitiesParseErrorKind),
        UnexpectedToken(QEntitiesTokenKind),
    }

    /// Helper for asserting that an error occured as expected.
    struct ExpectedError<'a> {
        src: &'a [u8],
        kind: ExpectedErrorVariant,
        location: QEntitiesParserLocation,
    }

    impl ExpectedError<'_> {
        /// Asserts that the expected error occured while parsing with the given parse options.
        fn test(&self, parse_opts: &QEntitiesParseOptions) {
            match parse_opts.parse(&self.src[..]) {
                Ok(_) => panic!(
                    "parsing of {:?} unexpectedly succeeded",
                    BStr::new(self.src),
                ),
                Err(e) => {
                    match self.kind {
                        ExpectedErrorVariant::SimpleKind(k) => assert_eq!(
                            e.kind(),
                            k,
                            "parsing of {:?} returned unexpected error kind",
                            BStr::new(self.src),
                        ),
                        ExpectedErrorVariant::UnexpectedToken(tk) => {
                            assert_eq!(
                                e.kind(),
                                QEntitiesParseErrorKind::UnexpectedToken,
                                "parsing of {:?} returned unexpected error kind",
                                BStr::new(self.src),
                            );
                            let tke = <&QEntitiesUnexpectedTokenError>::try_from(&e).unwrap();
                            assert_eq!(
                                tke.kind(),
                                tk,
                                "parsing of {:?} returned unexpected token kind",
                                BStr::new(self.src),
                            );
                        }
                    };
                    assert_eq!(
                        e.location().expect("error kind must carry a location"),
                        &self.location,
                        "parsing of {:?} returned unexpected error location",
                        BStr::new(self.src),
                    );
                }
            }
        }
    }

    #[test]
    fn unterminated_c_style_comments() {
        fn expected_error(src: &[u8], location: QEntitiesParserLocation) -> ExpectedError {
            ExpectedError {
                src,
                kind: ExpectedErrorVariant::SimpleKind(
                    QEntitiesParseErrorKind::UnterminatedCStyleComment,
                ),
                location,
            }
        }

        let parse_opts = QEntitiesParseOptions::new().with_c_style_comments(true);
        [
            expected_error(
                br#"/*"#,
                QEntitiesParserLocation {
                    offset: 0,
                    line: 1,
                    column: 1,
                },
            ),
            expected_error(
                b"\n/*",
                QEntitiesParserLocation {
                    offset: 1,
                    line: 2,
                    column: 1,
                },
            ),
            expected_error(
                br#"/**"#,
                QEntitiesParserLocation {
                    offset: 0,
                    line: 1,
                    column: 1,
                },
            ),
            expected_error(
                br#"/***"#,
                QEntitiesParserLocation {
                    offset: 0,
                    line: 1,
                    column: 1,
                },
            ),
            expected_error(
                br#"{"k"/*"v"}"#,
                QEntitiesParserLocation {
                    offset: 4,
                    line: 1,
                    column: 5,
                },
            ),
            expected_error(
                br#"{"k" "v"}/*"#,
                QEntitiesParserLocation {
                    offset: 9,
                    line: 1,
                    column: 10,
                },
            ),
            expected_error(
                br#"{ k /* v }"#,
                QEntitiesParserLocation {
                    offset: 4,
                    line: 1,
                    column: 5,
                },
            ),
            expected_error(
                br#"{ k v }/*"#,
                QEntitiesParserLocation {
                    offset: 7,
                    line: 1,
                    column: 8,
                },
            ),
        ]
        .iter()
        .for_each(|ee| ee.test(&parse_opts));
    }

    #[test]
    fn unterminated_quoted_strings() {
        fn expected_error(src: &[u8], location: QEntitiesParserLocation) -> ExpectedError {
            ExpectedError {
                src,
                kind: ExpectedErrorVariant::SimpleKind(
                    QEntitiesParseErrorKind::UnterminatedQuotedString,
                ),
                location,
            }
        }

        let parse_opts = QEntitiesParseOptions::new();
        [
            expected_error(
                br#"{""#,
                QEntitiesParserLocation {
                    offset: 1,
                    line: 1,
                    column: 2,
                },
            ),
            expected_error(
                br#"{"key" ""#,
                QEntitiesParserLocation {
                    offset: 7,
                    line: 1,
                    column: 8,
                },
            ),
            expected_error(
                b"{k\n\"",
                QEntitiesParserLocation {
                    offset: 3,
                    line: 2,
                    column: 1,
                },
            ),
            expected_error(
                b"{k \"v}",
                QEntitiesParserLocation {
                    offset: 3,
                    line: 1,
                    column: 4,
                },
            ),
        ]
        .iter()
        .for_each(|ee| ee.test(&parse_opts));
    }

    #[test]
    fn unterminated_entities() {
        fn expected_error(src: &[u8], location: QEntitiesParserLocation) -> ExpectedError {
            ExpectedError {
                src,
                kind: ExpectedErrorVariant::SimpleKind(QEntitiesParseErrorKind::UnterminatedEntity),
                location,
            }
        }

        let parse_opts = QEntitiesParseOptions::new();
        [
            expected_error(
                b"{",
                QEntitiesParserLocation {
                    offset: 0,
                    line: 1,
                    column: 1,
                },
            ),
            expected_error(
                b"\n{",
                QEntitiesParserLocation {
                    offset: 1,
                    line: 2,
                    column: 1,
                },
            ),
            expected_error(
                b"{ k v }{",
                QEntitiesParserLocation {
                    offset: 7,
                    line: 1,
                    column: 8,
                },
            ),
            expected_error(
                b"{ k v }\n{",
                QEntitiesParserLocation {
                    offset: 8,
                    line: 2,
                    column: 1,
                },
            ),
        ]
        .iter()
        .for_each(|ee| ee.test(&parse_opts));
    }

    #[test]
    fn invalid_escape_sequences() {
        fn expected_error(src: &[u8], location: QEntitiesParserLocation) -> ExpectedError {
            ExpectedError {
                src,
                kind: ExpectedErrorVariant::SimpleKind(
                    QEntitiesParseErrorKind::InvalidEscapeSequence,
                ),
                location,
            }
        }

        let parse_opts = QEntitiesParseOptions::new()
            .with_escape_options(Some(QEntitiesParseEscapeOptions::new()));
        [
            expected_error(
                br#"{"\x" "value"}"#,
                QEntitiesParserLocation {
                    offset: 2,
                    line: 1,
                    column: 3,
                },
            ),
            expected_error(
                b"{k\n\"\\x\"}",
                QEntitiesParserLocation {
                    offset: 4,
                    line: 2,
                    column: 2,
                },
            ),
            expected_error(
                b"{\"\\\x00\" \"\"}",
                QEntitiesParserLocation {
                    offset: 2,
                    line: 1,
                    column: 3,
                },
            ),
        ]
        .iter()
        .for_each(|ee| ee.test(&parse_opts));
    }

    #[test]
    fn nested_entities() {
        fn expected_error(src: &[u8], location: QEntitiesParserLocation) -> ExpectedError {
            ExpectedError {
                src,
                kind: ExpectedErrorVariant::UnexpectedToken(QEntitiesTokenKind::OpenBrace),
                location,
            }
        }

        let parse_opts = QEntitiesParseOptions::new();
        [
            expected_error(
                b"{{",
                QEntitiesParserLocation {
                    offset: 1,
                    line: 1,
                    column: 2,
                },
            ),
            expected_error(
                b"\n{{",
                QEntitiesParserLocation {
                    offset: 2,
                    line: 2,
                    column: 2,
                },
            ),
            expected_error(
                b"{\n{",
                QEntitiesParserLocation {
                    offset: 2,
                    line: 2,
                    column: 1,
                },
            ),
            expected_error(
                b"\n{\n{",
                QEntitiesParserLocation {
                    offset: 3,
                    line: 3,
                    column: 1,
                },
            ),
            expected_error(
                b"{ k v { k v } k v }",
                QEntitiesParserLocation {
                    offset: 6,
                    line: 1,
                    column: 7,
                },
            ),
        ]
        .iter()
        .for_each(|ee| ee.test(&parse_opts));
    }

    #[test]
    fn unpaired_close_braces() {
        fn expected_error(src: &[u8], location: QEntitiesParserLocation) -> ExpectedError {
            ExpectedError {
                src,
                kind: ExpectedErrorVariant::UnexpectedToken(QEntitiesTokenKind::CloseBrace),
                location,
            }
        }

        let parse_opts = QEntitiesParseOptions::new();
        [
            expected_error(
                b"}",
                QEntitiesParserLocation {
                    offset: 0,
                    line: 1,
                    column: 1,
                },
            ),
            expected_error(
                b"\n}",
                QEntitiesParserLocation {
                    offset: 1,
                    line: 2,
                    column: 1,
                },
            ),
            expected_error(
                b"{}}",
                QEntitiesParserLocation {
                    offset: 2,
                    line: 1,
                    column: 3,
                },
            ),
            expected_error(
                b"{ k v }}",
                QEntitiesParserLocation {
                    offset: 7,
                    line: 1,
                    column: 8,
                },
            ),
        ]
        .iter()
        .for_each(|ee| ee.test(&parse_opts));
    }

    #[test]
    fn exterior_key_values() {
        fn expected_error(
            kind: QEntitiesTokenKind,
            src: &[u8],
            location: QEntitiesParserLocation,
        ) -> ExpectedError {
            ExpectedError {
                src,
                kind: ExpectedErrorVariant::UnexpectedToken(kind),
                location,
            }
        }

        let parse_opts = QEntitiesParseOptions::new();
        [
            expected_error(
                QEntitiesTokenKind::QuotedString,
                b"\"k\" \"v\"",
                QEntitiesParserLocation {
                    offset: 0,
                    line: 1,
                    column: 1,
                },
            ),
            expected_error(
                QEntitiesTokenKind::UnquotedString,
                b"k v",
                QEntitiesParserLocation {
                    offset: 0,
                    line: 1,
                    column: 1,
                },
            ),
            expected_error(
                QEntitiesTokenKind::QuotedString,
                b"{ k v }\n\"k\" \"v\"",
                QEntitiesParserLocation {
                    offset: 8,
                    line: 2,
                    column: 1,
                },
            ),
        ]
        .iter()
        .for_each(|ee| ee.test(&parse_opts));
    }

    #[test]
    fn comments() {
        #[rustfmt::skip]
        let data =
br#"// first line
{ k0 v0 //
}{ k1 /*c*/ v1 }
//c
{ /* c */ k2 v2 }//c
{ k3//c
v3/**c**/}
{ k4/**c**/ v4//
}//"#;

        let entities = QEntitiesParseOptions::new()
            .cpp_style_comments(true)
            .c_style_comments(true)
            .comments_terminate_unquoted_strings(true)
            .parse(&data[..])
            .unwrap();

        assert_eq!(entities.len(), 5);
        for (index, entity) in entities.iter().enumerate() {
            assert_eq!(entity.len(), 1);
            let (key, value) = entity.get(0).map(|kv| (kv.key(), kv.value())).unwrap();
            assert_eq!(key, format!("k{}", index).into_bytes());
            assert_eq!(value, format!("v{}", index).into_bytes());
        }
    }

    #[test]
    fn vtmb_entities() {
        #[rustfmt::skip]
        let data =
br#"// vtmb
{
"world_maxs" "4096 4096 4096"
"world_mins" "-4096 -4096 -4096"
"classname" "worldspawn"
"skyname" "thesky"
"sounds" "1"
"MaxRange" "1337"
"fogcolor" "255 255 255"
"fogcolor2" "255 255 255"
"fogdir" "0 1 0"
"fogstart" "123.0"
"fogend" "456.0"
"wetness_fadetarget" "0.11"
"wetness_fadein" "2.3"
"wetness_fadeout" "5.4"
"levelscript" "thescript"
"safearea" "2"
"nosferatu_tolerrant" "1"
}
{
"classname" "logic_relay"
"StartDisabled" "0"
"targetname" "relay_a"
"spawnflags" "1"
"OnTrigger" ",,,0,-1,ScriptFn(\"arg_a\", \"arg_b\"),"
"origin" "1 2 3"
}
{
"classname" "logic_relay"
"StartDisabled" "0"
"targetname" "relay_b"
"spawnflags" "1"
"OnTrigger" ",,,0,-1,ScriptFn(\"a\", \"b\", \"c\"),"
"origin" "4 5 6"
}"#;

        let expected_entities: &[&[(&[u8], &[u8])]] = &[
            &[
                (b"world_maxs", b"4096 4096 4096"),
                (b"world_mins", b"-4096 -4096 -4096"),
                (b"classname", b"worldspawn"),
                (b"skyname", b"thesky"),
                (b"sounds", b"1"),
                (b"MaxRange", b"1337"),
                (b"fogcolor", b"255 255 255"),
                (b"fogcolor2", b"255 255 255"),
                (b"fogdir", b"0 1 0"),
                (b"fogstart", b"123.0"),
                (b"fogend", b"456.0"),
                (b"wetness_fadetarget", b"0.11"),
                (b"wetness_fadein", b"2.3"),
                (b"wetness_fadeout", b"5.4"),
                (b"levelscript", b"thescript"),
                (b"safearea", b"2"),
                (b"nosferatu_tolerrant", b"1"),
            ],
            &[
                (b"classname", b"logic_relay"),
                (b"StartDisabled", b"0"),
                (b"targetname", b"relay_a"),
                (b"spawnflags", b"1"),
                (b"OnTrigger", b",,,0,-1,ScriptFn(\"arg_a\", \"arg_b\"),"),
                (b"origin", b"1 2 3"),
            ],
            &[
                (b"classname", b"logic_relay"),
                (b"StartDisabled", b"0"),
                (b"targetname", b"relay_b"),
                (b"spawnflags", b"1"),
                (b"OnTrigger", b",,,0,-1,ScriptFn(\"a\", \"b\", \"c\"),"),
                (b"origin", b"4 5 6"),
            ],
        ];

        let entities = QEntitiesParseOptions::vtmb().parse(&data[..]).unwrap();
        assert_eq!(
            entities.len(),
            expected_entities.len(),
            "entities length mismatch",
        );

        for (entity_index, (entity, expected_entity)) in entities
            .iter()
            .zip(expected_entities.iter().copied())
            .enumerate()
        {
            assert_eq!(
                entity.len(),
                expected_entity.len(),
                "entity #{entity_index} length mismatch",
            );

            for (kv_index, ((k, v), (expected_k, expected_v))) in entity
                .iter()
                .map(|kv| (kv.key(), kv.value()))
                .zip(expected_entity.iter().copied())
                .enumerate()
            {
                assert_eq!(
                    k,
                    expected_k,
                    "key #{kv_index} in entity #{entity_index} mismatch: {:?} != {:?}",
                    String::from_utf8_lossy(k),
                    String::from_utf8_lossy(expected_k),
                );
                assert_eq!(
                    v,
                    expected_v,
                    "value #{kv_index} in entity #{entity_index} mismatch: {:?} != {:?}",
                    String::from_utf8_lossy(v),
                    String::from_utf8_lossy(expected_v),
                );
            }
        }
    }
}
