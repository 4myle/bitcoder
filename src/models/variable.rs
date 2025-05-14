/*
    Represents a variable, holding a vector of data points as either a strings, f32s or "missing".
*/
 
use crate::models::parser::Token;
use std::cmp::Ordering;

use std::collections::HashMap;
use std::hash::{
    Hash,
    Hasher
};
use std::fmt::{
    Display,
    Formatter
};

#[derive(Default, Clone, PartialEq)]
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

impl Eq for Value {}
impl Hash for Value 
{
    fn hash<H: Hasher>  (&self, state: &mut H) {
        match self {
            Value::String { string: s } => s.hash(state),
            Value::Number { number: n } => format!("{n:.3}").as_str().hash(state),
            Value::None => "13".hash(state),
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

#[derive(Default, PartialEq)]
pub struct Range 
{
    lower: Value, 
    upper: Value
}

#[derive(Default, PartialEq)]
pub enum Mapping 
{
    Cluster {clusters: Vec<Range>}, // Values are grouped into clusters, as described by an expression.
    #[default]                      // Recode is the default ...
    Recode                          // .. and means every unique value is a group.
}

#[derive(Default)]
pub struct Histogram
{
    density: HashMap<String,usize>, // Frequency of unique values, key is bit variable name (bitname).
    missing: usize,                 // Number of missing values.
    minimum: Value,                 // Minimum value (String or Number).
    maximum: Value,                 // Maximum value (String or Number).

}


#[derive(Default)]
pub struct Variable
{
    name: String,           // Identifier of this variable
    values: Vec<Value>,     // List of actual values (Number, String or None). 
    backup: Vec<Value>,     // Clone of string values when converting to number (and back).
    histogram: Histogram,   // Statistics, including table of frequence.
    mapping: Mapping,       // Values are either grouped as one cluster per unique value, or into clusters through an expression.
    is_included: bool,      // If included in output or not.
    is_numeric: bool        // If all values are numbers.
}

impl Variable
{
    pub fn new (name: &str) -> Self {
        Self { 
            name: name.to_string(),
            values: Vec::new(),
            backup: Vec::new(),
            histogram: Histogram::default(),
            mapping: Mapping::default(),
            is_included: true,
            is_numeric: false
        }
    }

    pub fn set_recoded (&mut self) {
        self.mapping = Mapping::Recode;
        self.histogram = Histogram::default();
        for value in &mut self.values {
            Self::recalculate(&mut self.histogram, &self.mapping, &self.name, value);
        }
    }

    pub fn set_cluster (&mut self) {
        self.mapping = Mapping::Cluster { clusters: Vec::new() };
        self.histogram = Histogram::default();
        for value in &mut self.values {
            Self::recalculate(&mut self.histogram, &self.mapping, &self.name, value);
        }
    }

    pub fn use_ranges (&mut self, tokens: &[Token]) -> Result<(), &'static str> {
        let mut ranges = Vec::<Range>::new();
        let mut tokens = tokens.iter();
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
                Some(Token::Minimum) => range.lower = self.histogram.minimum.clone(),
                _ => return Err("Type a value for the lower range.")
            }
            if operator.is_none() || operator != Some(&Token::Range) {
                return Err("Separate range values with keyword ' to '.")
            }
            match value2nd {
                Some(Token::Number { value }) => range.upper = Value::Number { number: *value },
                Some(Token::String { value }) => range.upper = Value::String { string: value.as_str().to_string() },
                Some(Token::Maximum) => range.upper = self.histogram.maximum.clone(),
                _ => return Err("Type a value for the upper range.")
            }
            if range.lower > range.upper {
                return Err("Lower value must be less than or equal to the upper value.")
            }
            if !ranges.is_empty() && range.lower < Option::unwrap(ranges.last()).upper {
                return Err("Lower value must be greater than or equal to the previous upper value.")
            }
            ranges.push(range);
        }
        let all_numbers = ranges.iter().all(|r| matches!(r.lower, Value::Number{..}) && matches!(r.upper, Value::Number{..}));
        let all_strings = ranges.iter().all(|r| matches!(r.lower, Value::String{..}) && matches!(r.upper, Value::String{..}));
        if self.is_numeric && !all_numbers || !self.is_numeric && !all_strings {
            return Err("All values must be of the same type and must match the variable type.")
        }
        let clusters = !ranges.is_empty();
        self.mapping = Mapping::Cluster { clusters: ranges };
        if clusters {
            self.histogram = Histogram::default();
            for value in &mut self.values {
                Self::recalculate(&mut self.histogram, &self.mapping, &self.name, value);
            }
        }
        Ok(())
    }

