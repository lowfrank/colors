//! The [`Highligher`] takes in a stream of characters and returns a stream of
//! [`Token`]s. Each token has a type, and the type determines the color it will
//! have in the IDE.

use std::collections::VecDeque;

/// Same naming conventions used in betty
type Int = i64;

/// Same naming conventions used in betty
type Float = f64;

/// A [`Token`] is composed of a type and of its literal value
pub struct Token(pub TokenType, pub String);

/// All the different [`Token`] types that a text can be divided into. Each token has
/// a color that is used when drawing text in the code editor. Each color can be
/// modified by the used in the `settings.json` file.
pub enum TokenType {
    Ident,
    Num,
    Str,
    Sym, // Symbol
    Kw,
    BuiltinFun,
    Fun,
    Comment,
    Error,
    Other,
}

/// The struct responsible for the analysis of the code editor text. It receives
/// as input the text as a sequence of characters and returns a sequence of tokens
/// out of it.
pub struct Highligher {
    source: VecDeque<char>,
    current_char: Option<char>,
}

impl Highligher {
    // betty reserved keywords
    const KEYWORDS: [&'static str; 25] = [
        "and", "or", "not", "if", "else", "do", "end", "for", "foreach", "while", "fun",
        "continue", "break", "return", "match", "try", "catch", "in", "throw", "using", "as",
        "true", "false", "nothing", "newerror",
    ];

    // betty builtin errors that can be catched and thrown
    const ERRORS: [&'static str; 11] = [
        "ValueError",
        "TypeError",
        "UnknownIdentifierError",
        "OverflowError",
        "DivisionByZeroError",
        "IndexOutOfBoundsError",
        "FileIOError",
        "VectorMutationError",
        "ModuleImportError",
        "AssertionError",
        "WrongArgumentsNumberError",
    ];

    // betty builtin functions
    const BUILTIN_FUNCTIONS: [&'static str; 42] = [
        "print",
        "println",
        "read_line",
        "to_int",
        "to_float",
        "to_str",
        "vpush_back",
        "vpush_front",
        "vpush_at",
        "vpop_front",
        "vpop_back",
        "vpop_at",
        "vfrom_range",
        "vcopy",
        "str_starts_with",
        "str_ends_with",
        "str_is_lowercase",
        "str_is_uppercase",
        "str_to_lowercase",
        "str_to_uppercase",
        "len",
        "get",
        "join",
        "slice",
        "split",
        "replace",
        "fread",
        "fwrite",
        "fappend",
        "err_short",
        "err_traceback",
        "err_kind",
        "err_line",
        "err_description",
        "assert",
        "isint",
        "isfloat",
        "isstr",
        "isbool",
        "isvec",
        "iscallable",
        "iserr",
    ];

    /// Valid simbols in betty. Some are commented out because I want to pass them as
    /// [`TokenType::Other`] color
    const SYMBOLS: [char; 12] = [
        '+', '-', '*', '/', '^', '%', ':', /*'(', ')', '[', ']',*/ '=', '>', '<',
        '!', /* '.', */
        /*',', */ '?',
    ];
}

impl Highligher {
    #[inline]
    pub fn new(source: String) -> Self {
        let mut source = source.chars().collect::<VecDeque<_>>();
        let current_char = source.pop_front();
        Self {
            source,
            current_char,
        }
    }

    /// Advance to the text character of the stream by removing the first character
    /// from it.
    #[inline]
    pub fn advance(&mut self) {
        self.current_char = self.source.pop_front();
    }

    /// Checks whether the next character is 'ch' in the stream of character.
    /// This does not consider spaces and tabs.
    #[inline]
    fn next_is(&mut self, ch: char) -> bool {
        if matches!(self.current_char, Some(c) if c == ch) {
            return true;
        }

        let first = self.source.iter().find(|ch| ![' ', '\t'].contains(ch));
        match first {
            Some(first) => first == &ch,
            None => false,
        }
    }

