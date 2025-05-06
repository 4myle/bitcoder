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

#[derive(Debug, Default, Clone, PartialEq)]
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
    Cluster {clusters: Vec<Range>},
    #[default] 
    Recode
}

//TODO: let this replace both ´uniques´ and ´missing´.
#[derive(Debug, Default)]
pub struct Histogram
{
    density: HashMap<String,usize>, // Frequency of unique values, key is bit variable name (bitname).
    // dropped: usize, // Number of uncategorized values (not in any cluster).
    summary: usize, // Number of unique bit variables to be created.
    missing: usize, // Number of missing values.
    minimum: Value, // Minimum value (string or number).
    maximum: Value, // Maximum value (string or number).

}


#[derive(Default)]
pub struct Variable
{
    name: String,
    values: Vec<Value>,
    backup: Vec<Value>,
    histogram: Histogram,    
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
            histogram: Histogram::default(),
            mapping: Mapping::default(),
            is_numeric: false
        }
    }

    pub fn set_recoded (&mut self) {
        self.mapping = Mapping::Recode;
    }

    pub fn set_cluster (&mut self, tokens: &[Token]) -> Result<(), &'static str> {
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
            ranges.push(range);
        }
        let all_numbers = ranges.iter().all(|r| matches!(r.lower, Value::Number{..}) && matches!(r.upper, Value::Number{..}));
        let all_strings = ranges.iter().all(|r| matches!(r.lower, Value::String{..}) && matches!(r.upper, Value::String{..}));
        if self.is_numeric && !all_numbers || !self.is_numeric && !all_strings {
            return Err("All values must be of the same type and must match the variable type.")
        }
        self.mapping = Mapping::Cluster { clusters: ranges };
        Ok(())
    }

    // fn range_from (&self, value: &Value) -> Option<&Range> {
    //     match &self.mapping {
    //         Mapping::Cluster { clusters } => {
    //             for cluster in clusters {
    //                 if  cluster.lower != Value::None && *value >= cluster.lower && *value <= cluster.upper {
    //                     return Some(&cluster);
    //                 }
    //             }
    //         },
    //         _ => {
    //             return None;
    //         }
    //     }
    //     None
    // }

    // fn recalculate (&mut self, value: &Value) {
    //     if *value == Value::None {
    //         self.histogram.missing += 1;
    //     } else {
    //         let mut identifier = String::new();
    //         match &self.mapping {
    //             Mapping::Recode => {
    //                 identifier = self.value_as_name(value);
    //             },
    //             Mapping::Cluster {..} => {
    //                 if let Some(range) = self.range_from(value) {
    //                     identifier = self.range_as_name(range);
    //                 } else {
    //                     identifier = String::from("Other");
    //                 }
    //             }
    //         }
    //         *self.histogram.density.entry(identifier).or_insert(0) += 1;
    //     }
    //     if  self.histogram.minimum == Value::None || self.histogram.minimum > *value {
    //         self.histogram.minimum = value.clone();
    //     }
    //     if  self.histogram.maximum == Value::None || self.histogram.maximum < *value {
    //         self.histogram.maximum = value.clone();
    //     }
    //     self.histogram.summary += 1;
    // }

    fn value_as_name (name: &String, value: &Value) -> String {
        name.to_owned() + "--" + &value.to_string()
    }

    fn range_as_name (name: &String, range: &Range) -> String {
        name.to_owned() + "--" + &range.lower.to_string() + "-to-" + &range.upper.to_string()
    }

    fn range_from<'a> (value: &Value, clusters: &'a [Range]) -> Option<&'a Range> {
        clusters.iter().find(|&cluster| cluster.lower != Value::None && *value >= cluster.lower && *value <= cluster.upper)
    }

    // Associated function instead of "method" to avoid (unsolvable?) borrow checker issues with self. 
    fn recalculate (histogram: &mut Histogram, mapping: &Mapping, name: &String, value: &Value) {
        if *value == Value::None {
            histogram.missing += 1;
        } else {
            let identifier;
            match &mapping {
                Mapping::Recode => {
                    identifier = Self::value_as_name(name, value);
                },
                Mapping::Cluster { clusters } => {
                    if let Some(range) = Self::range_from(value, clusters) {
                        identifier = Self::range_as_name(name, range);
                    } else {
                        identifier = String::from("Other");
                    }
                }
            }
            *histogram.density.entry(identifier).or_insert(0) += 1;
        }
        if  histogram.minimum == Value::None || histogram.minimum > *value {
            histogram.minimum = value.clone();
        }
        if  histogram.maximum == Value::None || histogram.maximum < *value {
            histogram.maximum = value.clone();
        }
        histogram.summary += 1;
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
        // println!("{0: <10} | {1: <10}", );
        println!("{:#?}", self.histogram);

    }

    pub fn as_strings (&mut self) {
        self.is_numeric = false;
        self.values = self.backup.clone();
        self.backup.clear();
        self.histogram = Histogram::default();
        for value in &mut self.values {
            Self::recalculate(&mut self.histogram, &self.mapping, &self.name, value);
        }
        // println!("As strings, {} contains {} unique values", self.name, self.uniques.len());
    }

    pub fn name (&self) -> &str {
        self.name.as_str()
    }

    // pub fn histogram (&self) -> &Histogram {
    //     &self.histogram
    // }

    pub fn uniques (&self) -> usize {
        self.histogram.summary
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
    
    // fn value_as_name (&self, value: &Value) -> String {
    //     //TODO: implement.
    //     String::from("MyFancyBitName")
    // }
    
    // fn range_as_name (&self, range: &Range) -> String {
    //     //TODO: implement.
    //     String::from("MyFancyBitName")
    // }

}
