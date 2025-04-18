/*
    Represents a variable, holding a vector of data points as either a strings, f32s or "missing".
*/

// use crate::models::parser::Parser;

use std::cmp::Ordering;
use std::fmt:: {
    Display,
    Formatter
};

#[derive(Default, PartialEq, Clone)]
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
                let decimals = match value.abs() {
                    0.001..0.1  => 3,
                    0.1..1.0    => 2,
                    1.0..100.0  => 1,
                    _ => 0
                };
                write!(formatter, "{:.decimals$}", *value)
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
        if  value.is_empty() { // trim() to?
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
            }
        }
    }

    // fn as_string (&mut self) {
    //     if let Value::Number { number } = self {
    //         *self = Value::String { string: number.to_string() };
    //     }
    // }

}

#[derive(Default)]
enum Mapping 
{
    Cluster {clusters: Vec<Range>},
    #[default] 
    Recode
}

#[derive(Default)]
pub struct Range 
{
    lower: Box<Value>, 
    upper: Box<Value>
}

impl Display for Range 
{
    fn fmt (&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} to {}", self.lower, self.upper)
    }
}

#[derive(Default)]
pub struct Variable
{
    name: String,
    values: Vec<Value>,
    backup: Vec<Value>,
    range: Range,
    mapping: Mapping
}

impl Variable
{
    pub fn new (name: &str) -> Self {
        Self { 
            name: name.to_string(),
            values: Vec::new(),
            backup: Vec::new(),
            mapping: Mapping::default(),
            range: Range::default()
        }
    }

    pub fn add_value (&mut self, value: &str) {
        let value = Value::new(value);
        if  *self.range.lower == Value::None || *self.range.lower > value {
            *self.range.lower = value.clone();
        }
        if  *self.range.upper == Value::None || *self.range.upper < value {
            *self.range.upper = value.clone();
        }
        self.values.push(value);
    }

    pub fn name (&self) -> &str {
        self.name.as_str()
    }

    pub fn range (&self) -> &Range {
        &self.range
    }

    pub fn as_numbers (&mut self) {
        if  self.backup.is_empty() {
            self.backup = self.values.clone();
        }
        self.values.iter_mut().for_each(|v| {
            v.as_number();
        });
        self.range.lower.as_number();
        self.range.upper.as_number();
    }

    pub fn as_strings (&mut self) {
        self.values = self.backup.clone();
        // self.range  = self.range_backup().clone()
        // self.range.lower.as_string();
        // self.range.upper.as_string();
    }

}
