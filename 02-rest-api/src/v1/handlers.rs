use super::service::{ServiceError, ServiceInjector};
use crate::models::{CustomData, User};
use actix_web::{error, web, HttpRequest, HttpResponse, Result};
use actix_web_middleware_cognito::CognitoInfo;
use std::sync::Arc;
use tracing::{self as log, instrument};
use uuid::Uuid;

pub mod users {
    use super::*;

    pub const PATH: &str = "/users";

    #[instrument]
    pub async fn get(
        id: web::Path<Uuid>,
        req: HttpRequest,
        auth: CognitoInfo,
        svc: web::Data<Arc<ServiceInjector>>,
    ) -> Result<HttpResponse> {
        svc_response!(
            svc.as_ref().get_user(&id, auth.user).await,
            HttpResponse::Ok(),
            req.path(),
            format!("Error getting user {}", id)
        )
    }

    #[instrument]
    pub async fn post(
        user: web::Json<User>,
        req: HttpRequest,
        svc: web::Data<Arc<ServiceInjector>>,
    ) -> Result<HttpResponse> {
        match svc.as_ref().create_user(user.into_inner()).await {
            Ok(usr) => {
                let response = HttpResponse::Created()
                    .header("Location", format!("{}/{}", req.path(), usr.id.unwrap()))
                    .json(usr);
                Ok(response)
            }
            Err(err) => {
                log::error!("Error creating user: {}", err);
                svc_err!(err)
            }
        }
    }

    #[instrument]
    pub async fn patch(
        id: web::Path<uuid::Uuid>,
        custom_data: web::Json<CustomData>,
        req: HttpRequest,
        auth: CognitoInfo,
        svc: web::Data<Arc<ServiceInjector>>,
    ) -> Result<HttpResponse> {
        svc_response!(
            svc.as_ref()
                .update_user(&id, auth.user, custom_data.into_inner())
                .await,
            HttpResponse::Ok(),
            req.path(),
            format!("Error updating user: {}", id)
        )
    }

    #[instrument]
    pub async fn delete(
        id: web::Path<uuid::Uuid>,
        req: HttpRequest,
        auth: CognitoInfo,
        svc: web::Data<Arc<ServiceInjector>>,
    ) -> Result<HttpResponse> {
        svc_response!(
            svc.as_ref().delete_user(&id, auth.user).await,
            HttpResponse::Ok(),
            req.path(),
            format!("Error getting user {}", id)
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::v1::service::{Result as ServiceResut, Service};
        use actix_web::{dev::Body, test};
        use async_trait::async_trait;
        use mockall::predicate::*;
        use mockall::*;

        mock! {
            pub Svc {
                fn sync_get_user(&self, user_id: &Uuid, caller_id: Option<String>) -> ServiceResut<User> {}
                fn sync_update_user(
                    &self, user_id: &Uuid,
                    caller_id: Option<String>,
                    custom_data: CustomData,
                ) -> ServiceResut<User> {}
                fn sync_create_user(
                    &self,
                    user: User,
                ) -> ServiceResut<User> {}
                fn sync_delete_user(
                    &self,
                    user_id: &Uuid,
                    caller_id: Option<String>,
                ) -> ServiceResut<User> {}
            }
        }

        impl std::fmt::Debug for MockSvc {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("MockSvc").finish()
            }
        }

        #[async_trait]
        impl Service for MockSvc {
            async fn get_user(
                &self,
                user_id: &Uuid,
                caller_id: Option<String>,
            ) -> ServiceResut<User> {
                self.sync_get_user(&user_id, caller_id)
            }
            async fn update_user(
                &self,
                user_id: &Uuid,
                caller_id: Option<String>,
                custom_data: CustomData,
            ) -> ServiceResut<User> {
                self.sync_update_user(&user_id, caller_id, custom_data)
            }
            async fn create_user(&self, user: User) -> ServiceResut<User> {
                self.sync_create_user(user)
            }
            async fn delete_user(
                &self,
                user_id: &Uuid,
                caller_id: Option<String>,
            ) -> ServiceResut<User> {
                self.sync_delete_user(&user_id, caller_id)
            }
        }

        // get handler

        #[actix_rt::test]
        async fn get_users_handler_works() {
            let user_id = Uuid::new_v4();
            let user_name = "my_name";
            let path = format!("/v1/users/{}", user_id);

            let mut mock_svc = MockSvc::default();
            mock_svc
                .expect_sync_get_user()
                .returning(move |user_id, _caller_id| {
                    let mut user = User::default();
                    user.id = Some(*user_id);
                    user.name = user_name.to_string();
                    Ok(user)
                });

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(user_id);
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::with_uri(path.as_ref()).to_http_request();
            let mut res: HttpResponse = get(id, req, auth, svc).await.unwrap();

            let user = res
                .take_body()
                .as_ref()
                .map(|b| match b {
                    Body::Bytes(x) => serde_json::from_slice::<'_, User>(x).ok(),
                    _ => None,
                })
                .flatten()
                .unwrap();

            let location = res.headers().get("Location").unwrap().to_str().unwrap();

            assert_eq!(user.id.unwrap(), user_id);
            assert_eq!(user.name, user_name.to_string());
            assert_eq!(location, path);
            assert!(res.status().is_success());
        }

        #[actix_rt::test]
        async fn get_users_handler_maps_err_to_unauthorized() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::Unauthorized;
            mock_svc
                .expect_sync_get_user()
                .returning(move |_, _| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(Uuid::new_v4());
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::default().to_http_request();
            let res = get(id, req, auth, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), err_svc().to_string());
            assert_eq!(err.status_code().as_u16(), 401);
        }

