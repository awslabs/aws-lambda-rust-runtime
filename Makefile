INTEG_STACK_NAME ?= rust-lambda-integration-tests
INTEG_FUNCTIONS_BUILD := runtime-fn runtime-trait
INTEG_FUNCTIONS_INVOKE := RuntimeFn RuntimeFnAl2 RuntimeTrait RuntimeTraitAl2 Python PythonAl2
INTEG_EXTENSIONS := extension-fn extension-trait
# Using musl to run extensions on both AL1 and AL2
INTEG_ARCH := x86_64-unknown-linux-musl

integration-tests:
# Build Integration functions
	cross build --release --target $(INTEG_ARCH) -p lambda_integration_tests
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

build-integration-function-%:
	mkdir -p ./build/$*
	cp -v ./target/$(INTEG_ARCH)/release/$* ./build/$*/bootstrap

build-integration-extension-%:
	mkdir -p ./build/$*/extensions
	cp -v ./target/$(INTEG_ARCH)/release/$* ./build/$*/extensions/$*

invoke-integration-function-%:
	aws lambda invoke --function-name $$(aws cloudformation describe-stacks --stack-name $(INTEG_STACK_NAME) \
		--query 'Stacks[0].Outputs[?OutputKey==`$*`].OutputValue' \
		--output text) --payload '{"command": "hello"}' --cli-binary-format raw-in-base64-out /dev/stdout