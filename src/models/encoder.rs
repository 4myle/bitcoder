/*
    Processes a vector of variables and recodes or clusters values according to their expression and 
    then writes the result as rows of bit strings with corresponding outcome (fitness) value at the end.
*/

use crate::models::variable::Variable;
use std::fs::File;
use std::io::Write;

pub struct Encoder;
impl Encoder 
{
    pub fn save (path: &str, variables: &[Variable], outcome: &Variable, rows: usize) -> Result<(), &'static str> {
        if let Some(desktop) = dirs::desktop_dir() {
            let original = std::path::PathBuf::from(&path);
            let mut path = desktop.join(original.file_name().unwrap_or_default());
            let mut data = String::new();
            path.set_extension("bitcoder");
            if let Ok(mut file) = File::create(path) {
                // Write variable names within quotation and comma-separated.
                data.clear();
                for variable in variables {
                    for name in variable.density().keys() {
                        data.push_str(format!("\"{name}\",").as_str());
                    }
                }
                data.push_str(format!("\"{}\"", outcome.name()).as_str());
                if file.write_all(data.as_bytes()).is_err() {
                    return Err("Error when writing to file.");
                }
                // Write clustered variable values as bit strings, end with outcome value.
                for index in 0..rows {
                    data.clear();
                    data.push('\n');
                    for variable in variables {
                        let bits: String = variable.vector_of(index)
                            .iter()
                            .map(|b| if b.1 { "1"} else {"0"})
                            .collect();
                        data.push_str(bits.as_str());
                    }
                    data.push_str(format!(",{}", outcome.value_at(index)).as_str());
                    if file.write_all(data.as_bytes()).is_err() {
                        return Err("Error when writing to file.");
                    }
                }
            } else {
                return Err("File could not be opened for writing. Is it open somewhere else?");
            }
        }
        Ok(())
    }
}
