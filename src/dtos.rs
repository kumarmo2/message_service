use serde::{Deserialize, Serialize};

// I am sad that I had to repeat the definition of this dto again(I had already defined in rocket project).
// Forgive me.:( . It was the easiest think to do at the moment.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: i32,
    pub name: String,
    pub age: i32,
    pub email: String,
}
