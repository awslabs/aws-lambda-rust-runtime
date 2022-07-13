INTEG_STACK_NAME ?= rust-lambda-integration-tests
INTEG_FUNCTIONS_BUILD := runtime-fn runtime-trait http-fn http-trait
INTEG_FUNCTIONS_INVOKE := RuntimeFn RuntimeFnAl2 RuntimeTrait RuntimeTraitAl2 Python PythonAl2
INTEG_API_INVOKE := RestApiUrl HttpApiUrl
INTEG_EXTENSIONS := extension-fn extension-trait logs-trait
# Using musl to run extensions on both AL1 and AL2
INTEG_ARCH := x86_64-unknown-linux-musl

define uppercase
$(shell sed -r 's/(^|-)(\w)/\U\2/g' <<< $(1))
endef

pr-check:
	cargo +1.54.0 check --all
	cargo +stable fmt --all -- --check
	cargo +stable clippy
	cargo +1.54.0 test
	cargo +stable test

integration-tests:
# Build Integration functions
	cargo zigbuild --release --target $(INTEG_ARCH) -p lambda_integration_tests
	rm -rf ./build
	mkdir -p ./build
	${MAKE} ${MAKEOPTS} $(foreach function,${INTEG_FUNCTIONS_BUILD}, build-integration-function-${function})
	${MAKE} ${MAKEOPTS} $(foreach extension,${INTEG_EXTENSIONS}, build-integration-extension-${extension})
# Deploy to AWS
	sam deploy \
		--template lambda-integration-tests/template.yaml \
		--stack-name ${INTEG_STACK_NAME} \
		--capabilities CAPABILITY_IAM \
		--resolve-s3 \
		--no-fail-on-empty-changeset
# Invoke functions
	${MAKE} ${MAKEOPTS} $(foreach function,${INTEG_FUNCTIONS_INVOKE}, invoke-integration-function-${function})
	${MAKE} ${MAKEOPTS} $(foreach api,${INTEG_API_INVOKE}, invoke-integration-api-${api})

build-integration-function-%:
	mkdir -p ./build/$*
	cp -v ./target/$(INTEG_ARCH)/release/$* ./build/$*/bootstrap

build-integration-extension-%:
	mkdir -p ./build/$*/extensions
	cp -v ./target/$(INTEG_ARCH)/release/$* ./build/$*/extensions/$(call uppercase,$*)

invoke-integration-function-%:
	aws lambda invoke --function-name $$(aws cloudformation describe-stacks --stack-name $(INTEG_STACK_NAME) \
		--query 'Stacks[0].Outputs[?OutputKey==`$*`].OutputValue' \
		--output text) --payload '{"command": "hello"}' --cli-binary-format raw-in-base64-out /dev/stdout

invoke-integration-api-%:
	$(eval API_URL := $(shell aws cloudformation describe-stacks --stack-name $(INTEG_STACK_NAME) \
		--query 'Stacks[0].Outputs[?OutputKey==`$*`].OutputValue' \
		--output text))
	curl $(API_URL)/get
	curl $(API_URL)/trait/get
	curl $(API_URL)/al2/get
	curl $(API_URL)/al2-trait/get
	curl -X POST -d '{"command": "hello"}' $(API_URL)/post
	curl -X POST -d '{"command": "hello"}' $(API_URL)/trait/post
	curl -X POST -d '{"command": "hello"}' $(API_URL)/al2/post
	curl -X POST -d '{"command": "hello"}' $(API_URL)/al2-trait/post
