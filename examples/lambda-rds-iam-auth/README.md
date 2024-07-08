# AWS Lambda Function that uses RDS's IAM Authnetication 
This example shows how to build and deploy Rust Lambda Function and an RDS instance using AWS CDK and 

Build & Deploy
1. `npm install`
1. `npx cdk deploy`
1. Using the dev instance or using a local Postgres client: connect into the RDS instance as root and create the required Users with permissions `CREATE USER lambda; GRANT rds_iam TO lambda;`
1. Go to the Lambda Function in the AWS console and invoke the lambda function