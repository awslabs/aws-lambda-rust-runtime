#[test]
fn test_calling_lambda_should_return_200() {
    let test_endpoint = std::env::var("TEST_ENDPOINT").expect("could not read TEST_ENDPOINT");
    let secret_token = std::env::var("SECRET_TOKEN").expect("could not read SECRET_TOKEN");
    let client = reqwest::blocking::Client::new();
    let res = client
        .get(test_endpoint)
        .header("Authorization", secret_token)
        .send()
        .expect("could not the request");
    assert_eq!(res.status(), 200);
}