    pub fn add_value (&mut self, value: &str) {
        let value = Value::new(value);
        Self::recalculate(&mut self.histogram, &self.mapping, &self.name, &value);
        self.values.push(value);
    }

    pub fn as_numbers (&mut self) {
        self.is_numeric = true;
        self.backup = self.values.clone();
        self.histogram = Histogram::default();
        for value in &mut self.values {
            value.as_number();
            Self::recalculate(&mut self.histogram, &self.mapping, &self.name, value);
        }
    }

    pub fn as_strings (&mut self) {
        self.is_numeric = false;
        self.values = self.backup.clone();
        self.backup.clear();
        self.histogram = Histogram::default();
        for value in &mut self.values {
            Self::recalculate(&mut self.histogram, &self.mapping, &self.name, value);
        }
    }

    pub fn name (&self) -> &str {
        self.name.as_str()
    }

    pub fn density (&self) -> &HashMap<String,usize> {
        &self.histogram.density
    }

    pub fn missing (&self) -> usize {
        self.histogram.missing
    }

    pub fn minimum (&self) -> &Value {
        &self.histogram.minimum
    }

    pub fn maximum (&self) -> &Value {
        &self.histogram.maximum
    }

    pub fn mapping (&self) -> &Mapping {
        &self.mapping
    }

    pub fn value_at (&self, index: usize) -> &Value {
        &self.values[index]
    }

    pub fn set_name (&mut self, name: &str) {
        self.name = name.to_string();
        // Recalculation needed since bit variable names are stored in density map.
        self.histogram = Histogram::default();
        for value in &mut self.values {
            Self::recalculate(&mut self.histogram, &self.mapping, &self.name, value);
        }
    }

    pub fn include (&mut self) {
        if self.is_included {
            return;
        }
        self.is_included = true;
        self.histogram = Histogram::default();
        for value in &mut self.values {
            Self::recalculate(&mut self.histogram, &self.mapping, &self.name, value);
        }
    }

    pub fn exclude (&mut self) {
        if !self.is_included {
            return;
        }
        self.is_included = false;
        self.histogram = Histogram::default();
    }

    pub fn vector_of (&self, index: usize) -> Vec<(String,bool)> {
        let mut bits = Vec::<(String,bool)>::new();
        let current  = Self::bit_name(&self.mapping, &self.name, &self.values[index]);
        for key in self.histogram.density.keys() {
            bits.push((key.clone(), *key == current));
        }
        bits
    }

    // Associated function instead of method to avoid "cannot mutate self twice". 
    fn bit_name (mapping: &Mapping, name: &String, value: &Value) -> String {
        match &mapping {
            Mapping::Recode => {
                Self::name_from_value(name, value)
            },
            Mapping::Cluster { clusters } => {
                if let Some(range) = Self::get_range(value, clusters) {
                    return Self::name_from_range(name, range);
                }
                name.to_owned() + "|Other"
            }
        }
    }

    // Associated function instead of method to avoid "cannot mutate self twice". 
    fn name_from_value (name: &String, value: &Value) -> String {
        name.to_owned() + "|" + &value.to_string()
    }

    // Associated function instead of method to avoid "cannot mutate self twice". 
    fn name_from_range (name: &String, range: &Range) -> String {
        name.to_owned() + "|" + &range.lower.to_string() + "|" + &range.upper.to_string()
    }

    // Associated function instead of method to avoid "cannot mutate self twice". 
    fn get_range<'a> (value: &Value, clusters: &'a [Range]) -> Option<&'a Range> {
        clusters.iter().find(|&cluster| cluster.lower != Value::None && *value >= cluster.lower && *value <= cluster.upper)
    }

    // Associated function instead of method to avoid "cannot mutate self twice". 
    fn recalculate (histogram: &mut Histogram, mapping: &Mapping, name: &String, value: &Value) {
        if *value == Value::None {
            histogram.missing += 1;
        } else {
            *histogram.density.entry(Self::bit_name(mapping, name, value)).or_insert(0) += 1;
        }
        if  histogram.minimum == Value::None || histogram.minimum > *value {
            histogram.minimum = value.clone();
        }
        if  histogram.maximum == Value::None || histogram.maximum < *value {
            histogram.maximum = value.clone();
        }
    }

}

