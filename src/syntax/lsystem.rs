use combine::{*, parser::{char::string, combinator::Either}};

use super::{spaces, newline};
use super::math::{compare_operator, number, variable, expression, ExpressionTerm, CompareOperator};
use crate::code::*;

#[derive(Debug)]
pub struct Symbol {
    pub symbol: char,
    pub params: Option<Vec<ExpressionTerm>>,
}

impl Evaluable for Symbol {
    type Output = crate::lsystem::LSymbol;
    fn evaluate(&self, scope: VariableScope) -> Self::Output {
        let params = self.params.as_ref().map(|params| {
            params.iter().map(|p| p.evaluate(scope)).collect()
        }).unwrap_or_default();
        crate::lsystem::LSymbol::new_params(self.symbol, params)
    }
}

#[derive(Debug)]
pub struct ProductionSymbol {
    pub symbol: char,
    pub params: Option<Vec<Variable>>,
}

pub type SymbolString = Vec<Symbol>;

#[derive(Debug)]
pub struct Condition {
    left: Variable,
    op: CompareOperator,
    right: f64,
}

impl Evaluable for Condition {
    type Output = bool;
    fn evaluate(&self, scope: VariableScope) -> bool {
        let left = scope.get(&self.left).expect("unknown variable in comparison");
        match self.op {
            CompareOperator::Less =>            left <  self.right,
            CompareOperator::LessEqual =>       left <= self.right,
            CompareOperator::Equal =>           left == self.right,
            CompareOperator::NotEqual =>        left != self.right,
            CompareOperator::GreaterEqual =>    left >= self.right,
            CompareOperator::Greater =>         left >  self.right,
        }
    }
}

impl Evaluable for Option<Vec<Condition>> {
    type Output = bool;
    fn evaluate(&self, scope: VariableScope) -> bool {
        match self {
            Some(conds) => {
                for cond in conds.iter() {
                    if !cond.evaluate(scope) {
                        return false;
                    }
                }
                true
            }
            None => true,
        }
    }
}

#[derive(Debug)]
pub struct Constant {
    left: Variable,
    right: f64,
}

#[derive(Debug)]
pub struct Production {
    pub predecessor: ProductionSymbol,
    pub conditions: Option<Vec<Condition>>,
    pub successor: SymbolString,
}

#[derive(Default, Debug)]
pub struct System {
    pub constants: VariableMap,
    pub productions: Vec<Production>,
}

impl Extend<Either<Constant, Production>> for System {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=Either<Constant, Production>> {
        for item in iter {
            match item {
                Either::Left(constant) => { self.constants.insert(constant.left, constant.right); }
                Either::Right(production) => self.productions.push(production),
            }
        }
    }
}

pub fn symbol_name<Input>() -> impl Parser<Input, Output = char, PartialState = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    satisfy(|ch: char| !ch.is_whitespace()).expected("symbol name")
}

fn symbol<'a, I>() -> impl Parser<I, Output = Symbol>
where
    I: RangeStream<Token = char, Range = &'a str>,
{
    let params = sep_by1(expression(), token(',').skip(spaces()));
    symbol_name().and(optional(between(token('('), token(')'), params)))
        .map(|(symbol, params)| Symbol { symbol, params })
}

fn symbol_string<'a, I>() -> impl Parser<I, Output = SymbolString>
where
    I: RangeStream<Token = char, Range = &'a str>,
{
    sep_by1(symbol(), skip_many1(one_of(" \t".chars())))
}

fn condition<'a, I>() -> impl Parser<I, Output = Condition>
where
    I: RangeStream<Token = char, Range = &'a str>,
{
    (
        variable().skip(spaces()),
        compare_operator().skip(spaces()),
        number(),
    ).map(|(left, op, right)| Condition { left, op, right })
}

pub fn production<'a, I>() -> impl Parser<I, Output = Production>
where
    I: RangeStream<Token = char, Range = &'a str>,
{
    let params = sep_by1(variable(), token(',').skip(spaces()));
    let predecessor = symbol_name().and(optional(between(token('('), token(')'), params)))
        .map(|(symbol, params)| ProductionSymbol { symbol, params });
    let conditions = token(':').skip(spaces()).with(sep_by1(condition(), token(',').skip(spaces())));
    (
        predecessor.skip(spaces()),
        optional(conditions.skip(spaces())),
        string("=>").skip(spaces()),
        symbol_string(),
    ).map(|(predecessor, conditions, _, successor)| Production { predecessor, conditions, successor })
}

pub fn constant<'a, I>() -> impl Parser<I, Output = Constant>
where
    I: RangeStream<Token = char, Range = &'a str>,
{
    (
        token('#').skip(spaces()),
        variable().skip(spaces()),
        token('=').skip(spaces()),
        number(),
    ).map(|(_, left, _, right)| Constant { left, right })
}

pub fn system<'a, I>() -> impl Parser<I, Output = System>
where
    I: RangeStream<Token = char, Range = &'a str>,
{
    let eol = spaces().with(newline());
    let line = spaces().with(
        constant().map(|c| Either::Left(c)).or(production().map(|p| Either::Right(p))));
    sep_by1(line, skip_many1(eol))
}

