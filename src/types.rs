#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum Boolean {
    True,
    False,
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        match value
        {
            true => Boolean::True,
            false => Boolean::False
        }
    }
}

impl From<Boolean> for bool
{
    fn from(value: Boolean) -> Self {
        match value
        {
            Boolean::True => true,
            Boolean::False => false
        }
    }
}

impl From<Boolean> for String
{
    fn from(value: Boolean) -> Self {
        match value
        {
            Boolean::True => "True".into(),
            Boolean::False => "False".into()
        }
    }
}