use super::repository::Repository;
use crate::models::{CustomData, User};
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, ServiceError>;

/// Errors for the Service trait implementations
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("User is not authorized to access this resource")]
    Unauthorized,
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
}

#[async_trait]
pub trait Service: Send + Sync + std::fmt::Debug {
    /// Gets a user by id.
    /// The caller_id is optional and it's used for validation purposes.
    async fn get_user(&self, user_id: &Uuid, caller_id: Option<String>) -> Result<User>;

    /// Updates the custom_data of a user by id.
    /// The caller_id is optional and it's used for validation purposes.
    async fn update_user(
        &self,
        user_id: &Uuid,
        caller_id: Option<String>,
        custom_data: CustomData,
    ) -> Result<User>;

    /// Creates a new user.
    async fn create_user(&self, user: User) -> Result<User>;

    /// Deletes a user.
    /// The caller_id is optional and it's used for validation purposes.
    async fn delete_user(&self, user_id: &Uuid, caller_id: Option<String>) -> Result<User>;
}

/// Helper struct to locate the service from the handlers
/// and avoid relying on a specific implementation of the Service trait.
/// We're using dynamic dispatching but it will make tests easier.
#[derive(Debug)]
pub struct ServiceInjector(Box<dyn Service>);

impl ServiceInjector {
    /// Builds a new ServiceInjector
    pub fn new(svc: impl Service + 'static) -> Self {
        Self(Box::new(svc))
    }
}

impl std::ops::Deref for ServiceInjector {
    type Target = dyn Service;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// Our custom Service implementing the Service trait.
#[derive(Debug)]
pub struct Rpts02Service<T: Repository> {
    pub repository: T,
}

impl<T: Repository> Rpts02Service<T> {
    /// Builds a new Rpts02Service
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T: Repository + Send + Sync + 'static> Service for Rpts02Service<T> {
    #[instrument]
    async fn get_user(&self, user_id: &Uuid, caller_id: Option<String>) -> Result<User> {
        authorized!(user_id, caller_id);
        self.repository
            .get_user(&user_id)
            .await
            .map_err(|e| e.into())
    }

    #[instrument]
    async fn update_user(
        &self,
        user_id: &Uuid,
        caller_id: Option<String>,
        custom_data: CustomData,
    ) -> Result<User> {
        authorized!(user_id, caller_id);
        self.repository
            .update_user(user_id, custom_data)
            .await
            .map_err(|e| e.into())
    }

    #[instrument]
    async fn create_user(&self, user: User) -> Result<User> {
        self.repository
            .create_user(user)
            .await
            .map_err(|e| e.into())
    }

    #[instrument]
    async fn delete_user(&self, user_id: &Uuid, caller_id: Option<String>) -> Result<User> {
        authorized!(user_id, caller_id);
        self.repository
            .delete_user(user_id)
            .await
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use sqlx::{types::Json, Result};

    mock! {
        pub Repo {
            fn sync_get_user(&self, id: &Uuid) -> Result<User> {}
            fn sync_update_user(
                &self,
                id: &Uuid,
                custom_data: CustomData,
            ) -> Result<User> {}
            fn sync_create_user(&self, user: User) -> Result<User> {}
            fn sync_delete_user(&self, id: &Uuid) -> Result<User> {}
        }
    }

    impl std::fmt::Debug for MockRepo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("MockRepo").finish()
        }
    }

    #[async_trait]
    impl Repository for MockRepo {
        async fn get_user(&self, user: &Uuid) -> Result<User> {
            self.sync_get_user(user)
        }
        async fn create_user(&self, user: User) -> Result<User> {
            self.sync_create_user(user)
        }
        async fn update_user(&self, id: &Uuid, custom_data: CustomData) -> Result<User> {
            self.sync_update_user(id, custom_data)
        }
        async fn delete_user(&self, id: &Uuid) -> Result<User> {
            self.sync_delete_user(id)
        }
    }

    // get user tests

