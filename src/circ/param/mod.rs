use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "circ/param/expr.pest"]
pub struct ExprParser;

use std::{collections::HashMap, f64::consts::PI};


#[derive(Debug, derive_more::Display, thiserror::Error, Clone)]
pub enum NumericError {
    ParseError(String),
    DivisionByZero,
    InvalidArgument(String),
    UndefinedVariable(String),
}

pub fn evaluate_with_pi(expr: &str) -> Result<f64, NumericError> {
    evaluate(expr, &HashMap::from([("pi".to_owned(), PI)]))
}

pub fn evaluate(expr: &str, vars: &HashMap<String, f64>) -> Result<f64, NumericError> {
    let pairs =
        ExprParser::parse(Rule::exp, expr).map_err(|e| NumericError::ParseError(e.to_string()))?;

    let pair = pairs
        .peek()
        .ok_or_else(|| NumericError::ParseError("Empty expression".to_string()))?;
    eval_expr(pair.into_inner(), vars)
}

fn eval_expr(
    mut pair: pest::iterators::Pairs<Rule>,
    vars: &HashMap<String, f64>,
) -> Result<f64, NumericError> {
    let mut result = eval_term(pair.next().unwrap().into_inner(), vars)?;
    while let Some(op) = pair.next() {
        let n = pair.next().unwrap();
        let term = eval_term(n.into_inner(), vars)?;
        result = caculate(op.as_str(), result, term)?;
    }
    Ok(result)
}

fn eval_term(
    mut pair: pest::iterators::Pairs<Rule>,
    vars: &HashMap<String, f64>,
) -> Result<f64, NumericError> {
    let mut result = eval_factor(pair.next().unwrap().into_inner(), vars)?;
    while let Some(op) = pair.next() {
        let n = pair.next().unwrap();
        let factor = eval_factor(n.into_inner(), vars)?;
        result = caculate(op.as_str(), result, factor)?;
    }
    Ok(result)
}

fn eval_factor(
    mut pair: pest::iterators::Pairs<Rule>,
    vars: &HashMap<String, f64>,
) -> Result<f64, NumericError> {
    let mut result = eval_power(pair.next().unwrap().into_inner(), vars)?;
    while let Some(op) = pair.next() {
        let factor = eval_power(op.into_inner(), vars)?;
        result = caculate("^", result, factor)?;
    }
    Ok(result)
}

fn eval_power(
    mut pair: pest::iterators::Pairs<Rule>,
    vars: &HashMap<String, f64>,
) -> Result<f64, NumericError> {
    let inner = pair.next().unwrap();
    match inner.as_rule() {
        Rule::real => Ok(inner.as_str().parse::<f64>().unwrap()),
        Rule::nninteger => Ok(inner.as_str().parse::<f64>().unwrap()),
        Rule::id => {
            let id = inner.as_str();
            let Some(value) = vars.get(id) else {
                return Err(NumericError::UndefinedVariable(id.to_string()));
            };
            Ok(*value)
        }
        Rule::unary_op => {
            let op = inner.as_str();
            let inner = pair.next().unwrap();
            let value = eval_expr(inner.into_inner(), vars)?;
            Ok(caculate(op, value, 0.0)?)
        }
        Rule::power => Ok(-eval_power(inner.into_inner(), vars)?),
        Rule::exp => {
            let inner = inner.into_inner().next().unwrap();
            Ok(eval_expr(inner.into_inner(), vars)?)
        }
        _ => unreachable!(),
    }
}

fn caculate(op: &str, a: f64, b: f64) -> Result<f64, NumericError> {
    match op {
        "+" => Ok(a + b),
        "-" => Ok(a - b),
        "*" => Ok(a * b),
        "/" => {
            if b == 0.0 {
                Err(NumericError::DivisionByZero)
            } else {
                Ok(a / b)
            }
        }
        "^" => Ok(a.powf(b)),
        "sin" => Ok(a.sin()),
        "cos" => Ok(a.cos()),
        "tan" => Ok(a.tan()),
        "exp" => Ok(a.exp()),
        "ln" => Ok(a.ln()),
        "sqrt" => Ok(a.sqrt()),
        _ => Err(NumericError::InvalidArgument(op.to_string())),
    }
}
#[cfg(test)]
mod tests {
    use claim::assert_matches;

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_eval() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 2.0);
        vars.insert("y".to_string(), 3.0);

        assert_eq!(evaluate("  0 ", &vars).unwrap(), 0.0);
        assert_eq!(evaluate("-  x", &vars).unwrap(), -2.0);
        assert_eq!(evaluate("x + y", &vars).unwrap(), 5.0);
        assert_eq!(evaluate("x * y", &vars).unwrap(), 6.0);
        assert_eq!(evaluate("x^2 + y^2", &vars).unwrap(), 13.0);
        assert_eq!(evaluate("sin(x)^2 + cos(x)^2", &vars).unwrap(), 1.0);
        assert_eq!(evaluate("sqrt(x^2 + y^2 - y^2)", &vars).unwrap(), 2.0);

        // Test error cases
        assert_matches!(evaluate("x / 0", &vars), Err(NumericError::DivisionByZero));
        assert_matches!(
            evaluate("z", &vars),
            Err(NumericError::UndefinedVariable(_))
        );
        assert_matches!(
            evaluate("invalid", &vars),
            Err(NumericError::UndefinedVariable(_))
        );
    }
}
