use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::pg::PgConnection;

/// Trait for items that can have a password applied to them.
pub trait Passwordable {
    /// Set the password on the object.
    /// The string contains an error if there is one.
    fn set_password(&mut self, password: Option<String>, conn: &PgConnection) -> Option<String>;

    /// Get the hashed password from the database
    fn get_hashed_password(&self, conn: &PgConnection) -> Option<String>;

    /// Check the password for correctness
    fn check_password(&self, password: String, conn: &PgConnection) -> bool
    {
        let hashed_pw = self.get_hashed_password(conn);
        match hashed_pw {
            Some(s) => verify(&password, &s).unwrap(),
            None => true
        }
    }

    // TODO: Figure out how this will work for pastes
    /// Gets the path of this resource on s3 
    fn get_s3_location(&self) -> String;


    /// Gets the owner of te object
    fn owner(&self) -> i32;
}

/// If the password is not none, this will hash it and return it
/// Otherwise, it just passes
pub fn retrieve_hashed(pw: Option<String>) -> Option<String>
{
    match pw {
        Some(v) => Some(hash(&v, DEFAULT_COST).unwrap()),
        None => None
    }
}