    #[actix_rt::test]
    async fn get_user_returns_if_userid_equals_caller() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();
        let user_name = "my_name";

        mock.expect_sync_get_user().returning(move |id| {
            let mut user = User::default();
            user.id = Some(*id);
            user.name = user_name.to_string();
            Ok(user)
        });

        let svc = Rpts02Service::new(mock);

        let result = svc
            .get_user(&user_id, Some(user_id.to_string()))
            .await
            .unwrap();

        assert_eq!(result.name, user_name);
        assert_eq!(result.id.unwrap(), user_id);
    }

    #[actix_rt::test]
    async fn get_user_returns_if_caller_is_none() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();
        let user_name = "my_name";

        mock.expect_sync_get_user().returning(move |id| {
            let mut user = User::default();
            user.id = Some(*id);
            user.name = user_name.to_string();
            Ok(user)
        });

        let svc = Rpts02Service::new(mock);

        let result = svc.get_user(&user_id, None).await.unwrap();

        assert_eq!(result.name, user_name);
        assert_eq!(result.id.unwrap(), user_id);
    }

    #[actix_rt::test]
    async fn get_user_returns_unauthorized_if_userid_not_equal_caller() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();

        mock.expect_sync_get_user()
            .returning(|_| Ok(User::default()));

        let svc = Rpts02Service::new(mock);

        let error = svc
            .get_user(&user_id, Some("2".to_string()))
            .await
            .err()
            .unwrap();

        let is_unauthorized = match error {
            ServiceError::Unauthorized => true,
            _ => false,
        };

        assert!(is_unauthorized);
    }

    #[actix_rt::test]
    async fn get_user_returns_mapped_error() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();

        mock.expect_sync_get_user()
            .returning(|_| Err(sqlx::Error::RowNotFound));

        let svc = Rpts02Service::new(mock);

        let error = svc
            .get_user(&user_id, Some(user_id.to_string()))
            .await
            .err()
            .unwrap();

        let is_mapped_error = match error {
            ServiceError::DbError(sqlx::Error::RowNotFound) => true,
            _ => false,
        };

        assert!(is_mapped_error);
    }

    // update user tests

    #[actix_rt::test]
    async fn update_user_returns_if_userid_equals_caller() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();
        let user_name = "my_name";
        let random = 78900;

        mock.expect_sync_update_user()
            .returning(move |id, custom_data| {
                let mut user = User::default();
                user.id = Some(*id);
                user.name = user_name.to_string();
                user.custom_data = Some(Json(custom_data));
                Ok(user)
            });

        let svc = Rpts02Service::new(mock);

        let result = svc
            .update_user(&user_id, Some(user_id.to_string()), CustomData { random })
            .await
            .unwrap();

        assert_eq!(result.id.unwrap(), user_id);
        assert_eq!(result.name, user_name);
        assert_eq!(result.custom_data.unwrap().random, random);
    }

    #[actix_rt::test]
    async fn update_user_returns_if_caller_is_none() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();
        let user_name = "my_name";
        let random = 78900;

        mock.expect_sync_update_user()
            .returning(move |id, custom_data| {
                let mut user = User::default();
                user.id = Some(*id);
                user.name = user_name.to_string();
                user.custom_data = Some(Json(custom_data));
                Ok(user)
            });

        let svc = Rpts02Service::new(mock);

        let result = svc
            .update_user(&user_id, None, CustomData { random })
            .await
            .unwrap();

        assert_eq!(result.id.unwrap(), user_id);
        assert_eq!(result.name, user_name);
        assert_eq!(result.custom_data.unwrap().random, random);
    }

    #[actix_rt::test]
    async fn update_user_returns_unauthorized_if_userid_not_equal_caller() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();

        mock.expect_sync_update_user()
            .returning(|_, _| Ok(User::default()));

        let svc = Rpts02Service::new(mock);

        let error = svc
            .update_user(&user_id, Some("2".to_string()), CustomData::default())
            .await
            .err()
            .unwrap();

        let is_unauthorized = match error {
            ServiceError::Unauthorized => true,
            _ => false,
        };

        assert!(is_unauthorized);
    }

    #[actix_rt::test]
    async fn update_user_returns_mapped_error() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();

        mock.expect_sync_update_user()
            .returning(|_, _| Err(sqlx::Error::RowNotFound));

        let svc = Rpts02Service::new(mock);

        let error = svc
            .update_user(&user_id, Some(user_id.to_string()), CustomData::default())
            .await
            .err()
            .unwrap();

        let is_mapped_error = match error {
            ServiceError::DbError(sqlx::Error::RowNotFound) => true,
            _ => false,
        };

        assert!(is_mapped_error);
    }

    // create user tests

    #[actix_rt::test]
    async fn create_user_returns_if_userid_equals_caller() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();
        let user_name = "my_name";

        let mut user = User::default();
        user.name = user_name.to_string();
        user.id = Some(user_id);

        mock.expect_sync_create_user().returning(|usr| Ok(usr));

        let svc = Rpts02Service::new(mock);

        let result = svc.create_user(user).await.unwrap();

        assert_eq!(result.id.unwrap(), user_id);
        assert_eq!(result.name, user_name);
    }

    #[actix_rt::test]
    async fn create_user_returns_mapped_error() {
        let mut mock = MockRepo::default();

        mock.expect_sync_create_user()
            .returning(|_| Err(sqlx::Error::RowNotFound));

        let svc = Rpts02Service::new(mock);

        let error = svc.create_user(User::default()).await.err().unwrap();

        let is_mapped_error = match error {
            ServiceError::DbError(sqlx::Error::RowNotFound) => true,
            _ => false,
        };

        assert!(is_mapped_error);
    }

    // delete user tests

    #[actix_rt::test]
    async fn delete_user_returns_if_userid_equals_caller() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();
        let user_name = "my_name";

        mock.expect_sync_delete_user().returning(move |id| {
            let mut user = User::default();
            user.id = Some(*id);
            user.name = user_name.to_string();
            Ok(user)
        });

        let svc = Rpts02Service::new(mock);

        let result = svc
            .delete_user(&user_id, Some(user_id.to_string()))
            .await
            .unwrap();

        assert_eq!(result.id.unwrap(), user_id);
        assert_eq!(result.name, user_name);
    }

    #[actix_rt::test]
    async fn delete_user_returns_if_caller_is_none() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();
        let user_name = "my_name";

        mock.expect_sync_delete_user().returning(move |id| {
            let mut user = User::default();
            user.id = Some(*id);
            user.name = user_name.to_string();
            Ok(user)
        });

        let svc = Rpts02Service::new(mock);

        let result = svc.delete_user(&user_id, None).await.unwrap();

        assert_eq!(result.id.unwrap(), user_id);
        assert_eq!(result.name, user_name);
    }

    #[actix_rt::test]
    async fn delete_user_returns_unauthorized_if_userid_not_equal_caller() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();

        mock.expect_sync_delete_user()
            .returning(|_| Ok(User::default()));

        let svc = Rpts02Service::new(mock);

        let error = svc
            .delete_user(&user_id, Some("2".to_string()))
            .await
            .err()
            .unwrap();

        let is_unauthorized = match error {
            ServiceError::Unauthorized => true,
            _ => false,
        };

        assert!(is_unauthorized);
    }

    #[actix_rt::test]
    async fn delete_user_returns_mapped_error() {
        let mut mock = MockRepo::default();
        let user_id = Uuid::new_v4();

        mock.expect_sync_delete_user()
            .returning(|_| Err(sqlx::Error::RowNotFound));

        let svc = Rpts02Service::new(mock);

        let error = svc
            .delete_user(&user_id, Some(user_id.to_string()))
            .await
            .err()
            .unwrap();

        let is_mapped_error = match error {
            ServiceError::DbError(sqlx::Error::RowNotFound) => true,
            _ => false,
        };

        assert!(is_mapped_error);
    }
}
