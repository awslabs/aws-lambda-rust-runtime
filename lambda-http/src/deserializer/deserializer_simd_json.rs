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
        
        // We can't do this because D is not enforced to be 'static
        //if TypeId::of::<D>() != TypeId::of::<JsonDeserializer<'_>>() {panic!("Deserializer must be simd_json::Deserializer")};

        //
        // There is a future feature of simd_json being considered that is very similar to serde_json's RawValue - effectively a
        // Deserializer-implementation-specific way of introspecting/peeking at the parsed JSON before continuing with deserialization.
        // That's what the serde_json implementation of this uses, but, for now we have to resort to some wildly unsafe code in order
        // to cast the incoming D to &mut simd_json::Deserializer. AND we don't "forget" the incoming value either because we need THAT
        // to pass to our sub-deserialize attempts!
        //
        // HOWEVER... IF we are here, then someone has built this code with the simd_json feature enabled - which means that all KNOWN 
        // invokers have ALSO been built with simd_json enabled - which means that this is, in fact, "safe"... IF users ONLY exploit the 
        // standard invokers (i.e. the Lambda Runtime or the standard entry-point functions in aws_lambda_json_impl).
        // 
        // If not ... well, expect bad things to happen!
        //
        debug!("Deserializing event into some sort of HTTP event - going unsafe...");
        let d = unsafe { std::ptr::read_unaligned(&deserializer as *const _ as *mut &mut JsonDeserializer<'de>) };
//      std::mem::forget(deserializer); We need deserializer - otherwise we have to Box the Deserialize dyn when casting back and using for deserialize calls


        debug!("Getting the value from d: {:?}", d);
        let v = d.as_value();
        debug!("Establishing the event type");

        let (http_context, alb_context, websocket_context) = if let Some(rc) = v.get("requestContext") {
            (rc.contains_key("http"), rc.contains_key("elb"), rc.contains_key("connectedAt"))
        } else {
            (false, false, false)
        };
        debug!("State of d before restart: {:?}", d);
        d.restart();
        debug!("State of d after restart: {:?}", d);
        
        #[cfg(feature = "apigw_rest")]
        if !(http_context || alb_context || websocket_context) {
            debug!("Parsing REST API request");
            return ApiGatewayProxyRequest::deserialize(deserializer).map_err(Error::custom).map(LambdaRequest::ApiGatewayV1);
        }

        #[cfg(feature = "apigw_http")]
        if http_context {
            debug!("Parsing HTTP API request");
            return ApiGatewayV2httpRequest::deserialize(deserializer).map_err(Error::custom).map(LambdaRequest::ApiGatewayV2);
        }

        #[cfg(feature = "alb")]
        if alb_context {
            debug!("Parsing ALB request");
            return AlbTargetGroupRequest::deserialize(deserializer).map_err(Error::custom).map(LambdaRequest::Alb);   
        }

        #[cfg(feature = "apigw_websockets")]
        if websocket_context {
            debug!("Parsing WebSocket request");
            return ApiGatewayWebsocketProxyRequest::deserialize(deserializer).map_err(Error::custom).map(LambdaRequest::WebSocket); 
        }

/* Can't support this yet
        #[cfg(feature = "pass_through")]
        if PASS_THROUGH_ENABLED {
            debug!("Defaulting to pass_through");
            return Ok(LambdaRequest::PassThrough(data.to_string()));
        }
 */
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
