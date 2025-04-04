use crate::config::{chain_info_request_timeout, collectibles_request_timeout};
use crate::tests::main::setup_rocket;
use crate::utils::errors::{ApiError, ErrorDetails};
use crate::utils::http_client::{MockHttpClient, Request, Response};
use core::time::Duration;
use mockall::predicate::eq;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use serde_json::json;

#[rocket::async_test]
async fn collectibles() {
    let mut chain_request = Request::new(config_uri!("/v1/chains/{}/", 4));
    chain_request.timeout(Duration::from_millis(chain_info_request_timeout()));

    let mut mock_http_client = MockHttpClient::new();
    mock_http_client
        .expect_get()
        .times(1)
        .with(eq(chain_request))
        .return_once(move |_| {
            Ok(Response {
                status_code: 200,
                body: String::from(crate::tests::json::CHAIN_INFO_RINKEBY),
            })
        });

    let mut collectibles_request = Request::new(String::from("https://safe-transaction.rinkeby.staging.gnosisdev.com/api/v1/safes/0x1230B3d59858296A31053C1b8562Ecf89A2f888b/collectibles/?trusted=false&exclude_spam=true"));
    collectibles_request.timeout(Duration::from_millis(collectibles_request_timeout()));
    mock_http_client
        .expect_get()
        .times(1)
        .with(eq(collectibles_request))
        .return_once(move |_| {
            Ok(Response {
                status_code: 200,
                body: String::from(crate::tests::json::COLLECTIBLES_PAGE),
            })
        });

    let client = Client::tracked(setup_rocket(
        mock_http_client,
        routes![super::super::routes::get_collectibles],
    ))
    .await
    .expect("valid rocket instance");
    let response = {
        let mut response = client
            .get("/v1/chains/4/safes/0x1230B3d59858296A31053C1b8562Ecf89A2f888b/collectibles");
        response.add_header(Header::new("Host", "test.gnosis.io"));
        response.dispatch().await
    };

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string().await.unwrap(),
        crate::tests::json::COLLECTIBLES_PAGE
    );
}

#[rocket::async_test]
async fn collectibles_not_found() {
    let backend_error_json = json!({"details": "Not found"}).to_string();
    let error = ErrorDetails {
        code: 1337,
        message: Some(backend_error_json.clone()),
        arguments: None,
        debug: None,
    };

    let mut chain_request = Request::new(config_uri!("/v1/chains/{}/", 4));
    chain_request.timeout(Duration::from_millis(chain_info_request_timeout()));

    let mut mock_http_client = MockHttpClient::new();
    mock_http_client
        .expect_get()
        .times(1)
        .with(eq(chain_request))
        .return_once(move |_| {
            Ok(Response {
                status_code: 200,
                body: String::from(crate::tests::json::CHAIN_INFO_RINKEBY),
            })
        });

    let mut collectibles_request = Request::new(String::from("https://safe-transaction.rinkeby.staging.gnosisdev.com/api/v1/safes/0x1230B3d59858296A31053C1b8562Ecf89A2f888b/collectibles/?trusted=false&exclude_spam=true"));
    collectibles_request.timeout(Duration::from_millis(collectibles_request_timeout()));
    mock_http_client
        .expect_get()
        .times(1)
        .with(eq(collectibles_request))
        .return_once(move |_| {
            Err(ApiError::from_http_response(&Response {
                status_code: 404,
                body: backend_error_json.clone(),
            }))
        });

    let client = Client::tracked(setup_rocket(
        mock_http_client,
        routes![super::super::routes::get_collectibles],
    ))
    .await
    .expect("valid rocket instance");
    let response = {
        let mut response = client
            .get("/v1/chains/4/safes/0x1230B3d59858296A31053C1b8562Ecf89A2f888b/collectibles");
        response.add_header(Header::new("Host", "test.gnosis.io"));
        response.dispatch().await
    };

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.into_string().await.unwrap(),
        serde_json::to_string(&error).unwrap()
    );
}
