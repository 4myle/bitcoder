/*
    Represents a variable, holding a vector of data points as either a strings, f32s or "missing".
*/

use crate::models::parser::Token;

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

#[derive(Default)]
pub struct Range 
{
    lower: Value, 
    upper: Value
}

#[derive(Default)]
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
    mapping: Mapping
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
        }
    }

    pub fn add_value (&mut self, value: &str) {
        self.values.push(Value::new(value));
        if let Some(value) = self.values.last() {
            if *value == Value::None {
                self.missing += 1;
            }
            if  self.minimum == Value::None || self.minimum > *value {
                self.minimum = value.clone();
            }
            if  self.maximum == Value::None || self.maximum < *value {
                self.maximum = value.clone();
            }
        }
    }

    pub fn set_expression(&mut self, tokens: &Vec<Token>) { //TODO: return Result<(), &static str>?
        self.mapping = Mapping::Recode;
        println!("{tokens:?}");
    }

    pub fn as_numbers (&mut self) {
        self.backup = self.values.clone();
        self.minimum = Value::None;
        self.maximum = Value::None;
        self.missing = 0;
        self.values.iter_mut().for_each(|value| {
            value.as_number();
            if *value == Value::None {
                self.missing += 1;
            }
            if  self.minimum == Value::None || self.minimum > *value {
                self.minimum = value.clone();
            }
            if  self.maximum == Value::None || self.maximum < *value {
                self.maximum = value.clone();
            }
        });
    }

    pub fn as_strings (&mut self) {
        self.values = self.backup.clone();
        self.backup.clear();
        self.minimum = Value::None;
        self.maximum = Value::None;
        self.missing = 0;
        self.values.iter_mut().for_each(|value| {
            if *value == Value::None {
                self.missing += 1;
            }
            if  self.minimum == Value::None || self.minimum > *value {
                self.minimum = value.clone();
            }
            if  self.maximum == Value::None || self.maximum < *value {
                self.maximum = value.clone();
            }
        });
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

