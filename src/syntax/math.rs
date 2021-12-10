use std::str::FromStr;

use combine::{
    *,
    parser::{
        char::{digit, letter},
        choice::optional,
        combinator::no_partial,
        range::recognize,
        repeat::{skip_many, skip_many1},
        token::token,
    },
};
use super::spaces;
use crate::code::*;

#[derive(Debug)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum CompareOperator {
    Less,
    LessEqual,
    Equal,
    NotEqual,
    GreaterEqual,
    Greater,
}

impl FromStr for CompareOperator {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "=" | "==" => Ok(CompareOperator::Equal),
            "!" | "!=" => Ok(CompareOperator::NotEqual),
            "<" => Ok(CompareOperator::Less),
            "<=" => Ok(CompareOperator::LessEqual),
            ">" => Ok(CompareOperator::Greater),
            ">=" => Ok(CompareOperator::GreaterEqual),
            _ => Err("invalid comparison operator".to_string()),
        }
    }
}

#[derive(Debug)]
pub enum ExpressionTerm {
    Variable(Variable),
    Number(f64),
    Expression(Box<Expression>),
}

impl Evaluable for ExpressionTerm {
    type Output = f64;
    fn evaluate(&self, scope: VariableScope) -> f64 {
        match self {
            ExpressionTerm::Variable(var) => scope.get(var).expect("unknown variable in expression"),
            ExpressionTerm::Number(value) => *value,
            ExpressionTerm::Expression(expr) => expr.evaluate(scope),
        }
    }
}

#[derive(Debug)]
pub struct Expression {
    left: ExpressionTerm,
    op: Operator,
    right: ExpressionTerm,
}

impl Evaluable for Expression {
    type Output = f64;
    fn evaluate(&self, scope: VariableScope) -> f64 {
        let left = self.left.evaluate(scope);
        let right = self.right.evaluate(scope);
        match self.op {
            Operator::Add => left + right,
            Operator::Subtract => left - right,
            Operator::Multiply => left * right,
            Operator::Divide => left / right,
        }
    }
}

pub fn number<'a, I>() -> impl Parser<I, Output = f64>
where
    I: RangeStream<Token = char, Range = &'a str>,
{
    from_str(recognize((
        skip_many1(digit()),
        optional((token('.'), skip_many(digit()))),
    )))
}

pub fn variable<I>() -> impl Parser<I, Output = Variable>
where
    I: Stream<Token = char>,
{
    many1(letter())
}

pub fn operator<I>() -> impl Parser<I, Output = Operator>
where
    I: Stream<Token = char>,
{
    choice!(
        token('+').map(|_| Operator::Add),
        token('-').map(|_| Operator::Subtract),
        token('*').map(|_| Operator::Multiply),
        token('/').map(|_| Operator::Divide)
    )
}

pub fn compare_operator<'a, I>() -> impl Parser<I, Output = CompareOperator>
where
    I: RangeStream<Token = char, Range = &'a str>,
{
    from_str(recognize(one_of("=<>!".chars()).and(optional(token('=')))))
}

pub fn expression_term<'a, I>() -> impl Parser<I, Output = ExpressionTerm>
where
    I: RangeStream<Token = char, Range = &'a str>,
{
    choice!(
        variable().map(|v| ExpressionTerm::Variable(v)),
        number().map(|n| ExpressionTerm::Number(n)),
        between(token('('), token(')'), expression())
    )
}

pub fn expression<'a, I>() -> impl Parser<I, Output = ExpressionTerm>
where
    I: RangeStream<Token = char, Range = &'a str>,
{
    opaque!(no_partial(
        expression_term().skip(spaces()).and(optional(
            operator().skip(spaces()).and(
                expression_term())))
    ).map(|(left, option)| {
        if let Some((op, right)) = option {
            ExpressionTerm::Expression(Box::new(Expression { left, op, right }))
        } else {
            left
        }
    }))
}
