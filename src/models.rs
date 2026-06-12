use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Note {
    pub id: i64,
    pub text: String,
    pub color: String,
    pub pos_x: i32,
    pub pos_y: i32,
    pub width: i32,
    pub height: i32,
    pub created_at: String,
    pub updated_at: String,
}
