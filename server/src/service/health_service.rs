use tonic::{Request, Response, Status};

use crate::proto::health_service_server::HealthService;
use crate::proto::{PingRequest, PingResponse};

pub struct HealthServiceImpl;

#[tonic::async_trait]
impl HealthService for HealthServiceImpl {
    async fn ping(&self, _request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        Ok(Response::new(PingResponse {
            message: "pong".to_string(),
        }))
    }
}
