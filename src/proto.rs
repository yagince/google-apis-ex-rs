pub(crate) const TLS_CERT: &[u8] = include_bytes!("google.pem");

pub mod google {
    pub mod api {
        include!("proto/google.api.rs");
    }
    pub mod r#type {
        include!("proto/google.r#type.rs");
    }
    pub mod iam {
        pub mod v1 {
            include!("proto/google.iam.v1.rs");
        }
    }
    pub mod storage {
        pub mod v2 {
            include!("proto/google.storage.v2.rs");
        }
    }
    pub mod pubsub {
        pub mod v1 {
            include!("proto/google.pubsub.v1.rs");
        }
    }
    pub mod cloud {
        pub mod kms {
            pub mod v1 {
                include!("proto/google.cloud.kms.v1.rs");
            }
        }
    }
}
