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
}
