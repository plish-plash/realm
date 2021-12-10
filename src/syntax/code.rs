use combine::{
    *,
    parser::{
        char::{space, spaces},
        combinator::no_partial,
        repeat::{skip_many1},
        token::token,
    }, stream::position::SourcePosition,
};
use super::math::number;

#[derive(Debug)]
pub enum Error {
    EmptyList,
    InvalidFunctionName,
    UnknownVariable { unexpected: String },
    UnknownFunction { unexpected: String },
    NotEnoughArguments { function: String, expected: usize, unexpected: usize },
    UnexpectedTerm { function: String, argument: usize, expected: &'static str, unexpected: String },
    UnexpectedValue { function: String, argument: usize, expected: &'static str, unexpected: String },
}

#[derive(Debug)]
pub struct SourceError {
    position: SourcePosition,
    error: Error,
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Parse error at {}", self.position)?;
        match &self.error {
            Error::EmptyList => writeln!(f, "Empty list not allowed here"),
            Error::InvalidFunctionName => writeln!(f, "Expected function name"),
            Error::UnknownVariable { unexpected } => writeln!(f, "Unknown variable \"{}\"", unexpected),
            Error::UnknownFunction { unexpected } => writeln!(f, "Unknown function \"{}\"", unexpected),
            Error::NotEnoughArguments { function, expected, unexpected } =>
                writeln!(f, "Not enough arguments to \"{}\" (need {}, found {})", function, expected - 1, unexpected - 1),
            Error::UnexpectedTerm { function, argument, expected, unexpected } =>
                writeln!(f, "Unexpected {}\nExpected {} for argument {} of \"{}\"", unexpected, expected, argument, function),
            Error::UnexpectedValue { function, argument, expected, unexpected } =>
                writeln!(f, "Unexpected {} value\nExpected {} value for argument {} of \"{}\"", unexpected, expected, argument, function),
        }
    }
}

impl SourceError {
    pub fn empty_list(position: SourcePosition) -> SourceError {
        SourceError { position, error: Error::EmptyList }
    }
    pub fn invalid_function_name(position: SourcePosition) -> SourceError {
        SourceError { position, error: Error::InvalidFunctionName }
    }
    pub fn unknown_variable(position: SourcePosition, unexpected: &str) -> SourceError {
        SourceError {
            position,
            error: Error::UnknownVariable { unexpected: unexpected.to_owned() },
        }
    }
    pub fn unknown_function(position: SourcePosition, unexpected: &str) -> SourceError {
        SourceError {
            position,
            error: Error::UnknownFunction { unexpected: unexpected.to_owned() },
        }
    }
    pub fn not_enough_arguments(position: SourcePosition, function: &List, expected: usize) -> SourceError {
        SourceError {
            position,
            error: Error::NotEnoughArguments { function: function.argument(0).function().to_owned(), expected, unexpected: function.len() },
        }
    }
    pub fn unexpected_term(argument: &SourceListArgument, expected: &'static str, unexpected: String) -> SourceError {
        SourceError {
            position: argument.term().source_position(),
            error: Error::UnexpectedTerm { function: argument.function().to_owned(), argument: argument.argument, expected, unexpected },
        }
    }
    pub fn unexpected_value(argument: &SourceListArgument, expected: &'static str, unexpected: &crate::code::Value) -> SourceError {
        SourceError {
            position: argument.term().source_position(),
            error: Error::UnexpectedValue { function: argument.function().to_owned(), argument: argument.argument, expected, unexpected: unexpected.kind().to_owned() },
        }
    }
}

#[derive(Debug)]
pub enum ListTerm {
    Identifier(String),
    Number(f64),
    List(Box<List>),
}

#[derive(Debug)]
pub struct SourceListTerm {
    position: SourcePosition,
    pub term: ListTerm,
}

impl SourceListTerm {
    pub fn source_position(&self) -> SourcePosition {
        self.position
    }
    pub fn into_literal(&self) -> Option<&str> {
        match &self.term {
            ListTerm::Identifier(ident) => Some(ident),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct List(Vec<SourceListTerm>);

impl List {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn argument(&self, index: usize) -> SourceListArgument {
        SourceListArgument { list: self, argument: index }
    }
}

#[derive(Clone)]
pub struct SourceListArgument<'a> {
    list: &'a List,
    argument: usize,
}

impl<'a> SourceListArgument<'a> {
    fn function(&self) -> &str {
        self.list.0[0].into_literal().unwrap()
    }
    pub fn term(&self) -> &SourceListTerm {
        &self.list.0[self.argument]
    }
    
    pub fn into_number(&self) -> Result<f64, SourceError> {
        match &self.term().term {
            ListTerm::Identifier(ident) => Err(SourceError::unexpected_term(self, "number", format!("`{}`", ident))),
            ListTerm::Number(num) => Ok(*num),
            ListTerm::List(_) => Err(SourceError::unexpected_term(self, "number", "list".to_string())),
        }
    }
}

impl std::ops::Index<usize> for List {
    type Output = SourceListTerm;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

pub fn spaces1<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    skip_many1(space()).expected("whitespaces")
}

pub fn identifier<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(satisfy(|ch: char| !ch.is_whitespace() && ch != '(' && ch != ')')).expected("identifier")
}

fn list_term<'a, I>() -> impl Parser<I, Output = SourceListTerm>
where
    I: RangeStream<Token = char, Range = &'a str, Position = SourcePosition>,
{
    position().and(
        choice!(
            number().map(|num| ListTerm::Number(num)),
            list().map(|list| ListTerm::List(list)),
            identifier().map(|ident| ListTerm::Identifier(ident))
        )
    ).map(|(position, term)| SourceListTerm { position, term })
}

pub fn list<'a, I>() -> impl Parser<I, Output = Box<List>>
where
    I: RangeStream<Token = char, Range = &'a str, Position = SourcePosition>,
{
    opaque!(no_partial(
        between(token('(').skip(spaces()), spaces().with(token(')')), sep_by(list_term(), spaces1()))
    ).map(|list| Box::new(List(list))))
}

pub fn list_file<'a, I>() -> impl Parser<I, Output = Vec<SourceListTerm>>
where
    I: RangeStream<Token = char, Range = &'a str, Position = SourcePosition>,
{
    spaces().with(many1(list_term().skip(spaces())))
}
