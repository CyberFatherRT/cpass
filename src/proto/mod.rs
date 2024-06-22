use tonic::include_file_descriptor_set;

pub mod auth;
pub mod pass;

pub mod auth_proto {
    tonic::include_proto!("auth");
}

pub mod pass_proto {
    tonic::include_proto!("pass");
}

pub mod types {
    tonic::include_proto!("types");
}

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = include_file_descriptor_set!("cpass_descriptor");
