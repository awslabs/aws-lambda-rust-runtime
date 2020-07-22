
## How to compile and run the examples

1. Compile the example you want to run

```
cargo build --release --target x86_64-unknown-linux-musl --example error-handling
```
2. Prepare the package
```
cp ./target/x86_64-unknown-linux-musl/release/examples/error-handling ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
```
3. Upload to AWS Lambda
```
aws lambda update-function-code --region us-east-1 --function-name ReturnValue --zip-file fileb://lambda.zip
```
_Remember to replace the names and IDs with your own values._


## Error handling examples for aws-lambda-rust-runtime

#### Error logging by the runtime

Errors are logged by the runtime only if `log` is initialised by the handler.
These examples use `simple_logger`, but you can use any other provider that works with `log`.
```
simple_logger::init_with_level(log::Level::Debug)?;
```

#### Sample log output

The following input/output examples correspond to different `match` arms in the handler of `error-handling.rs`.

### Invalid JSON input

Test input:
```json
{
  "event_type": "WrongType"
}
```

Lambda output:
```
{
  "errorType": "alloc::boxed::Box<dyn std::error::Error+core::marker::Sync+core::marker::Send>",
  "errorMessage": "unknown variant `WrongType`, expected one of `Response`, `ExternalError`, `SimpleError`, `CustomError`"
}
```

CloudWatch records:
```
START RequestId: b98e07c6-e2ba-4ca6-9968-d0b94729ddba Version: $LATEST
2020-07-21 04:28:52,630 ERROR [lambda] unknown variant `WrongType`, expected one of `Response`, `ExternalError`, `SimpleError`, `CustomError`
END RequestId: b98e07c6-e2ba-4ca6-9968-d0b94729ddba
REPORT RequestId: b98e07c6-e2ba-4ca6-9968-d0b94729ddba	Duration: 2.06 ms	Billed Duration: 100 ms	Memory Size: 128 MB	Max Memory Used: 28 MB	Init Duration: 33.67 ms	
```

### A simple text-only error

Test input:
```json
{
  "event_type": "SimpleError"
}
```

Lambda output:
```
{
  "errorType": "alloc::boxed::Box<dyn std::error::Error+core::marker::Sync+core::marker::Send>",
  "errorMessage": "A simple error as requested!"
}
```

CloudWatch records:
```
START RequestId: 77c66dbf-bd60-4f77-8453-682d0bceba91 Version: $LATEST
2020-07-21 04:35:28,044 ERROR [lambda] A simple error as requested!
END RequestId: 77c66dbf-bd60-4f77-8453-682d0bceba91
REPORT RequestId: 77c66dbf-bd60-4f77-8453-682d0bceba91	Duration: 0.98 ms	Billed Duration: 100 ms	Memory Size: 128 MB	Max Memory Used: 28 MB	
```

### A custom error with JSON output for Display trait

Test input:
```json
{
  "event_type": "CustomError"
}
```

Lambda output:
```
{
  "errorType": "alloc::boxed::Box<dyn std::error::Error+core::marker::Sync+core::marker::Send>",
  "errorMessage": "{\"is_authenticated\":false,\"msg\":\"A custom error as requested!\",\"req_id\":\"b46b0588-1383-4224-bc7a-42b0d61930c1\"}"
}
```

CloudWatch records:
```
START RequestId: b46b0588-1383-4224-bc7a-42b0d61930c1 Version: $LATEST
2020-07-21 04:39:00,133 ERROR [lambda] {"is_authenticated":false,"msg":"A custom error as requested!","req_id":"b46b0588-1383-4224-bc7a-42b0d61930c1"}
END RequestId: b46b0588-1383-4224-bc7a-42b0d61930c1
REPORT RequestId: b46b0588-1383-4224-bc7a-42b0d61930c1	Duration: 0.91 ms	Billed Duration: 100 ms	Memory Size: 128 MB	Max Memory Used: 29 MB	
```

### A 3rd party error from _std::fs::File::open_

Test input:
```json
{
  "event_type": "ExternalError"
}
```

Lambda output:
```
{
  "errorType": "alloc::boxed::Box<dyn std::error::Error+core::marker::Sync+core::marker::Send>",
  "errorMessage": "No such file or directory (os error 2)"
}
```

CloudWatch records:
```
START RequestId: 893d24e5-cb79-4f6f-bae0-36304c62e9da Version: $LATEST
2020-07-21 04:43:56,254 ERROR [lambda] No such file or directory (os error 2)
END RequestId: 893d24e5-cb79-4f6f-bae0-36304c62e9da
REPORT RequestId: 893d24e5-cb79-4f6f-bae0-36304c62e9da	Duration: 1.15 ms	Billed Duration: 100 ms	Memory Size: 128 MB	Max Memory Used: 29 MB	
```

### A response to a successful Lambda execution

Test input:
```json
{
  "event_type": "Response"
}
```

Lambda output:
```
{
  "msg": "OK",
  "req_id": "9752a3ad-6566-44e4-aafd-74db1fd4f361"
}
```

CloudWatch records:
```
START RequestId: 9752a3ad-6566-44e4-aafd-74db1fd4f361 Version: $LATEST
END RequestId: 9752a3ad-6566-44e4-aafd-74db1fd4f361
REPORT RequestId: 9752a3ad-6566-44e4-aafd-74db1fd4f361	Duration: 0.89 ms	Billed Duration: 100 ms	Memory Size: 128 MB	Max Memory Used: 29 MB	
```