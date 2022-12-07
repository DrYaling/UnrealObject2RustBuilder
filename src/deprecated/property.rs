
use serde::{Deserialize};
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Property{
    #[serde(default)]
    pub r#static: bool,
    pub name: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub value: Option<String>,
}