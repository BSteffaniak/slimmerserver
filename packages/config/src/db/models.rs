use moosicbox_json_utils::{database::ToValue, ParseError, ToValueType};

pub struct Profile {
    pub id: u64,
    pub name: String,
}

impl ToValueType<Profile> for &moosicbox_database::Row {
    fn to_value_type(self) -> Result<Profile, ParseError> {
        Ok(Profile {
            id: self.to_value("id")?,
            name: self.to_value("data")?,
        })
    }
}
