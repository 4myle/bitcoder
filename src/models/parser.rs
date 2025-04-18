
pub enum Token 
{
    None,
    OperatorLowest,
    OperatorHighest,
    OperatorRange,
    Delimiter,
    String {value: String},
    Number {value: f32}
}

pub struct Parser
{
    input: String,
    error: String 
}

impl Parser
{
    pub fn new (input: String) -> Self {
        Self {
            input,
            error: String::new()
        }
    }

    pub fn parse () -> Result<(), &'static str> {
        if false {
            return Err("Syntax error in template.")
        }
        Ok(())
    }

    pub fn token () -> Result<Token, &'static str> {
        if false {
            return Err("Strings must be enclosed in quotation marks.")
        }
        Ok(Token::None)
    }

}
