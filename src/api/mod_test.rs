#[cfg(test)]
mod tests {
    use crate::api::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_check() -> Result<(), Box<dyn std::error::Error>> {
        let app = test::init_service(App::new().configure(configure)).await;
        let req = test::TestRequest::get().uri("/api/v1/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
        Ok(())
    }
}
