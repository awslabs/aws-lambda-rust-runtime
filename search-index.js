var searchIndex = JSON.parse('{\
"extension_fn":{"doc":"","t":[5,5],"n":["main","my_extension"],"q":["extension_fn",""],"d":["",""],"i":[0,0],"f":[[[],["result",4,[["error",6]]]],[[["lambdaevent",3]]]],"p":[]},\
"extension_trait":{"doc":"","t":[3,11,11,11,11,11,11,12,5,11,11,11,11],"n":["MyExtension","borrow","borrow_mut","call","default","from","into","invoke_count","main","poll_ready","try_from","try_into","type_id"],"q":["extension_trait","","","","","","","","","","","",""],"d":["","","","","","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","","","","","",""],"i":[0,1,1,1,1,1,1,1,0,1,1,1,1],"f":[null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0],["lambdaevent",3]]],[[],["myextension",3]],[[]],[[]],null,[[],["result",4,[["error",6]]]],[[["",0],["context",3]],["poll",4,[["result",4]]]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]]],"p":[[3,"MyExtension"]]},\
"http_fn":{"doc":"","t":[5,5],"n":["handler","main"],"q":["http_fn",""],"d":["",""],"i":[0,0],"f":[[[["request",6]]],[[],["result",4,[["error",6]]]]],"p":[]},\
"http_trait":{"doc":"","t":[3,11,11,11,11,11,11,12,5,11,11,11,11],"n":["MyHandler","borrow","borrow_mut","call","default","from","into","invoke_count","main","poll_ready","try_from","try_into","type_id"],"q":["http_trait","","","","","","","","","","","",""],"d":["","","","","","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","","","","","",""],"i":[0,1,1,1,1,1,1,1,0,1,1,1,1],"f":[null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0],["request",6]]],[[],["myhandler",3]],[[]],[[]],null,[[],["result",4,[["error",6]]]],[[["",0],["context",3]],["poll",4,[["result",4]]]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]]],"p":[[3,"MyHandler"]]},\
"lambda_extension":{"doc":"This module includes utilities to create Lambda Runtime …","t":[6,16,3,13,3,13,16,3,13,3,3,3,4,3,3,3,4,13,13,13,13,13,13,13,13,16,8,3,13,3,3,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,10,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,11,11,11,11,11,11,11,11,11,12,11,11,11,11,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,11,11,11,12,12,12,12,11,11,11,11,11,11,12,10,11,11,11,11,11,12,12,0,5,11,11,5,12,12,12,11,11,11,11,11,11,11,11,2,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,11,11,11,11,11,11,11,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,3,11,11,12,12,5,11,11,5,11,11,12,11,11,11],"n":["Error","Error","Extension","Extension","ExtensionError","Function","Future","Identity","Invoke","InvokeEvent","LambdaEvent","LambdaLog","LambdaLogRecord","LogBuffering","LogPlatformReportMetrics","MakeIdentity","NextEvent","PlatformEnd","PlatformExtension","PlatformFault","PlatformLogsDropped","PlatformLogsSubscription","PlatformReport","PlatformRuntimeDone","PlatformStart","Response","Service","SharedService","Shutdown","ShutdownEvent","Tracing","billed_duration_ms","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","call","call","call","call","clone","clone","clone","clone","clone","clone","clone","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","deadline_ms","deadline_ms","default","default","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","duration_ms","eq","eq","eq","eq","extension_id","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","from","from","from","from","from","from","from","init_duration_ms","into","into","into","into","into","into","into","into","into","into","into","into","into","into","invoked_function_arn","is_invoke","make_service","make_service","max_bytes","max_items","max_memory_used_mb","memory_size_mb","ne","ne","ne","ne","new","new","next","poll_ready","poll_ready","poll_ready","poll_ready","poll_ready","poll_ready","record","request_id","requests","run","run","serialize","service_fn","shutdown_reason","time","timeout_ms","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","to_string","tower","tracing","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","type","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","value","with_events","with_events_processor","with_extension_name","with_log_buffering","with_log_port_number","with_log_types","with_logs_processor","0","0","0","dropped_bytes","dropped_records","events","metrics","name","name","reason","request_id","request_id","request_id","request_id","state","state","status","types","0","0","ErrorRequest","borrow","borrow_mut","error_message","error_type","exit_error","fmt","from","init_error","into","serialize","stack_trace","try_from","try_into","type_id"],"q":["lambda_extension","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","lambda_extension::LambdaLogRecord","","","","","","","","","","","","","","","","","","lambda_extension::NextEvent","","lambda_extension::requests","","","","","","","","","","","","","",""],"d":["Error type that extensions may result in","Errors produced by the service.","An Extension that runs event and log processors","Extension log records","Simple error that encapsulates human readable descriptions","Function log records","The future response value.","A no-op generic processor","Payload when the event happens in the INVOKE phase","Event received when there is a new Lambda invocation.","Wrapper with information about the next event that the …","Payload received from the Lambda Logs API See: …","Record in a LambdaLog entry","Log buffering configuration. Allows Lambda to buffer logs …","Platform report metrics","Service factory to generate no-op generic processors","Event that the extension receives in either the INVOKE or …","Platform stop record","Extension-specific record","Runtime or execution environment error record","Record generated when the log processor is falling behind","Log processor-specific record","Platform report record","Record marking the completion of an invocation","Platform start record","Responses given by the service.","An asynchronous function from a <code>Request</code> to a <code>Response</code>.","A <code>MakeService</code> that produces services by cloning an inner …","Payload when the event happens in the SHUTDOWN phase","Event received when a Lambda function shuts down.","Request tracing information","Billed duration in milliseconds","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Process the request and return the response asynchronously.","","","","","","","","","","","","","","","","","","The time that the function times out","The time that the function times out","","","","","","","","","","Duration in milliseconds","","","","","ID assigned to this extension by the Lambda Runtime","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Init duration in case of a cold start","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","The function’s Amazon Resource Name","Return whether the event is a <code>NextEvent::Invoke</code> event or …","","","The maximum size (in bytes) of the logs to buffer in …","The maximum number of events to buffer in memory. Default: …","Maximum memory used for the invoke in megabytes","Memory allocated in megabytes","","","","","Create a new base <code>Extension</code> with a no-op events processor","Create a new <code>Shared</code> from a service.","Next incoming event","Returns <code>Poll::Ready(Ok(()))</code> when the service is able to …","","","","","","Log record entry","The ID assigned to the Lambda request","Include several request builders to interact with the …","Execute the given events processor","Execute the given extension","","Returns a new <code>ServiceFn</code> with the given closure.","The reason why the function terminates It can be SPINDOWN, …","Time when the log was generated","The maximum time (in milliseconds) to buffer a batch. …","","","","","","","","","","The request tracing information","","","","","","","","","","","","","","","","","","","","","","","","","","","","","The type of tracing exposed to the extension","","","","","","","","","","","","","","","The span value","Create a new <code>Extension</code> with a list of given events. The …","Create a new <code>Extension</code> with a service that receives Lambda …","Create a new <code>Extension</code> with a given extension name","Create a new <code>Extension</code> with specific configuration to …","Create a new <code>Extension</code> with a different port number to …","Create a new <code>Extension</code> with a list of logs types to …","Create a new <code>Extension</code> with a service that receives Lambda …","","","","Total size of the dropped records","Number of records dropped","Events sent to the extension","Request metrics","Name of the extension","Name of the extension","Reason for dropping the logs","Request identifier","Request identifier","Request identifier","Request identifier","State of the extension","State of the extensions","Status of the invocation","Types of records sent to the extension","","","Payload to send error information to the Extensions API.","","","Human readable error description","The type of error to categorize","Create a new exit error request to send to the Extensions …","","Returns the argument unchanged.","Create a new init error request to send to the Extensions …","Calls <code>U::from(self)</code>.","","The error backtrace","","",""],"i":[0,1,0,2,0,2,1,0,3,0,0,0,0,0,0,0,0,2,2,2,2,2,2,2,2,1,0,0,3,0,0,4,5,6,7,8,9,10,11,12,3,13,2,4,14,15,5,6,7,8,9,10,11,12,3,13,2,4,14,15,1,8,9,15,7,8,9,13,2,4,15,7,8,9,13,2,4,15,11,12,5,14,10,11,12,3,13,2,4,4,7,13,2,4,6,7,7,10,11,12,3,13,2,4,14,15,5,6,7,8,9,10,11,12,3,13,2,4,14,15,4,5,6,7,8,9,10,11,12,3,13,2,4,14,15,11,3,9,15,14,14,4,4,7,13,2,4,5,15,6,1,8,9,9,15,15,13,11,0,0,5,14,0,12,13,14,7,8,9,13,2,4,15,7,0,11,5,6,7,8,9,10,11,12,3,13,2,4,14,15,5,6,7,8,9,10,11,12,3,13,2,4,14,15,10,5,6,7,8,9,10,11,12,3,13,2,4,14,15,10,5,5,5,5,5,5,5,16,17,18,19,19,20,21,20,22,19,23,24,21,25,20,22,25,22,26,27,0,28,28,28,28,0,28,28,0,28,28,28,28,28,28],"f":[null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]]],[[["",0]]],[[["",0]]],[[["",0]]],[[["",0]],["extensionerror",3]],[[["",0]],["identity",3]],[[["",0]],["makeidentity",3]],[[["",0]],["lambdalog",3]],[[["",0]],["lambdalogrecord",4]],[[["",0]],["logplatformreportmetrics",3]],[[["",0]],["shared",3]],[[["",0],["",0]]],[[["",0],["",0]]],[[["",0],["",0]]],[[["",0],["",0]]],[[["",0],["",0]]],[[["",0],["",0]]],[[["",0],["",0]]],null,null,[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],null,[[["",0],["extensionerror",3]],["bool",0]],[[["",0],["lambdalog",3]],["bool",0]],[[["",0],["lambdalogrecord",4]],["bool",0]],[[["",0],["logplatformreportmetrics",3]],["bool",0]],null,[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",4,[["error",3]]]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],null,[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],null,[[["",0]],["bool",0]],[[["",0]]],[[["",0]]],null,null,null,null,[[["",0],["extensionerror",3]],["bool",0]],[[["",0],["lambdalog",3]],["bool",0]],[[["",0],["lambdalogrecord",4]],["bool",0]],[[["",0],["logplatformreportmetrics",3]],["bool",0]],[[]],[[],["shared",3]],null,[[["",0],["context",3]],["poll",4,[["result",4]]]],[[["",0],["context",3]],["poll",4,[["result",4]]]],[[["",0],["context",3]],["poll",4,[["result",4]]]],[[["",0],["context",3]],["poll",4,[["result",4]]]],[[["",0],["context",3]],["poll",4,[["result",4]]]],[[["",0],["context",3]],["poll",4,[["result",4]]]],null,null,null,[[]],[[]],[[["",0]],["result",4]],[[],["servicefn",3]],null,null,null,[[["",0]]],[[["",0]]],[[["",0]]],[[["",0]]],[[["",0]]],[[["",0]]],[[["",0]]],[[["",0]],["string",3]],null,null,[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],null,[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],null,[[]],[[],["extension",3]],[[["str",0]]],[[["logbuffering",3]]],[[["u16",0]]],[[]],[[],["extension",3]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[["",0]],["",0]],[[["",0]],["",0]],null,null,[[["str",0],["str",0],["option",4,[["errorrequest",3]]]],["result",4,[["request",3,[["body",3]]],["error",6]]]],[[["",0],["formatter",3]],["result",6]],[[]],[[["str",0],["str",0],["option",4,[["errorrequest",3]]]],["result",4,[["request",3,[["body",3]]],["error",6]]]],[[]],[[["",0]],["result",4]],null,[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]]],"p":[[8,"Service"],[4,"LambdaLogRecord"],[4,"NextEvent"],[3,"LogPlatformReportMetrics"],[3,"Extension"],[3,"LambdaEvent"],[3,"ExtensionError"],[3,"Identity"],[3,"MakeIdentity"],[3,"Tracing"],[3,"InvokeEvent"],[3,"ShutdownEvent"],[3,"LambdaLog"],[3,"LogBuffering"],[3,"SharedService"],[13,"Function"],[13,"Extension"],[13,"PlatformFault"],[13,"PlatformLogsDropped"],[13,"PlatformExtension"],[13,"PlatformReport"],[13,"PlatformLogsSubscription"],[13,"PlatformStart"],[13,"PlatformEnd"],[13,"PlatformRuntimeDone"],[13,"Invoke"],[13,"Shutdown"],[3,"ErrorRequest"]]},\
"lambda_http":{"doc":"Enriches the <code>lambda</code> crate with <code>http</code> types targeting AWS ALB…","t":[13,4,3,13,6,16,16,8,6,2,3,16,8,13,11,11,11,11,11,11,11,11,11,11,10,12,11,11,11,11,12,11,11,11,11,11,11,12,11,11,0,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,2,12,11,11,11,11,11,10,11,12,11,11,2,11,11,11,11,11,11,10,11,11,0,12,5,11,11,5,11,11,11,11,11,11,2,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,12,13,4,8,13,11,11,11,11,11,11,10,10,10,10,10,10,11,10,11,11,11,11,10,10,10,10,12,12,13,13,13,4,13,11,11,11,11,11,11,11,5,5,11,11,11,11,11,12,12,12,12],"n":["Binary","Body","Context","Empty","Error","Error","Future","IntoResponse","Request","RequestExt","Response","Response","Service","Text","as_ref","body","body_mut","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","builder","call","client_context","clone","clone","clone_into","clone_into","deadline","default","default","default","deref","deserialize","deserialize","env_config","eq","eq","ext","extensions","extensions_mut","fmt","fmt","fmt","from","from","from","from","from","from","from","from","from","from","from_maybe_encoded","from_parts","headers","headers_mut","http","identity","into","into","into","into_body","into_parts","into_response","into_response","invoked_function_arn","is_end_stream","is_end_stream","lambda_runtime","map","ne","ne","new","poll_data","poll_data","poll_ready","poll_trailers","poll_trailers","request","request_id","run","serialize","serialize","service_fn","size_hint","size_hint","status","status_mut","to_owned","to_owned","tower","try_from","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id","version","version_mut","with_config","xray_trace_id","0","0","Json","PayloadError","RequestExt","WwwFormUrlEncoded","borrow","borrow_mut","fmt","fmt","from","into","lambda_context","path_parameters","payload","query_string_parameters","raw_http_path","request_context","source","stage_variables","to_string","try_from","try_into","type_id","with_lambda_context","with_path_parameters","with_query_string_parameters","with_raw_http_path","0","0","Alb","ApiGatewayV1","ApiGatewayV2","RequestContext","WebSocket","borrow","borrow_mut","clone","clone_into","deserialize","fmt","from","from_reader","from_str","into","to_owned","try_from","try_into","type_id","0","0","0","0"],"q":["lambda_http","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","lambda_http::Body","","lambda_http::ext","","","","","","","","","","","","","","","","","","","","","","","","","","lambda_http::ext::PayloadError","","lambda_http::request","","","","","","","","","","","","","","","","","","","lambda_http::request::RequestContext","","",""],"d":["A body containing binary data","Representation of http request and response bodies as …","The Lambda function execution context. The values in this …","An empty body","Error type that lambdas may result in","Errors produced by the service.","The future response value.","A conversion of self into a <code>Response&lt;Body&gt;</code> for various …","Type alias for <code>http::Request</code>s with a fixed <code>Body</code> type","","Represents an HTTP response","Responses given by the service.","An asynchronous function from a <code>Request</code> to a <code>Response</code>.","A body containing string data","","Returns a reference to the associated HTTP body.","Returns a mutable reference to the associated HTTP body.","","","","","","","Creates a new builder-style object to manufacture a …","Process the request and return the response asynchronously.","The client context object sent by the AWS mobile SDK. This …","","","","","The execution deadline for the current invocation in …","","","","","","","Lambda function configuration from the local environment …","","","Extension methods for <code>http::Request</code> types","Returns a reference to the associated extensions.","Returns a mutable reference to the associated extensions.","","","","Returns the argument unchanged.","Returns the argument unchanged.","","","","","","Returns the argument unchanged.","","","Decodes body, if needed.","Creates a new <code>Response</code> with the given head and body","Returns a reference to the associated header field map.","Returns a mutable reference to the associated header field …","","The Cognito identity that invoked the function. This field …","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Consumes the response, returning just the body.","Consumes the response returning the head and body parts.","Return a translation of <code>self</code> into a <code>Response&lt;Body&gt;</code>","","The ARN of the Lambda function being invoked.","","","","Consumes the response returning a new response with body …","","","Creates a new blank <code>Response</code> with the body","","","Returns <code>Poll::Ready(Ok(()))</code> when the service is able to …","","","ALB and API Gateway request adaptations","The AWS request ID generated by the Lambda service.","Starts the Lambda Rust runtime and begins polling for …","","","Returns a new <code>ServiceFn</code> with the given closure.","","","Returns the <code>StatusCode</code>.","Returns a mutable reference to the associated <code>StatusCode</code>.","","","","","","","","","","","","","","Returns a reference to the associated version.","Returns a mutable reference to the associated version.","Add environment details to the context by setting …","The X-Ray trace ID for the current invocation.","","","Returned when <code>application/json</code> bodies fail to deserialize …","Request payload deserialization errors","Extentions for <code>lambda_http::Request</code> structs that provide …","Returned when <code>application/x-www-form-urlencoded</code> bodies …","","","","","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","Return the Lambda function context associated with the …","Return pre-extracted path parameters, parameter provided …","Return the Result of a payload parsed into a serde …","Return pre-parsed http query string parameters, parameters …","Return the raw http path for a request without any stage …","Return request context data assocaited with the ALB or API …","","Return stage variables associated with the API gateway …","","","","","Configures instance with lambda context","Configures instance with path parameters under #[cfg(test)]…","Configures instance with query string parameters under #[…","Configures instance with the raw http path.","","","ALB request context","API Gateway proxy request context","API Gateway v2 request context","Event request context as an enumeration of request contexts","WebSocket request context","","","","","","","Returns the argument unchanged.","Deserializes a <code>Request</code> from a <code>Read</code> impl providing JSON …","Deserializes a <code>Request</code> from a string of JSON text.","Calls <code>U::from(self)</code>.","","","","","","","",""],"i":[1,0,0,1,0,2,2,0,0,0,0,2,0,1,1,3,3,4,3,1,4,3,1,3,2,4,4,1,4,1,4,4,3,1,1,4,1,4,4,1,0,3,3,4,3,1,4,3,1,1,1,1,1,1,1,1,1,3,3,3,0,4,4,3,1,3,3,5,3,4,3,1,0,3,4,1,3,3,1,2,3,1,0,4,0,4,1,0,3,1,3,3,4,1,0,4,4,3,1,4,3,1,4,3,1,3,3,4,4,6,7,8,0,0,8,8,8,8,8,8,8,9,9,9,9,9,9,8,9,8,8,8,8,9,9,9,9,10,11,12,12,12,0,12,12,12,12,12,12,12,12,0,0,12,12,12,12,12,13,14,15,16],"f":[null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[["",0]]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[],["builder",3]],[[["",0]]],null,[[["",0]],["context",3]],[[["",0]],["body",4]],[[["",0],["",0]]],[[["",0],["",0]]],null,[[],["context",3]],[[],["response",3]],[[],["body",4]],[[["",0]]],[[],["result",4,[["context",3]]]],[[],["result",4,[["body",4]]]],null,[[["",0],["context",3]],["bool",0]],[[["",0],["body",4]],["bool",0]],null,[[["",0]],["extensions",3]],[[["",0]],["extensions",3]],[[["",0],["formatter",3]],["result",4,[["error",3]]]],[[["",0],["formatter",3]],["result",4,[["error",3]]]],[[["",0],["formatter",3]],["result",4,[["error",3]]]],[[]],[[]],[[["cow",4]],["body",4]],[[["str",0]],["body",4]],[[],["body",4]],[[["vec",3,[["u8",0],["global",3]]]],["body",4]],[[],["body",4]],[[]],[[["string",3]],["body",4]],[[["cow",4,[["str",0]]]],["body",4]],[[["bool",0],["str",0]],["body",4]],[[["parts",3]],["response",3]],[[["",0]],["headermap",3]],[[["",0]],["headermap",3]],null,null,[[]],[[]],[[]],[[]],[[]],[[],["response",3,[["body",4]]]],[[],["response",3,[["body",4]]]],null,[[["",0]],["bool",0]],[[["",0]],["bool",0]],null,[[],["response",3]],[[["",0],["context",3]],["bool",0]],[[["",0],["body",4]],["bool",0]],[[],["response",3]],[[["pin",3,[["response",3]]],["context",3]],["poll",4,[["option",4,[["result",4]]]]]],[[["pin",3,[["body",4]]],["context",3]],["poll",4,[["option",4,[["result",4]]]]]],[[["",0],["context",3]],["poll",4,[["result",4]]]],[[["pin",3,[["response",3]]],["context",3]],["poll",4,[["result",4,[["option",4,[["headermap",3,[["headervalue",3]]]]]]]]]],[[["pin",3,[["body",4]]],["context",3]],["poll",4,[["result",4,[["option",4,[["headermap",3,[["headervalue",3]]]]]]]]]],null,null,[[]],[[["",0]],["result",4]],[[["",0]],["result",4]],[[],["servicefn",3]],[[["",0]],["sizehint",3]],[[["",0]],["sizehint",3]],[[["",0]],["statuscode",3]],[[["",0]],["statuscode",3]],[[["",0]]],[[["",0]]],null,[[],["result",4]],[[["headermap",3,[["headervalue",3]]]],["result",4,[["context",3]]]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["version",3]],[[["",0]],["version",3]],[[["config",3]],["context",3]],null,null,null,null,null,null,null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[]],[[]],[[["",0]],["context",3]],[[["",0]],["querymap",3]],[[["",0]],["result",4,[["option",4],["payloaderror",4]]]],[[["",0]],["querymap",3]],[[["",0]],["string",3]],[[["",0]],["requestcontext",4]],[[["",0]],["option",4,[["error",8]]]],[[["",0]],["querymap",3]],[[["",0]],["string",3]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],[[["context",3]]],[[]],[[]],[[["str",0]]],null,null,null,null,null,null,null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["requestcontext",4]],[[["",0],["",0]]],[[],["result",4]],[[["",0],["formatter",3]],["result",6]],[[]],[[],["result",4,[["request",6],["jsonerror",3]]]],[[["str",0]],["result",4,[["request",6],["jsonerror",3]]]],[[]],[[["",0]]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],null,null,null,null],"p":[[4,"Body"],[8,"Service"],[3,"Response"],[3,"Context"],[8,"IntoResponse"],[13,"Text"],[13,"Binary"],[4,"PayloadError"],[8,"RequestExt"],[13,"Json"],[13,"WwwFormUrlEncoded"],[4,"RequestContext"],[13,"ApiGatewayV1"],[13,"ApiGatewayV2"],[13,"Alb"],[13,"WebSocket"]]},\
"lambda_runtime":{"doc":"The mechanism available for defining a Lambda function is …","t":[3,3,6,16,16,3,16,8,11,11,11,11,11,11,10,12,11,11,11,11,11,11,12,12,11,11,11,11,12,11,11,11,11,11,11,11,11,11,12,5,12,11,11,11,11,12,12,12,12,11,11,11,12,10,12,5,11,11,5,11,11,11,2,11,11,11,11,11,11,11,11,11,11,12,11,12],"n":["Config","Context","Error","Error","Future","LambdaEvent","Response","Service","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","call","client_context","clone","clone","clone","clone_into","clone_into","clone_into","context","deadline","default","default","deserialize","deserialize","env_config","eq","eq","fmt","fmt","fmt","from","from","from","from_env","function_name","handler_fn","identity","into","into","into","into_parts","invoked_function_arn","log_group","log_stream","memory","ne","ne","new","payload","poll_ready","request_id","run","serialize","serialize","service_fn","to_owned","to_owned","to_owned","tower","try_from","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id","version","with_config","xray_trace_id"],"q":["lambda_runtime","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"d":["Configuration derived from environment variables.","The Lambda function execution context. The values in this …","Error type that lambdas may result in","Errors produced by the service.","The future response value.","Incoming Lambda request containing the event payload and …","Responses given by the service.","An asynchronous function from a <code>Request</code> to a <code>Response</code>.","","","","","","","Process the request and return the response asynchronously.","The client context object sent by the AWS mobile SDK. This …","","","","","","","Invocation context.","The execution deadline for the current invocation in …","","","","","Lambda function configuration from the local environment …","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Attempts to read configuration from environment variables.","The name of the function.","Return a new <code>ServiceFn</code> with a closure that takes an event …","The Cognito identity that invoked the function. This field …","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Split the Lambda event into its payload and context.","The ARN of the Lambda function being invoked.","The name of the Amazon CloudWatch Logs group for the …","The name of the Amazon CloudWatch Logs stream for the …","The amount of memory available to the function in MB.","","","Creates a new Lambda request","Event payload.","Returns <code>Poll::Ready(Ok(()))</code> when the service is able to …","The AWS request ID generated by the Lambda service.","Starts the Lambda Rust runtime and begins polling for …","","","Returns a new <code>ServiceFn</code> with the given closure.","","","","","","","","","","","","","","","The version of the function being executed.","Add environment details to the context by setting …","The X-Ray trace ID for the current invocation."],"i":[0,0,0,1,1,0,1,0,2,3,4,2,3,4,1,2,2,3,4,2,3,4,3,2,2,4,2,4,2,2,4,2,3,4,2,3,4,4,4,0,2,2,3,4,3,2,4,4,4,2,4,3,3,1,2,0,2,4,0,2,3,4,0,2,2,3,4,2,3,4,2,3,4,4,2,2],"f":[null,null,null,null,null,null,null,null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]]],null,[[["",0]],["context",3]],[[["",0]],["lambdaevent",3]],[[["",0]],["config",3]],[[["",0],["",0]]],[[["",0],["",0]]],[[["",0],["",0]]],null,null,[[],["context",3]],[[],["config",3]],[[],["result",4]],[[],["result",4]],null,[[["",0],["context",3]],["bool",0]],[[["",0],["config",3]],["bool",0]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[]],[[]],[[]],[[],["result",4,[["error",6]]]],null,[[],["servicefn",3]],null,[[]],[[]],[[]],[[]],null,null,null,null,[[["",0],["context",3]],["bool",0]],[[["",0],["config",3]],["bool",0]],[[["context",3]]],null,[[["",0],["context",3]],["poll",4,[["result",4]]]],null,[[]],[[["",0]],["result",4]],[[["",0]],["result",4]],[[],["servicefn",3]],[[["",0]]],[[["",0]]],[[["",0]]],null,[[["headermap",3]],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],null,[[["config",3]]],null],"p":[[8,"Service"],[3,"Context"],[3,"LambdaEvent"],[3,"Config"]]},\
"lambda_runtime_api_client":{"doc":"This crate includes a base HTTP client to interact with …","t":[3,3,6,12,11,11,11,11,11,5,11,11,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11],"n":["Client","ClientBuilder","Error","base","borrow","borrow","borrow_mut","borrow_mut","build","build_request","builder","call","client","fmt","from","from","into","into","try_from","try_from","try_into","try_into","type_id","type_id","with","with_connector","with_endpoint"],"q":["lambda_runtime_api_client","","","","","","","","","","","","","","","","","","","","","","","","","",""],"d":["API client to interact with the AWS Lambda Runtime API.","Builder implementation to construct any Runtime API …","Error type that lambdas may result in","The runtime API URI","","","","","Create the new client to interact with the Runtime API.","Create a request builder. This builder uses …","Create a builder struct to configure the client.","Send a given request to the Runtime API. Use the client’…","The client that manages the API connections","","Returns the argument unchanged.","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","","Create a new client with a given base URI and HTTP …","Create a new builder with a given HTTP connector.","Create a new builder with a given base URI. Inherits all …"],"i":[0,0,0,1,2,1,2,1,2,0,1,1,1,1,2,1,2,1,2,1,2,1,2,1,1,2,2],"f":[null,null,null,null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[],["result",4,[["client",3],["error",6]]]],[[],["builder",3]],[[],["clientbuilder",3,[["httpconnector",3]]]],[[["",0],["request",3,[["body",3]]]]],null,[[["",0],["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["uri",3]]],[[],["clientbuilder",3]],[[["uri",3]]]],"p":[[3,"Client"],[3,"ClientBuilder"]]},\
"logs_trait":{"doc":"","t":[6,3,11,11,11,11,11,12,11,11,11,5,11,11,11,11,11,11],"n":["MyLogsFuture","MyLogsProcessor","borrow","borrow_mut","call","clone","clone_into","counter","default","from","into","main","new","poll_ready","to_owned","try_from","try_into","type_id"],"q":["logs_trait","","","","","","","","","","","","","","","","",""],"d":["","Custom log processor that increments a counter for each …","","","","","","","","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","","","","","","",""],"i":[0,0,1,1,1,1,1,1,1,1,1,0,1,1,1,1,1,1],"f":[null,null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0],["vec",3,[["lambdalog",3]]]]],[[["",0]],["mylogsprocessor",3]],[[["",0],["",0]]],null,[[],["mylogsprocessor",3]],[[]],[[]],[[],["result",4,[["error",6]]]],[[]],[[["",0],["context",3]],["poll",4,[["result",4]]]],[[["",0]]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]]],"p":[[3,"MyLogsProcessor"]]},\
"runtime_fn":{"doc":"","t":[3,3,11,11,11,11,12,11,11,11,11,11,5,11,11,5,12,11,11,11,11,11,11,11],"n":["Request","Response","borrow","borrow","borrow_mut","borrow_mut","command","deserialize","fmt","fmt","from","from","handler","into","into","main","message","serialize","try_from","try_from","try_into","try_into","type_id","type_id"],"q":["runtime_fn","","","","","","","","","","","","","","","","","","","","","","",""],"d":["","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","","","",""],"i":[0,0,1,2,1,2,1,1,1,2,1,2,0,1,2,0,2,2,1,2,1,2,1,2],"f":[null,null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],null,[[],["result",4]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[]],[[]],[[["lambdaevent",3,[["request",3]]]]],[[]],[[]],[[],["result",4,[["error",6]]]],null,[[["",0]],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]]],"p":[[3,"Request"],[3,"Response"]]},\
"runtime_trait":{"doc":"","t":[3,3,3,11,11,11,11,11,11,11,12,11,11,11,11,11,11,11,11,11,11,12,5,12,11,11,11,11,11,11,11,11,11,11,11],"n":["MyHandler","Request","Response","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","call","command","default","deserialize","fmt","fmt","from","from","from","into","into","into","invoke_count","main","message","poll_ready","serialize","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id"],"q":["runtime_trait","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"d":["","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","","","","","","","","",""],"i":[0,0,0,1,2,3,1,2,3,3,1,3,1,1,2,1,2,3,1,2,3,3,0,2,3,2,1,2,3,1,2,3,1,2,3],"f":[null,null,null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0],["lambdaevent",3,[["request",3]]]]],null,[[],["myhandler",3]],[[],["result",4]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[]],[[]],null,[[],["result",4,[["error",6]]]],null,[[["",0],["context",3]],["poll",4,[["result",4]]]],[[["",0]],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]]],"p":[[3,"Request"],[3,"Response"],[3,"MyHandler"]]}\
}');
if (window.initSearch) {window.initSearch(searchIndex)};