
## How to compile and run the examples

1. Create a Lambda function called _RuntimeTest_ in AWS with a custom runtime and no code.

2. Compile all examples

```
cargo build --release --target x86_64-unknown-linux-musl --examples
```
3. Prepare the package for the example you want to test, e.g.
```
cp ./target/x86_64-unknown-linux-musl/release/examples/hello ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
```
4. Upload the package to AWS Lambda
```
aws lambda update-function-code --region us-east-1 --function-name RuntimeTest --zip-file fileb://lambda.zip
```
_Feel free to replace the names and IDs with your own values._

## basic.rs

**Deployment**:
```bash
cp ./target/x86_64-unknown-linux-musl/release/examples/basic ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
aws lambda update-function-code --region us-east-1 --function-name RuntimeTest --zip-file fileb://lambda.zip
```

**Test event JSON (success)**:
```json
{ "command": "do something" }
```

Sample response:
```json
{
  "msg": "Command do something executed.",
  "req_id": "67a038e4-dc19-4adf-aa32-5ba09312c6ca"
}
```

**Test event JSON (error)**:
```json
{ "foo": "do something" }
```

Sample response:
```json
{
  "errorType": "Runtime.ExitError",
  "errorMessage": "RequestId: 586366df-f271-4e6e-9c30-b3dcab30f4e8 Error: Runtime exited with error: exit status 1"
}
```
The runtime could not deserialize our invalid input, but it did not give a detailed explanation why the error occurred in the response. More details appear in the CloudWatch log:
```
START RequestId: 6e667f61-c5d4-4b07-a60f-cd1ab339c35f Version: $LATEST
Error: Error("missing field `command`", line: 1, column: 22)
END RequestId: 6e667f61-c5d4-4b07-a60f-cd1ab339c35f
REPORT RequestId: 6e667f61-c5d4-4b07-a60f-cd1ab339c35f	Duration: 36.34 ms	Billed Duration: 100 ms	Memory Size: 128 MB	Max Memory Used: 10 MB	
RequestId: 6e667f61-c5d4-4b07-a60f-cd1ab339c35f Error: Runtime exited with error: exit status 1
Runtime.ExitError
```

 See _error-handling.rs_ example for more error handling options.

## macro.rs

The most basic example using `#[lambda]` macro to reduce the amount of boilerplate code.

**Deployment**:
```bash
cp ./target/x86_64-unknown-linux-musl/release/examples/macro ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
aws lambda update-function-code --region us-east-1 --function-name RuntimeTest --zip-file fileb://lambda.zip
```

**Test event JSON**:
```json
{ "foo": "bar" }
```

Sample response:
```json
{
  "foo": "bar"
}
```

## error-handling.rs

Errors are logged by the runtime only if `log` is initialized by the handler.
These examples use `simple_logger`, but you can use any other provider that works with `log`.
```
simple_logger::init_with_level(log::Level::Debug)?;
```

**Deployment**:
```bash
cp ./target/x86_64-unknown-linux-musl/release/examples/error-handling ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
aws lambda update-function-code --region us-east-1 --function-name RuntimeTest --zip-file fileb://lambda.zip
```

The following input/output examples correspond to different `match` arms in the handler of `error-handling.rs`.

#### Invalid event JSON

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

#### A simple text-only error

Test event JSON:
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

#### A custom error with JSON output for Display trait

Test event JSON:
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

#### A 3rd party error from _std::fs::File::open_

Test event JSON:
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

#### Handler panic

Test event JSON:
```json
{
  "event_type": "Panic"
}
```

Lambda output:
```
{
  "errorType": "Runtime.ExitError",
  "errorMessage": "RequestId: 2d579019-07f7-409a-a6e6-af7725253307 Error: Runtime exited with error: exit status 101"
}
```

CloudWatch records:
```
START RequestId: 2d579019-07f7-409a-a6e6-af7725253307 Version: $LATEST
thread 'main' panicked at 'explicit panic', lambda/examples/error-handling.rs:87:13
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
END RequestId: 2d579019-07f7-409a-a6e6-af7725253307
REPORT RequestId: 2d579019-07f7-409a-a6e6-af7725253307	Duration: 43.40 ms	Billed Duration: 100 ms	Memory Size: 128 MB	Max Memory Used: 27 MB	Init Duration: 23.15 ms	
RequestId: 2d579019-07f7-409a-a6e6-af7725253307 Error: Runtime exited with error: exit status 101
Runtime.ExitError
```

#### A response to a successful Lambda execution

Test event JSON:
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