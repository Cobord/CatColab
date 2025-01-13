use regex::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::app::{AppCtx, AppError};

/// Notify the backend that a user has signed up or signed in.
pub async fn sign_up_or_sign_in(ctx: AppCtx) -> Result<(), AppError> {
    let Some(user) = ctx.user else {
        return Err(AppError::Unauthorized);
    };
    let query = sqlx::query!(
        "
        INSERT INTO users(id, created, signed_in)
        VALUES ($1, NOW(), NOW())
        ON CONFLICT (id) DO UPDATE
        SET signed_in = EXCLUDED.signed_in
        ",
        user.user_id,
    );
    query.execute(&ctx.state.db).await?;
    Ok(())
}

/// Get profile data for a user.
pub async fn get_user_profile(ctx: AppCtx) -> Result<UserProfile, AppError> {
    let Some(user) = ctx.user else {
        return Err(AppError::Unauthorized);
    };
    let query = sqlx::query_as!(
        UserProfile,
        "
        SELECT username, display_name FROM users
        WHERE id = $1
        ",
        user.user_id,
    );
    Ok(query.fetch_one(&ctx.state.db).await?)
}

/// Set profile data for a user.
pub async fn set_user_profile(ctx: AppCtx, profile: UserProfile) -> Result<(), AppError> {
    let Some(user) = ctx.user else {
        return Err(AppError::Unauthorized);
    };
    profile.validate().map_err(AppError::Invalid)?;

    let query = sqlx::query!(
        "
        UPDATE users SET username = $2, display_name = $3
        WHERE id = $1
        ",
        user.user_id,
        profile.username,
        profile.display_name,
    );
    query.execute(&ctx.state.db).await?;
    Ok(())
}

/// Data of a user profile.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct UserProfile {
    pub username: Option<String>,
    pub display_name: Option<String>,
}

impl UserProfile {
    fn validate(&self) -> Result<(), String> {
        if let Some(username) = self.username.as_ref() {
            is_username_valid(username)
                .then_some(())
                .ok_or_else(|| "Username does not follow the rules".into())
        } else {
            Ok(())
        }
    }
}

/** Is the proposed user name valid?

A username is **valid** when it

- is nonempty
- comprises ASCII alphanumeric characters, dashes, dots, and underscores
- has alphanumeric first and last characters

In particular, this ensures that a valid username is also a valid URL. Similar
rules for usernames are enforced by sites such as GitHub.
 */
pub fn is_username_valid(username: &str) -> bool {
    let valid_chars = Regex::new(r"^[0-9A-Za-z_\-\.]*$").unwrap();
    let starts_alpha = Regex::new(r"^[0-9A-Za-z]").unwrap();
    let ends_alpha = Regex::new(r"[0-9A-Za-z]$").unwrap();

    valid_chars.is_match(username)
        && starts_alpha.is_match(username)
        && ends_alpha.is_match(username)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_user_profile() {
        assert!(UserProfile {
            username: None,
            display_name: None
        }
        .validate()
        .is_ok());

        assert!(UserProfile {
            username: Some("evan!".into()),
            display_name: Some("Evan".into()),
        }
        .validate()
        .is_err());
    }

    #[test]
    fn validate_username() {
        assert!(!is_username_valid(""));
        assert!(is_username_valid("foo"));
        assert!(!is_username_valid("_foo"));
        assert!(!is_username_valid("foo_"));
        assert!(is_username_valid("foo_bar"));
        assert!(is_username_valid("foo-bar"));
        assert!(is_username_valid("foo.bar"));
        assert!(!is_username_valid("foo!bar"));
    }
}
