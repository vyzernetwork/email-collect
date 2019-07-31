#[derive(Queryable, Debug)]
pub struct EmailAddress {
    pub address: String,
    pub confirmed: bool, 
}

use super::schema::emails;

#[derive(Insertable)]
#[table_name="emails"]
pub struct NewEmailAddress<'a> {
    pub address: &'a str,
}

#[derive(Insertable)]
#[table_name="emails"]
pub struct NewConfirmation<'a> {
    pub address: &'a str,
    pub confirmed: bool,
}


