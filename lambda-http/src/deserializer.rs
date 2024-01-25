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
use serde_json::value::Value;

const ERROR_CONTEXT: &str = "this function expects a JSON payload from Amazon API Gateway, Amazon Elastic Load Balancer, or AWS Lambda Function URLs, but the data doesn't match any of those services' events";

//
// This enum duplicates the purpose of "request::RequestOrigin" - i.e. the
// "from_value" could be implemented on that enum. But in the interests of keeping
// all serde references in this module, we use a separate private enum. 
//
enum RequestType {
    #[cfg(feature = "apigw_rest")]
    ApiGatewayProxyRequest,
    #[cfg(feature = "apigw_http")]
    ApiGatewayV2httpRequest,
    #[cfg(feature = "alb")]
    AlbTargetGroupRequest,
    #[cfg(feature = "apigw_websockets")]
    ApiGatewayWebsocketProxyRequest
}

impl RequestType {
    //
    // The type determinants were established by what errors Serde deserialization threw
    // in the previous implementation. Therefore we are being a little arbitrary here but
    // this should significantly improve performance since we don't attempt the full
    // deserialization of each kind, we simply look for fields in the already-partially
    // deserialized data.
    //
    // They are also in a very specific order - API Gateway V1 requests will deserialize with
    // just "httpMethod" in the context, but websocket requests contain that too.
    //
    fn from_value(value: &Value) -> Option<Self> {
        #[cfg(feature = "apigw_websockets")]
        if value.pointer("/requestContext/connectedAt").is_some() { return Some(Self::ApiGatewayWebsocketProxyRequest) }
        #[cfg(feature = "apigw_rest")]
        if value.pointer("/requestContext/httpMethod").is_some() { return Some(Self::ApiGatewayProxyRequest) }
        #[cfg(feature = "apigw_http")]
        if value.pointer("/requestContext/http").is_some() { return Some(Self::ApiGatewayV2httpRequest) }
        #[cfg(feature = "alb")]
        if value.pointer("/requestContext/elb").is_some() { return Some(Self::AlbTargetGroupRequest) }
        None
    }
}


#[cfg(feature = "pass_through")]
const PASS_THROUGH_ENABLED: bool = true;

impl<'de> Deserialize<'de> for LambdaRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = Value::deserialize(deserializer)?;
        match RequestType::from_value(&data) {
            #[cfg(feature = "apigw_rest")]
            Some(RequestType::ApiGatewayProxyRequest) => serde_json::from_value::<ApiGatewayProxyRequest>(data)
                .map(|r| LambdaRequest::ApiGatewayV1(r)),
            #[cfg(feature = "apigw_http")]
            Some(RequestType::ApiGatewayV2httpRequest) => serde_json::from_value::<ApiGatewayV2httpRequest>(data)
                .map(|r| LambdaRequest::ApiGatewayV2(r)),
            #[cfg(feature = "alb")]
            Some(RequestType::AlbTargetGroupRequest) => serde_json::from_value::<AlbTargetGroupRequest>(data)
                .map(|r| LambdaRequest::Alb(r)),
            #[cfg(feature = "apigw_websockets")]
            Some(RequestType::ApiGatewayWebsocketProxyRequest) => serde_json::from_value::<ApiGatewayWebsocketProxyRequest>(data)
                .map(|r| LambdaRequest::WebSocket(r)),
            None => {
                #[cfg(feature = "pass_through")]
                if PASS_THROUGH_ENABLED {
                    return Ok(LambdaRequest::PassThrough(data.to_string()));
                }
                Err(Error::custom(ERROR_CONTEXT))
            },
        }.map_err(|_| Error::custom(ERROR_CONTEXT))
 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_apigw_rest() {
        let data = include_bytes!("../../lambda-events/src/fixtures/example-apigw-request.json");

        let req: LambdaRequest = serde_json::from_slice(data).expect("failed to deserialize apigw rest data");
        match req {
            LambdaRequest::ApiGatewayV1(req) => {
                assert_eq!("12345678912", req.request_context.account_id.unwrap());
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_apigw_http() {
        let data = include_bytes!("../../lambda-events/src/fixtures/example-apigw-v2-request-iam.json");

        let req: LambdaRequest = serde_json::from_slice(data).expect("failed to deserialize apigw http data");
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
            "../../lambda-events/src/fixtures/example-alb-lambda-target-request-multivalue-headers.json"
        );

        let req: LambdaRequest = serde_json::from_slice(data).expect("failed to deserialize alb rest data");
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
            include_bytes!("../../lambda-events/src/fixtures/example-apigw-websocket-request-without-method.json");

        let req: LambdaRequest = serde_json::from_slice(data).expect("failed to deserialize apigw websocket data");
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
        let data = include_bytes!("../../lambda-events/src/fixtures/example-bedrock-agent-runtime-event.json");

        let req: LambdaRequest =
            serde_json::from_slice(data).expect("failed to deserialize bedrock agent request data");
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
        let data = include_bytes!("../../lambda-events/src/fixtures/example-sqs-event.json");

        let req: LambdaRequest = serde_json::from_slice(data).expect("failed to deserialize sqs event data");
        match req {
            LambdaRequest::PassThrough(req) => {
                assert_eq!(String::from_utf8_lossy(data), req);
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }
}
