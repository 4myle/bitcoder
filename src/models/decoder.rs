/* 
    Reads and parses a CSV file and creates a vector of variables.
*/


use crate::models::variable::Variable;
use std::fs::File;
use std::io::{
    BufRead, 
    BufReader
};

static SPLITTER: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    Result::unwrap(regex::Regex::new(r#""[^"]*"|[^,]*"#)) // Pre-compiling for performance.
});

pub struct Decoder;
impl Decoder 
{
    pub fn load (path: &str, variables: &mut Vec<Variable>, rows: &mut usize) -> Result<(), & 'static str> {
        variables.clear();
        *rows = 0;
        if let Ok(file) = File::open(path) {
            let mut lines = BufReader::new(file).lines();
            if let Some(Ok(row)) = lines.next() { 
                if let Ok(names) = Self::split(&row) {
                    for name in names {
                        variables.push(Variable::new(name));
                    };
                }
            } else {
                return Err("The file appears to be empty.")
            }
            while let Some(Ok(row)) = lines.next() {
                if let Ok(values) = Self::split(&row) {
                    // Skip rows that are missing value of outcome variable (last one).
                    if values.ends_with(&[""]) {
                        continue;
                    }
                    if values.len() != variables.len() {
                        return Err("Number of values differ between rows in this file.")
                    }
                    for (index, value) in values.into_iter().enumerate() {
                        variables[index].add_value(value);
                    };
                }
                *rows += 1;
            };
        } else {
            return Err("File cannot be opened. Is it opened somewhere else?")
        }
        Ok(())
    }

    fn split (row: &str) -> Result<Vec<&str>, &str> {
        if row.is_empty() {
            return Err("Nothing to split.");
        }
        let mut result = Vec::new();
        let bytes  = row.as_bytes();
        let slices = SPLITTER.find_iter(row);
        for slice in slices {
            let mut lower = slice.range().start;
            let mut upper = slice.range().end;
            if  lower < bytes.len() && bytes[lower] == b'"' && bytes[upper-1] == b'"' {
                lower += 1;
                upper -= 1;
            }
            result.push(&row[lower..upper]);
        }
        Ok(result)
    }
}
