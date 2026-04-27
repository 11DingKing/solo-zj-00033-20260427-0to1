use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Snippet {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub language: String,
    pub is_public: bool,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub likes_count: i32,
    pub forks_count: i32,
    pub views_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSnippet {
    pub title: String,
    pub description: Option<String>,
    pub language: String,
    pub is_public: bool,
    pub files: Vec<CreateFile>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSnippet {
    pub title: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub is_public: Option<bool>,
    pub files: Option<Vec<CreateFile>>,
    pub tags: Option<Vec<String>>,
    pub commit_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SnippetFile {
    pub id: Uuid,
    pub snippet_id: Uuid,
    pub version_id: Uuid,
    pub filename: String,
    pub content: String,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFile {
    pub filename: String,
    pub content: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Version {
    pub id: Uuid,
    pub snippet_id: Uuid,
    pub version_number: i32,
    pub commit_message: Option<String>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Like {
    pub id: Uuid,
    pub user_id: Uuid,
    pub snippet_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub snippet_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComment {
    pub content: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub language: String,
    pub is_public: bool,
    pub user: UserResponse,
    pub parent_id: Option<Uuid>,
    pub likes_count: i32,
    pub forks_count: i32,
    pub views_count: i32,
    pub files: Vec<SnippetFile>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub snippet: SnippetResponse,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
