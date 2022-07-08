
use serde::{Deserialize};
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Parameter{
    pub r#type: String,
    #[serde(default)]
    pub name: String,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Api{
    #[serde(default)]
    pub r#static: bool,
    pub name: String,
    #[serde(default)]
    pub ret_type: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}