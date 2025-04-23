/*
    A state machine for parsing expression for categorizing values. Expression are on the form:

    <expressionSequence> :: <expression>[,<expression>...]
    <expression> :: {<stringExpression>|<numberExpression}
    <stringExpression> :: {<minimumOperator>|stringLiteral} to {<maximumOperator>|stringLiteral}
    <numberExpression> :: {<minimumOperator>|numberLiteral} to {<maximumOperator>|numberLiteral}
    <minimumOperator> :: {"min","low","lowest"}
    <maximumOperator> :: {"max","high","highest"}
    <stringLiteral> :: any sequence of characters with in quotation marks.
    <numberLiteral> :: any sequence of characters parseable as an f32.
    
    Example:
    low to 3.5, 3.5 to max
    "A" to "C", "D" to "E", "F" to max

    Lower range value is inclusive (>=).
    Whitespace between tokens is discarded.

*/

use std::str::Chars;

#[derive(Debug, PartialEq)]
enum State 
{
    MinimumOrValue,
    ValueRange,
    StringRange,
    NumberRange,
    MaximumOrValue,
    MaximumOrString,
    MaximumOrNumber
}

#[derive(Debug, PartialEq)]
pub enum Token
{
    Minimum,
    Maximum,
    Range,
    String {value: String},
    Number {value: f32}
}

pub struct Parser;
impl Parser
{
    // #[allow(clippy::too_many_lines)]
    pub fn parse (input: &str) -> Result<Vec<Token>, String> {
        let mut expect = State::MinimumOrValue;
        let mut source = input.chars();
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(text) = Self::next(&mut source) {
            match expect {
                State::MinimumOrValue => {
                    match text.to_lowercase().as_str() {
                        "min" | "lowest"  | "low"  => { 
                            tokens.push(Token::Minimum);
                            expect = State::ValueRange;
                        },
                        "max" | "highest" | "high" | "to" => {
                            return Err(format!("Unexpected keyword '{text}' (string, number or 'min' expected)."))
                        }
                        _ => {
                            if text.starts_with('@') { // A quoted string.
                                // text.remove(0);
                                tokens.push(Token::String { value: text.strip_prefix('@').unwrap_or_default().to_string() });
                                expect = State::StringRange;
                                continue;
                            }
                            if let Ok(number) = text.parse::<f32>() {
                                tokens.push(Token::Number { value: number });
                                expect = State::NumberRange;
                                continue;
                            }
                            return Err(format!("Unexpected lower range value '{text}' (a string or a valid number expected)."))
                        }
                    }
                },
                State::ValueRange | State::StringRange | State::NumberRange => {
                    match text.to_lowercase().as_str() {
                        "to" => { 
                            tokens.push(Token::Range);
                            expect = match expect {
                                State::StringRange => State::MaximumOrString,
                                State::NumberRange => State::MaximumOrNumber,
                                _ => State::MaximumOrValue
                            };
                        }
                        _ => {
                            return Err(String::from("Separate range values with keyword 'to'."))
                        }
                    }
                },
                State::MaximumOrValue | State::MaximumOrString | State::MaximumOrNumber => {
                    match text.to_lowercase().as_str() {
                        "max" | "highest" | "high"  => { 
                            tokens.push(Token::Maximum);
                            expect = State::MinimumOrValue;
                        },
                        "min" | "lowest" | "low" | "to" => {
                            return Err(format!("Unexpected keyword '{text}' (string, number or 'max' expected)."))
                        }
                        _ => {
                            if text.starts_with('@') { // A quoted string.
                                if expect == State::MaximumOrNumber {
                                    return Err(String::from("Both lower and upper range must be strings."))
                                }
                                tokens.push(Token::String { value: text.strip_prefix('@').unwrap_or_default().to_string() });
                                expect = State::MinimumOrValue;
                                continue;
                            }
                            if let Ok(number) = text.parse::<f32>() {
                                if expect == State::MaximumOrString {
                                    return Err(String::from("Both lower and upper range must be valid numbers."))
                                }
                                tokens.push(Token::Number { value: number });
                                expect = State::MinimumOrValue;
                                continue;
                            }
                            return Err(format!("Unexpected upper range value '{text}' (a string or a valid number expected)."))
                        }
                    }
                }
            }
        }
        Ok(tokens)
    }

    pub fn next (input: &mut Chars<'_>) -> Option<String> {
        let mut quote = false; // If within quotation.
        let mut value = String::new();
        while let Some(character) = Iterator::next(input) {
            match character {
                '"' => {
                    if quote {
                        return Some(format!("@{value}")) // Always a string.
                    }
                    quote = true; 
                }
                ' ' | ',' => {
                    if quote { 
                        value.push(character);
                        continue;
                    }
                    if !value.is_empty() {
                        return Some(value)
                    }
                },
                _ => {
                    value.push(character);
                }
            }
        }
        if value.is_empty() || quote { // Invalid if still in quotation.
            return None 
        } 
        Some(value)
    }

}
