//! `users` domain model (Database.md Section 3.1). Deliberately excludes `password_hash` —
//! this is the struct returned to the frontend, and IPC.md's `auth_list_users` contract requires
//! that field never leave the backend.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub full_name: String,
    pub username: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn serializes_field_names_as_camel_case_and_never_includes_a_password_hash() {
        let user = User {
            id: 1,
            full_name: "Ada Lovelace".to_string(),
            username: "ada".to_string(),
            role: "admin".to_string(),
            is_active: true,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: None,
        };

        let json = serde_json::to_string(&user).expect("serializes");

        assert!(json.contains("\"fullName\""));
        assert!(json.contains("\"isActive\""));
        assert!(json.contains("\"createdAt\""));
        assert!(!json.contains("password"));
    }
}