        #[actix_rt::test]
        async fn get_users_handler_maps_err_to_not_found() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::DbError(sqlx::Error::RowNotFound);
            mock_svc
                .expect_sync_get_user()
                .returning(move |_, _| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(Uuid::new_v4());
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::default().to_http_request();
            let res = get(id, req, auth, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), err_svc().to_string());
            assert_eq!(err.status_code().as_u16(), 404);
        }

        #[actix_rt::test]
        async fn get_users_handler_maps_err_to_internal() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::DbError(sqlx::Error::PoolTimedOut);
            mock_svc
                .expect_sync_get_user()
                .returning(move |_, _| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(Uuid::new_v4());
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::default().to_http_request();
            let res = get(id, req, auth, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), "Database Error");
            assert_eq!(err.status_code().as_u16(), 500);
        }

        // post handler

        #[actix_rt::test]
        async fn post_users_handler_works() {
            let user_id = Uuid::new_v4();
            let user_name = "my_name";
            let path = format!("/v1/users/{}", user_id);

            let mut user = User::default();
            user.id = Some(user_id);
            user.name = user_name.to_string();

            let mut mock_svc = MockSvc::default();
            mock_svc
                .expect_sync_create_user()
                .returning(move |user| Ok(user));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let usr = web::Json(user);
            let req = test::TestRequest::with_uri("/v1/users").to_http_request();
            let mut res: HttpResponse = post(usr, req, svc).await.unwrap();

            let user = res
                .take_body()
                .as_ref()
                .map(|b| match b {
                    Body::Bytes(x) => serde_json::from_slice::<'_, User>(x).ok(),
                    _ => None,
                })
                .flatten()
                .unwrap();

            let location = res.headers().get("Location").unwrap().to_str().unwrap();

            assert_eq!(user.id.unwrap(), user_id);
            assert_eq!(user.name, user_name);
            assert_eq!(location, path);
            assert!(res.status().is_success());
        }

        #[actix_rt::test]
        async fn post_users_handler_maps_err_to_unauthorized() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::Unauthorized;
            mock_svc
                .expect_sync_create_user()
                .returning(move |_| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let usr = web::Json(User::default());
            let req = test::TestRequest::default().to_http_request();
            let res = post(usr, req, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), err_svc().to_string());
            assert_eq!(err.status_code().as_u16(), 401);
        }

        #[actix_rt::test]
        async fn post_users_handler_maps_err_to_not_found() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::DbError(sqlx::Error::RowNotFound);
            mock_svc
                .expect_sync_create_user()
                .returning(move |_| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let usr = web::Json(User::default());
            let req = test::TestRequest::default().to_http_request();
            let res = post(usr, req, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), err_svc().to_string());
            assert_eq!(err.status_code().as_u16(), 404);
        }

        #[actix_rt::test]
        async fn post_users_handler_maps_err_to_internal() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::DbError(sqlx::Error::PoolTimedOut);
            mock_svc
                .expect_sync_create_user()
                .returning(move |_| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let usr = web::Json(User::default());
            let req = test::TestRequest::default().to_http_request();
            let res = post(usr, req, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), "Database Error");
            assert_eq!(err.status_code().as_u16(), 500);
        }

        // patch handler

        #[actix_rt::test]
        async fn patch_users_handler_works() {
            let user_id = Uuid::new_v4();
            let user_name = "my_name";
            let random = 74444;
            let path = format!("/v1/users/{}", user_id);

            let mut mock_svc = MockSvc::default();
            mock_svc
                .expect_sync_update_user()
                .returning(move |user_id, _caller, custom_data| {
                    let mut user = User::default();
                    user.id = Some(*user_id);
                    user.name = user_name.to_string();
                    user.custom_data = Some(sqlx::types::Json(custom_data));
                    Ok(user)
                });

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(user_id);
            let fields = web::Json(CustomData { random });
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::with_uri(path.as_ref()).to_http_request();
            let mut res: HttpResponse = patch(id, fields, req, auth, svc).await.unwrap();

            let user = res
                .take_body()
                .as_ref()
                .map(|b| match b {
                    Body::Bytes(x) => serde_json::from_slice::<'_, User>(x).ok(),
                    _ => None,
                })
                .flatten()
                .unwrap();

            let location = res.headers().get("Location").unwrap().to_str().unwrap();

            assert_eq!(user.id.unwrap(), user_id);
            assert_eq!(user.name, user_name);
            assert_eq!(user.custom_data.unwrap().random, random);
            assert_eq!(location, path);
            assert!(res.status().is_success());
        }

        #[actix_rt::test]
        async fn patch_users_handler_maps_err_to_unauthorized() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::Unauthorized;
            mock_svc
                .expect_sync_update_user()
                .returning(move |_, _, _| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(Uuid::new_v4());
            let custom_data = web::Json(CustomData::default());
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::default().to_http_request();
            let res = patch(id, custom_data, req, auth, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), err_svc().to_string());
            assert_eq!(err.status_code().as_u16(), 401);
        }

        #[actix_rt::test]
        async fn patch_users_handler_maps_err_to_not_found() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::DbError(sqlx::Error::RowNotFound);
            mock_svc
                .expect_sync_update_user()
                .returning(move |_, _, _| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(Uuid::new_v4());
            let custom_data = web::Json(CustomData::default());
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::default().to_http_request();
            let res = patch(id, custom_data, req, auth, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), err_svc().to_string());
            assert_eq!(err.status_code().as_u16(), 404);
        }

        #[actix_rt::test]
        async fn patch_users_handler_maps_err_to_internal() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::DbError(sqlx::Error::PoolTimedOut);
            mock_svc
                .expect_sync_update_user()
                .returning(move |_, _, _| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(Uuid::new_v4());
            let custom_data = web::Json(CustomData::default());
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::default().to_http_request();
            let res = patch(id, custom_data, req, auth, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), "Database Error");
            assert_eq!(err.status_code().as_u16(), 500);
        }

        // delete handler

        #[actix_rt::test]
        async fn delete_users_handler_works() {
            let user_id = Uuid::new_v4();
            let user_name = "my_name";
            let path = format!("/v1/users/{}", user_id);

            let mut mock_svc = MockSvc::default();
            mock_svc
                .expect_sync_delete_user()
                .returning(move |user_id, _caller_id| {
                    let mut user = User::default();
                    user.id = Some(*user_id);
                    user.name = user_name.to_string();
                    Ok(user)
                });

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(user_id);
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::with_uri(path.as_ref()).to_http_request();
            let mut res: HttpResponse = delete(id, req, auth, svc).await.unwrap();

            let user = res
                .take_body()
                .as_ref()
                .map(|b| match b {
                    Body::Bytes(x) => serde_json::from_slice::<'_, User>(x).ok(),
                    _ => None,
                })
                .flatten()
                .unwrap();

            let location = res.headers().get("Location").unwrap().to_str().unwrap();

            assert_eq!(user.id.unwrap(), user_id);
            assert_eq!(user.name, user_name);
            assert_eq!(location, path);
            assert!(res.status().is_success());
        }

        #[actix_rt::test]
        async fn delete_users_handler_maps_err_to_unauthorized() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::Unauthorized;
            mock_svc
                .expect_sync_delete_user()
                .returning(move |_, _| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(Uuid::new_v4());
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::default().to_http_request();
            let res = delete(id, req, auth, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), err_svc().to_string());
            assert_eq!(err.status_code().as_u16(), 401);
        }

        #[actix_rt::test]
        async fn delete_users_handler_maps_err_to_not_found() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::DbError(sqlx::Error::RowNotFound);
            mock_svc
                .expect_sync_delete_user()
                .returning(move |_, _| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(Uuid::new_v4());
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::default().to_http_request();
            let res = delete(id, req, auth, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), err_svc().to_string());
            assert_eq!(err.status_code().as_u16(), 404);
        }

        #[actix_rt::test]
        async fn delete_users_handler_maps_err_to_internal() {
            let mut mock_svc = MockSvc::default();
            let err_svc = || ServiceError::DbError(sqlx::Error::PoolTimedOut);
            mock_svc
                .expect_sync_delete_user()
                .returning(move |_, _| Err(err_svc()));

            let svc = web::Data::new(Arc::new(ServiceInjector::new(mock_svc)));
            let id = web::Path::from(Uuid::new_v4());
            let auth = CognitoInfo::disabled();
            let req = test::TestRequest::default().to_http_request();
            let res = delete(id, req, auth, svc).await.err().unwrap();
            let err = res.as_response_error();

            assert_eq!(err.to_string(), "Database Error");
            assert_eq!(err.status_code().as_u16(), 500);
        }
    }
}
