use muon::{http::Method, ProtonRequest};

pub trait ToProtonRequest {
    fn to_get_request(&self) -> ProtonRequest;
    fn to_post_request(&self) -> ProtonRequest;
    fn to_put_request(&self) -> ProtonRequest;
    fn to_delete_request(&self) -> ProtonRequest;
}

impl ToProtonRequest for String {
    fn to_get_request(&self) -> ProtonRequest {
        ProtonRequest::new(Method::GET, self)
    }

    fn to_post_request(&self) -> ProtonRequest {
        ProtonRequest::new(Method::POST, self)
    }

    fn to_put_request(&self) -> ProtonRequest {
        ProtonRequest::new(Method::PUT, self)
    }

    fn to_delete_request(&self) -> ProtonRequest {
        ProtonRequest::new(Method::DELETE, self)
    }
}
