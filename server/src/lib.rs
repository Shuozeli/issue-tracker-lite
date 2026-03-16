pub mod db;
pub mod domain;
pub mod service;

pub mod proto {
    tonic::include_proto!("issuetracker.v1");
}

pub mod identity_proto {
    tonic::include_proto!("identity.v1");
}
