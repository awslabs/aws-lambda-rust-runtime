use crate::request::LambdaRequest;
use aws_lambda_events::{
    alb::AlbTargetGroupRequest,
    apigw::{ApiGatewayProxyRequest, ApiGatewayV2httpRequest, ApiGatewayWebsocketProxyRequest},
};
use serde::{de::Error, Deserialize};

const ERROR_CONTEXT: &str = "this function expects a JSON payload from Amazon API Gateway, Amazon Elastic Load Balancer, or AWS Lambda Function URLs, but the data doesn't match any of those services' events";

impl<'de> Deserialize<'de> for LambdaRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let content = match serde::__private::de::Content::deserialize(deserializer) {
            Ok(content) => content,
            Err(err) => return Err(err),
        };
        #[cfg(feature = "apigw_rest")]
        if let Ok(res) =
            ApiGatewayProxyRequest::deserialize(serde::__private::de::ContentRefDeserializer::<D::Error>::new(&content))
        {
            return Ok(LambdaRequest::ApiGatewayV1(res));
        }
        #[cfg(feature = "apigw_http")]
        if let Ok(res) = ApiGatewayV2httpRequest::deserialize(
            serde::__private::de::ContentRefDeserializer::<D::Error>::new(&content),
        ) {
            return Ok(LambdaRequest::ApiGatewayV2(res));
        }
        #[cfg(feature = "alb")]
        if let Ok(res) =
            AlbTargetGroupRequest::deserialize(serde::__private::de::ContentRefDeserializer::<D::Error>::new(&content))
        {
            return Ok(LambdaRequest::Alb(res));
        }
        #[cfg(feature = "apigw_websockets")]
        if let Ok(res) = ApiGatewayWebsocketProxyRequest::deserialize(serde::__private::de::ContentRefDeserializer::<
            D::Error,
        >::new(&content))
        {
            return Ok(LambdaRequest::WebSocket(res));
        }

        Err(Error::custom(ERROR_CONTEXT))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_apigw_rest() {
        let data = include_bytes!("../../lambda-events/src/fixtures/example-apigw-request.json");

        let req: LambdaRequest = serde_json::from_slice(data).expect("failed to deserialze apigw rest data");
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

        let req: LambdaRequest = serde_json::from_slice(data).expect("failed to deserialze apigw http data");
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

        let req: LambdaRequest = serde_json::from_slice(data).expect("failed to deserialze alb rest data");
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

        let req: LambdaRequest = serde_json::from_slice(data).expect("failed to deserialze apigw websocket data");
        match req {
            LambdaRequest::WebSocket(req) => {
                assert_eq!("CONNECT", req.request_context.event_type.unwrap());
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_error() {
        let err = serde_json::from_str::<LambdaRequest>("{\"command\": \"hi\"}").unwrap_err();

        assert_eq!(ERROR_CONTEXT, err.to_string());
    }
}
