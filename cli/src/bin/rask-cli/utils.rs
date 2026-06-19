use rask::user::*;

pub fn user_name_to_id(users: Vec<UserResponse>,name: &str) -> Option<usize> {
    for user in users {
        if user.screen_name == name {
            return Some(user.id);
        }
    }
    None
}