    /// Create a new [`Token`] that MAY be an identifier: indeed, if the [`String`]
    /// is one of:
    ///     - Reserved keyword
    ///     - Builtin function
    ///     - Type
    ///     - Error
    /// then its type will be that one.
    #[inline]
    fn make_ident(&mut self) -> Token {
        let mut ident = String::new();

        // Loop as long as we find a valid identifier character.
        loop {
            match self.current_char {
                Some(ch) if matches!(ch, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9') => {
                    ident.push(ch);
                    self.advance();
                }
                _ => break,
            }
        }

        let ident_str = &ident.as_str();
        let typ = if Self::KEYWORDS.contains(ident_str) {
            TokenType::Kw
        } else if Self::BUILTIN_FUNCTIONS.contains(ident_str) {
            TokenType::BuiltinFun
        } else if Self::ERRORS.contains(ident_str) {
            TokenType::Error
        } else if self.next_is('(') {
            TokenType::Fun
        } else {
            TokenType::Ident
        };
        Token(typ, ident)
    }

    /// Create a new [`Token`] of type [`TokenType::Num`] (integer or real doesn't matter
    /// because their color will be the very same). Follow the betty convention
    /// of numbers, where a user can type an undefined number of underscores, because
    /// they will be ignored by the lexer.
    #[inline]
    fn make_num(&mut self) -> Token {
        let mut num = String::new();

        loop {
            match self.current_char {
                Some(ch) if matches!(ch, '0'..='9' | '.' | '_') => {
                    num.push(ch);
                    self.advance();
                }
                _ => break,
            }
        }

        if num.replace('_', "").parse::<Int>().is_ok()
            || num.replace('_', "").parse::<Float>().is_ok()
        {
            Token(TokenType::Num, num)
        } else {
            Token(TokenType::Other, num)
        }
    }

    /// Create a new [`Token`] of type [`String`]. Loop as long as we dont't find a '"'
    /// or EOF. In that case return the [`Token`].
    #[inline]
    fn make_str(&mut self) -> Token {
        let mut string = String::from('"');
        self.advance(); // skip '"', otherwise we would not enter the loop

        loop {
            match self.current_char {
                Some(ch) if ch != '"' => {
                    string.push(ch);
                    self.advance();
                }
                Some(ch) if ch == '"' => {
                    string.push('"');
                    self.advance();
                    return Token(TokenType::Str, string);
                }
                _ => return Token(TokenType::Str, string),
            }
        }
    }

    /// Make a [`Token`] of type [`TokenType::Comment`]. It starts with the pipe operator, and
    /// are single line only. Therefore, we loop as long as we don't find a newline
    /// or EOF.
    #[inline]
    fn make_comment(&mut self) -> Token {
        let mut comment = String::from("|");
        self.advance(); // skip '|'

        loop {
            match self.current_char {
                Some(ch) if ch != '\n' => {
                    comment.push(ch);
                    self.advance();
                }
                _ => return Token(TokenType::Comment, comment),
            }
        }
    }

    /// Make a [`Token`] of type [`TokenType::Sym`] if the character is a valid betty symbol,
    /// otherwise the type will be [`TokenType::Other`].
    #[inline]
    fn make_sym_or_other(&mut self, ch: char) -> Token {
        let typ = if Self::SYMBOLS.contains(&ch) {
            TokenType::Sym
        } else {
            TokenType::Other
        };

        self.advance(); // Skip the character
        Token(typ, ch.into())
    }

    /// Main function, loop over all the characters and turn them into [`Token`]s, then
    /// return them when there are no more characters.
    #[inline]
    pub fn make_tokens(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.current_char {
            let token = match ch {
                'a'..='z' | 'A'..='Z' | '_' => self.make_ident(),
                '0'..='9' => self.make_num(),
                '"' => self.make_str(),
                '|' => self.make_comment(),
                _ => self.make_sym_or_other(ch),
            };
            tokens.push(token);
        }
        tokens
    }
}
