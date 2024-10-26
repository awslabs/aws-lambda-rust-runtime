use crate::request::LambdaRequest;
#[cfg(feature = "alb")]
use aws_lambda_events::alb::AlbTargetGroupRequest;
#[cfg(feature = "apigw_rest")]
use aws_lambda_events::apigw::ApiGatewayProxyRequest;
#[cfg(feature = "apigw_http")]
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
#[cfg(feature = "apigw_websockets")]
use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use aws_lambda_json_impl::JsonDeserializer;
use serde::{de::Error, Deserialize};
use tracing::debug;

const ERROR_CONTEXT: &str = "this function expects a JSON payload from Amazon API Gateway, Amazon Elastic Load Balancer, or AWS Lambda Function URLs, but the data doesn't match any of those services' events";

#[cfg(feature = "pass_through")]
const PASS_THROUGH_ENABLED: bool = true;


// simd_json and LambdaRequest don't sit well together due to how the request variant is discovered
// 
// To get there, we have to get a bit hacky - we panic if the deserializer we have been offered is not
// a simd_json::Deserializer (please someone show me a better way if there is one!) because we need the
// the Tape from it - THEN we can do some peeking to see what's there and try to deduce the type we are
// looking for.
//
impl<'de> Deserialize<'de> for LambdaRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        
        debug!("Establishing the event type");

        #[cfg(feature = "apigw_rest")]
        if let (Some(rc), true) = (v.get_object("request_context"),v.contains_key("http_method")) {
            if rc.get("http").is_none() {
                debug!("Parsing REST API request");
                let res: ApiGatewayProxyRequest = t.deserialize().map_err(Error::custom)?;
                return Ok(LambdaRequest::ApiGatewayV1(res));
            }
        }
        #[cfg(feature = "apigw_http")]
        if let Some(rc) = v.get_object("request_context") {
            if rc.get("http").is_some() {
                debug!("Parsing HTTP API request");
                let res: ApiGatewayV2httpRequest = t.deserialize().map_err(Error::custom)?;
                return Ok(LambdaRequest::ApiGatewayV2(res));
            }
        }
        #[cfg(feature = "alb")]
        if let Some(rc) = v.get_object("request_context") {
            if rc.get("elb").is_some() {
                debug!("Parsing ALB request");
                let res: AlbTargetGroupRequest = t.deserialize().map_err(Error::custom)?;
                return Ok(LambdaRequest::Alb(res));
            }
        }
        #[cfg(feature = "apigw_websockets")]
        if v.contains_key("connected_at") {
            debug!("Parsing WebSocket request");
            let res: ApiGatewayWebsocketProxyRequest = t.deserialize().map_err(Error::custom)?;
            return Ok(LambdaRequest::WebSocket(res));
        }

        #[cfg(feature = "pass_through")]
        if PASS_THROUGH_ENABLED {
            debug!("Defaulting to pass_through");
            return Ok(LambdaRequest::PassThrough(data.to_string()));
        }

        debug!("Failed to find a candidate request type");
        Err(Error::custom(ERROR_CONTEXT))

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_apigw_rest() {
        let mut data = include_bytes!("../../../lambda-events/src/fixtures/example-apigw-request.json").to_vec();

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data.as_mut_slice()).expect("failed to deserialize apigw rest data");
        match req {
            LambdaRequest::ApiGatewayV1(req) => {
                assert_eq!("12345678912", req.request_context.account_id.unwrap());
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_apigw_http() {
        let mut data = include_bytes!("../../../lambda-events/src/fixtures/example-apigw-v2-request-iam.json").to_vec();

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data.as_mut_slice()).expect("failed to deserialize apigw http data");
        match req {
            LambdaRequest::ApiGatewayV2(req) => {
                assert_eq!("123456789012", req.request_context.account_id.unwrap());
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_sam_rest() {
        let mut data = include_bytes!("../../../lambda-events/src/fixtures/example-apigw-sam-rest-request.json").to_vec();

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data.as_mut_slice()).expect("failed to deserialize SAM rest data");
        match req {
            LambdaRequest::ApiGatewayV1(req) => {
                assert_eq!("123456789012", req.request_context.account_id.unwrap());
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_sam_http() {
        let mut data = include_bytes!("../../../lambda-events/src/fixtures/example-apigw-sam-http-request.json").to_vec();

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data.as_mut_slice()).expect("failed to deserialize SAM http data");
        match req {
            LambdaRequest::ApiGatewayV2(req) => {
                assert_eq!("123456789012", req.request_context.account_id.unwrap());
            }
            other => panic!("unexpected request variant: {:?}", other),
        }
    }

    #[test]
    fn test_deserialize_alb() {
        let mut data = include_bytes!(
            "../../../lambda-events/src/fixtures/example-alb-lambda-target-request-multivalue-headers.json"
        ).to_vec();
        
        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data.as_mut_slice()).expect("failed to deserialize alb rest data");
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
        let mut data =
            include_bytes!("../../../lambda-events/src/fixtures/example-apigw-websocket-request-without-method.json").to_vec();

        let req: LambdaRequest = aws_lambda_json_impl::from_slice(data.as_mut_slice()).expect("failed to deserialize apigw websocket data");
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
        let mut data = include_bytes!("../../../lambda-events/src/fixtures/example-bedrock-agent-runtime-event.json").to_vec();

        let req: LambdaRequest =
        deserialize(data.as_mut_slice()).expect("failed to deserialize bedrock agent request data");
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
