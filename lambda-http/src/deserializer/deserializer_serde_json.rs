use crate::request::LambdaRequest;
#[cfg(feature = "alb")]
use aws_lambda_events::alb::AlbTargetGroupRequest;
#[cfg(feature = "apigw_rest")]
use aws_lambda_events::apigw::ApiGatewayProxyRequest;
#[cfg(feature = "apigw_http")]
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
#[cfg(feature = "apigw_websockets")]
use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use serde::{de::Error, Deserialize};
use aws_lambda_json_impl::RawValue;

const ERROR_CONTEXT: &str = "this function expects a JSON payload from Amazon API Gateway, Amazon Elastic Load Balancer, or AWS Lambda Function URLs, but the data doesn't match any of those services' events";

#[cfg(feature = "pass_through")]
const PASS_THROUGH_ENABLED: bool = true;

impl<'de> Deserialize<'de> for LambdaRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_value: Box<RawValue> = Box::deserialize(deserializer)?;
        let data = raw_value.get();

        #[cfg(feature = "apigw_rest")]
        if let Ok(res) = aws_lambda_json_impl::from_str::<ApiGatewayProxyRequest>(data) {
            return Ok(LambdaRequest::ApiGatewayV1(res));
        }
        #[cfg(feature = "apigw_http")]
        if let Ok(res) = aws_lambda_json_impl::from_str::<ApiGatewayV2httpRequest>(data) {
            return Ok(LambdaRequest::ApiGatewayV2(res));
        }
        #[cfg(feature = "alb")]
        if let Ok(res) = aws_lambda_json_impl::from_str::<AlbTargetGroupRequest>(data) {
            return Ok(LambdaRequest::Alb(res));
        }
        #[cfg(feature = "apigw_websockets")]
        if let Ok(res) = aws_lambda_json_impl::from_str::<ApiGatewayWebsocketProxyRequest>(data) {
            return Ok(LambdaRequest::WebSocket(res));
        }
        #[cfg(feature = "pass_through")]
        if PASS_THROUGH_ENABLED {
            return Ok(LambdaRequest::PassThrough(data.to_string()));
        }

        Err(Error::custom(ERROR_CONTEXT))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_apigw_rest() {
        let data = include_bytes!("../../../lambda-events/src/fixtures/example-apigw-request.json");

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data).expect("failed to deserialize apigw rest data");
        match req {
            LambdaRequest::ApiGatewayV1(req) => {
                assert_eq!("12345678912", req.request_context.account_id.unwrap());
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_apigw_http() {
        let data = include_bytes!("../../../lambda-events/src/fixtures/example-apigw-v2-request-iam.json");

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data).expect("failed to deserialize apigw http data");
        match req {
            LambdaRequest::ApiGatewayV2(req) => {
                assert_eq!("123456789012", req.request_context.account_id.unwrap());
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_sam_rest() {
        let data = include_bytes!("../../../lambda-events/src/fixtures/example-apigw-sam-rest-request.json");

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data).expect("failed to deserialize SAM rest data");
        match req {
            LambdaRequest::ApiGatewayV1(req) => {
                assert_eq!("123456789012", req.request_context.account_id.unwrap());
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_sam_http() {
        let data = include_bytes!("../../../lambda-events/src/fixtures/example-apigw-sam-http-request.json");

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data).expect("failed to deserialize SAM http data");
        match req {
            LambdaRequest::ApiGatewayV2(req) => {
                assert_eq!("123456789012", req.request_context.account_id.unwrap());
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_alb() {
        let data = include_bytes!(
            "../../../lambda-events/src/fixtures/example-alb-lambda-target-request-multivalue-headers.json"
        );

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data).expect("failed to deserialize alb rest data");
        match req {
            LambdaRequest::Alb(req) => {
                assert_eq!(
                    "arn:aws:elasticloadbalancing:us-east-1:123456789012:targetgroup/lambda-target/abcdefgh",
                    req.request_context.elb.target_group_arn.unwrap()
                );
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_apigw_websocket() {
        let data =
            include_bytes!("../../../lambda-events/src/fixtures/example-apigw-websocket-request-without-method.json");

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data).expect("failed to deserialize apigw websocket data");
        match req {
            LambdaRequest::WebSocket(req) => {
                assert_eq!("CONNECT", req.request_context.event_type.unwrap());
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    #[cfg(feature = "pass_through")]
    fn test_deserialize_bedrock_agent() {
        let data = include_bytes!("../../../lambda-events/src/fixtures/example-bedrock-agent-runtime-event.json");

        let req: LambdaRequest =
            aws_lambda_json_impl::from_slice(data).expect("failed to deserialize bedrock agent request data");
        match req {
            LambdaRequest::PassThrough(req) => {
                assert_eq!(String::from_utf8_lossy(data), req);
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    #[cfg(feature = "pass_through")]
    fn test_deserialize_sqs() {
        let data = include_bytes!("../../../lambda-events/src/fixtures/example-sqs-event.json");

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data).expect("failed to deserialize sqs event data");
        match req {
            LambdaRequest::PassThrough(req) => {
                assert_eq!(String::from_utf8_lossy(data), req);
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }
}
