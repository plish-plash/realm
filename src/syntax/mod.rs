pub mod math;
pub mod code;
pub mod lsystem;

use combine::{
    error::ParseError,
    parser::{
        combinator::no_partial,
        repeat::skip_many,
    },
    stream::Stream,
    stream::position::SourcePosition,
    Parser,
    EasyParser,
    one_of,
    optional,
    token,
};

use crate::code::{Evaluable, Value, VariableScope};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(combine::easy::Errors<char, String, SourcePosition>),
    Evaluate(code::SourceError),
}

type EasyStream<'a> = combine::easy::Stream<combine::stream::position::Stream<&'a str, SourcePosition>>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "{}", err),
            Error::Parse(ref err) => write!(f, "{}", err),
            Error::Evaluate(ref err) => write!(f, "{}", err),
        }
    }
}

pub fn parse_string<'a, P>(mut parser: P, text: &'a str) -> Result<P::Output, Error> where P: Parser<EasyStream<'a>> {
    parser
        .easy_parse(combine::stream::position::Stream::new(text))
        .map(|(output, _remaining)| output)
        .map_err(|err| Error::Parse(err.map_range(|s| s.to_string())))
}

pub fn parse_code_file<P: AsRef<std::path::Path>>(path: P, scope: VariableScope) -> Result<Vec<Value>, Error> {
    let text = std::fs::read_to_string(path).map_err(Error::Io)?;
    let code = parse_string(code::list_file(), &text)?;
    code.iter().map(|item| item.evaluate(scope).map_err(Error::Evaluate)).collect()
}

pub fn spaces<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    skip_many(one_of(" \t".chars())).expected("spaces")
}

pub fn newline<Input>() -> impl Parser<Input, Output = (), PartialState = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    no_partial((optional(token('\r')), token('\n'))).map(|_| ()).expected("newline")
}
