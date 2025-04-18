/*
    Process a vector of Variables, transforms values according to its Expressions and 
    writes the result as rows of bit strings with outcome (fitness) value at the end.
*/

use crate::models::variable::Variable;
use std::fs::File;
use std::io::Write;

pub struct Encoder;
impl Encoder 
{
    pub fn save (path: &str, variables: &[Variable]) -> Result<(), &'static str> {
        if let Some(desktop) = dirs::desktop_dir() {
            let original = std::path::PathBuf::from(&path);
            let mut path = desktop.join(original.file_name().unwrap_or_default());
            let mut data = String::new();
            path.set_extension("bitcoder");
            if let Ok(mut file) = File::create(path) {
                for index in 0..variables.len() {
                    // if let Some(parts) = self.data.get_parts(row) {
                    //     if let Ok(mut target) = self.parser.transform(parts, true) {
                            data.push(char::from_u32(u32::try_from(index).unwrap_or_default() % 2).unwrap_or_default());
                            if file.write_all(data.as_bytes()).is_err() {
                                return Err("Error when writing to file.");
                            }
                        // }
                    // }
                }
            }
        }
        Ok(())
    }

// pub fn transform (&self, parts: &[Box<str>], do_quotes: bool) -> Result<String, &str> {
//     if self.replacer.is_empty()  || self.target.positions.is_empty() {
//         return Err("Nothing to transform.");
//     }
//     if parts.len() < self.target.positions.len() {
//         return Err("Source variables fewer than target variables.");
//     }
//     let mut result = self.replacer.clone();
//     for position in &self.target.positions { 
//         let part: String = if do_quotes { 
//             format!("\"{}\"", parts[*position])
//         } else {
//             parts[*position].to_string()
//         };
//         // Make sure "$11" is not replaced together with "$1" (hence "replacen").
//         result = result.replacen(&format!("${}", position+1), &part, 1);
//     }
//     Ok(result)
// }

}
