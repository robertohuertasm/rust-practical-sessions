/// Executes the code in case the user is authorized to access the resource.
macro_rules! authorized {
    ($user_id: expr, $caller_id: expr) => {{
        if let Some(caller_id) = $caller_id {
            if $user_id.to_string() != caller_id {
                return Err(ServiceError::Unauthorized);
            }
        }
    }};
}

/// Expands to the typical error request handling
macro_rules! svc_err {
    ($err: expr) => {{
        match $err {
            ServiceError::Unauthorized => Err(error::ErrorUnauthorized($err)),
            ServiceError::DbError(sqlx::Error::RowNotFound) => Err(error::ErrorNotFound($err)),
            ServiceError::DbError(_) => Err(error::ErrorInternalServerError("Database Error")),
        }
    }};
}

/// Handles the service response
macro_rules! svc_response {
    ($svc_call: expr, $response_type: expr, $location: expr, $err_msg: expr) => {{
        match $svc_call {
            Ok(svc_resp) => {
                let response = $response_type.header("Location", $location).json(svc_resp);
                Ok(response)
            }
            Err(err) => {
                log::error!("{}: {}", $err_msg, err);
                svc_err!(err)
            }
        }
    }};
}

/// Wraps the result of the query into a measured block and returns the result
macro_rules! measure_query {
    ($method: literal, $block: block) => {{
        let now = Instant::now();
        let result = $block;
        log::debug!(
            "{} user query & deserialization took {} ms",
            $method,
            now.elapsed().as_millis()
        );
        result
    }};
}
