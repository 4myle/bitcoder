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
    pub fn save (path: &str, variables: &[Variable], rows: usize) -> Result<(), &'static str> {
        if let Some(desktop) = dirs::desktop_dir() {
            let original = std::path::PathBuf::from(&path);
            let mut path = desktop.join(original.file_name().unwrap_or_default());
            let mut data = String::new();
            path.set_extension("bitcoder");
            if let Ok(mut file) = File::create(path) {
                data.clear();
                for variable in variables {
                    for name in variable.density().keys() {
                        data.push_str(format!("\"{name}\",").as_str());
                    }
                    if  data.ends_with(',') {
                        data.remove(data.len()-1); 
                    }
                }
                if file.write_all(data.as_bytes()).is_err() {
                    return Err("Error when writing to file.");
                }
                for index in 0..rows {
                    for variable in variables {
                        // let density = variable.density();
                        // if file.write_all(data.as_bytes()).is_err() {
                        //     return Err("Error when writing to file.");
                        // }
                    }
                    //TODO: write outcome variable.
                }
            } else {
                return Err("File could not be opened for writing. Is it open somewhere else?");
            }
        }
        Ok(())
    }
}
