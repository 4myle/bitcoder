/*
    Represents a variable, holding a vector of data points as either a strings, f32s or "missing".
*/
 
use crate::models::parser::Token;

use std::cmp::Ordering;
use std::fmt:: {
    Display,
    Formatter
};

#[derive(Debug, Default, PartialEq, Clone)]
pub enum Value 
{
    String { string: String },
    Number { number: f32 },
    #[default] 
    None
}

impl Display for Value 
{
    fn fmt (&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => write!(formatter, "(missing)"),
            Value::String { string: value } => write!(formatter, "\u{201c}{}\u{201d}", *value),
            Value::Number { number: value } => {
                // Method of three significant figures.
                // let decimals = match value.abs() {
                //     0.001..0.1  => 3,
                //     0.1..1.0    => 2,
                //     1.0..100.0  => 1,
                //     _ => 0
                // };
                // write!(formatter, "{:.decimals$}", *value)
                // Remove trailing zeroes after rounded to three decimals.
                write!(formatter, "{}", format!("{value:.3}").trim_end_matches('0').trim_end_matches('.'))
            }
        }
    }
}

impl PartialOrd for Value 
{
    fn partial_cmp (&self, other: &Self) -> Option<Ordering> {
        match self {
            Value::String { string: s1 } => {
                match other {
                    Value::String { string: s2 } => s1.partial_cmp(s2),
                    Value::Number { number: _  } => None,
                    Value::None if *other == Value::None => Some(Ordering::Equal),
                    Value::None => Some(Ordering::Greater),
                }
            }
            Value::Number { number: n1 } => {
                match other {
                    Value::String { string: _  } => None,
                    Value::Number { number: n2 } => n1.partial_cmp(n2),
                    Value::None if *other == Value::None => Some(Ordering::Equal),
                    Value::None => Some(Ordering::Greater),
                }
            }
            Value::None if *other == Value::None => Some(Ordering::Equal),
            Value::None => Some(Ordering::Less)
        }
    }
}

impl Value 
{
    fn new (value: &str) -> Self {
        if  value.is_empty() {
            Value::None
        } else {
            Value::String { string: value.to_owned() }
        }
    }

    fn as_number (&mut self) {
        if let Value::String { string } = self {
            let value = string.replace(&[' ','\t','%'][..], "").replace(',', ".");
            if let Ok(number) = value.parse::<f32>() {
                *self = Value::Number { number };
            } else {
                *self = Value::None;
            }
        }
    }

}

#[derive(Debug, Default)]
pub struct Range 
{
    lower: Value, 
    upper: Value
}

#[derive(Debug, Default)]
enum Mapping 
{
    Cluster {clusters: Vec<Range>},
    #[default] 
    Recode
}

#[derive(Default)]
pub struct Variable
{
    name: String,
    values: Vec<Value>,
    backup: Vec<Value>,
    missing: usize,
    minimum: Value,
    maximum: Value,
    mapping: Mapping,
    is_numeric: bool
}

impl Variable
{
    pub fn new (name: &str) -> Self {
        Self { 
            name: name.to_string(),
            values: Vec::new(),
            backup: Vec::new(),
            missing: 0,
            minimum: Value::None,
            maximum: Value::None,
            mapping: Mapping::default(),
            is_numeric: false
        }
    }

    pub fn set_expression (&mut self, tokens: &[Token]) -> Result<(), &'static str> {
        let mut ranges = Vec::<Range>::new();
        let mut tokens = tokens.iter();
        self.mapping = Mapping::Cluster {clusters: Vec::new()};
        loop {
            let mut range = Range::default();
            let value1st = tokens.next();
            let operator = tokens.next();
            let value2nd = tokens.next();
            if  value1st.is_none() {
                break;
            }
            match value1st {
                Some(Token::Number { value }) => range.lower = Value::Number { number: *value },
                Some(Token::String { value }) => range.lower = Value::String { string: value.as_str().to_string() },
                _ => return Err("Type a value for the lower range.")
            }
            if operator.is_none() || operator != Some(&Token::Range) {
                return Err("Separate range values with keyword ' to '.")
            }
            match value2nd {
                Some(Token::Number { value }) => range.upper = Value::Number { number: *value },
                Some(Token::String { value }) => range.upper = Value::String { string: value.as_str().to_string() },
                _ => return Err("Type a value for the upper range.")
            }
            // println!("{range:?}");
            ranges.push(range);
        }
        println!("{ranges:?}");
        let all_numbers = ranges.iter().all(|r| matches!(r.lower, Value::Number{..}) && matches!(r.upper, Value::Number{..}));
        let all_strings = ranges.iter().all(|r| matches!(r.lower, Value::String{..}) && matches!(r.upper, Value::String{..}));
        if self.is_numeric && !all_numbers || !self.is_numeric && !all_strings {
            return Err("All values must be of the same type and must match the variable type.")
        }
        self.mapping = Mapping::Cluster { clusters: ranges };
        Ok(())
    }

    // Associated function instead of "method" to avoid (unsolvable?) borrow checker issues with self. 
    fn set_residuals (value: &Value, missing: &mut usize, minimum: &mut Value, maximum: &mut Value) {
        if *value == Value::None {
            *missing += 1;
        }
        if  *minimum == Value::None || *minimum > *value {
            *minimum = value.clone();
        }
        if  *maximum == Value::None || *maximum < *value {
            *maximum = value.clone();
        }
    }

    pub fn add_value (&mut self, value: &str) {
        self.values.push(Value::new(value));
        if let Some(value) = self.values.last() {
            Variable::set_residuals(value, &mut self.missing, &mut self.minimum, &mut self.maximum);
        }
    }

    pub fn as_numbers (&mut self) {
        self.is_numeric = true;
        self.backup = self.values.clone();
        self.minimum = Value::None;
        self.maximum = Value::None;
        self.missing = 0;
        for value in &mut self.values {
            value.as_number();
            Variable::set_residuals(value, &mut self.missing, &mut self.minimum, &mut self.maximum);
        }
    }

    pub fn as_strings (&mut self) {
        self.is_numeric = false;
        self.values = self.backup.clone();
        self.backup.clear();
        self.minimum = Value::None;
        self.maximum = Value::None;
        self.missing = 0;
        for value in &mut self.values {
            Variable::set_residuals(value, &mut self.missing, &mut self.minimum, &mut self.maximum);
        }
    }

    pub fn name (&self) -> &str {
        self.name.as_str()
    }

    pub fn missing (&self) -> usize {
        self.missing
    }

    pub fn minimum (&self) -> &Value {
        &self.minimum
    }

    pub fn maximum (&self) -> &Value {
        &self.maximum
    }

}

