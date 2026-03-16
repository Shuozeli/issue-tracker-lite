use tonic::service::Interceptor;

/// Interceptor that injects the x-user-id header into gRPC requests.
#[derive(Clone)]
pub struct UserInterceptor {
    user_id: String,
}

impl UserInterceptor {
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }
}

impl Interceptor for UserInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        request.metadata_mut().insert(
            "x-user-id",
            self.user_id
                .parse()
                .map_err(|_| tonic::Status::internal("invalid user id"))?,
        );
        Ok(request)
    }
}
