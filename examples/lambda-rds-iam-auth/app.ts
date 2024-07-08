import * as cdk from 'aws-cdk-lib';
import * as rds from 'aws-cdk-lib/aws-rds';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import { RustFunction } from '@cdklabs/aws-lambda-rust'

class LambdaRDSStack extends cdk.Stack {
    constructor(scope: cdk.App, id: string, props?: cdk.StackProps) {
        super(scope, id, props);

        // Create a VPC
        const vpc = new ec2.Vpc(this, 'VPC');

        // Admin DB user
        const DB_ADMIN_USERNAME = 'root';
        const DB_USERNAME = 'lambda';

        // Lambda DB user
        const DB_NAME = 'foo';

        // Create an RDS instance
        const db = new rds.DatabaseInstance(this, 'Postgres', {
            engine: rds.DatabaseInstanceEngine.POSTGRES,
            vpc,
            vpcSubnets: vpc.selectSubnets({ subnetType: ec2.SubnetType.PUBLIC }),
            credentials: rds.Credentials.fromGeneratedSecret(DB_ADMIN_USERNAME),
            iamAuthentication: true,
            publiclyAccessible: true,
            databaseName: DB_NAME,
            deleteAutomatedBackups: true,
            removalPolicy: cdk.RemovalPolicy.DESTROY
        })

        db.connections.allowFromAnyIpv4(ec2.Port.allTcp())

        // RDS SSL Cert Lambda Layer alternative to loading the certificates at compile time
        /*
        const certLayer = new lambda.LayerVersion(this, 'CertLayer', {
            description: 'SSL Certificate Layer',
            code: lambda.Code.fromAsset('certs'),
            compatibleArchitectures: [lambda.Architecture.X86_64, lambda.Architecture.ARM_64]
        });
        */

        const lambdaSG = new ec2.SecurityGroup(this, 'LambdaSG', {
            securityGroupName: 'LambdaSG',
            allowAllOutbound: true,
            vpc: vpc,
        })
        // create a rust lambda function 
        const rustLambdaFunction = new RustFunction(this, "lambda", {
            entry: 'lambda',
            vpc: vpc,
            securityGroups: [lambdaSG],
            environment: {
                DB_HOSTNAME: db.dbInstanceEndpointAddress,
                DB_PORT: db.dbInstanceEndpointPort,
                DB_NAME: DB_NAME,
                DB_USERNAME: DB_USERNAME,
            },
            bundling: {
                forceDockerBundling: true,
            },
            runtime: lambda.Runtime.PROVIDED_AL2023,
            timeout: cdk.Duration.seconds(60),
        });

        // MySQL 
        /*
        CREATE USER 'lambda' IDENTIFIED WITH AWSAuthenticationPlugin AS 'RDS'; 
        GRANT ALL PRIVILEGES ON foo.* TO 'lambda';
        ALTER USER 'lambda' REQUIRE SSL;
        */

        // Postgres
        /*
        CREATE USER db_userx; 
        GRANT rds_iam TO db_userx;
        */
        db.grantConnect(rustLambdaFunction, DB_USERNAME);
        db.connections.allowDefaultPortFrom(rustLambdaFunction);

        /*
        Dev Instance for initialising the datbase with the above commands
        */
        const devInstance = new ec2.Instance(this, 'dev', {
            vpc,
            vpcSubnets: vpc.selectSubnets({ subnetType: ec2.SubnetType.PUBLIC }),
            machineImage: ec2.MachineImage.latestAmazonLinux2023(),
            instanceType: ec2.InstanceType.of(ec2.InstanceClass.T3, ec2.InstanceSize.MEDIUM)
        })
        db.grantConnect(devInstance, DB_ADMIN_USERNAME);
        db.grantConnect(devInstance, DB_USERNAME);
        db.connections.allowDefaultPortFrom(devInstance);

        // Output the Lambda function ARN
        new cdk.CfnOutput(this, 'LambdaFunctionConsole', {
            value: `https://${this.region}.console.aws.amazon.com/lambda/home?region=${this.region}#/functions/${rustLambdaFunction.functionName}?tab=testing`
        });
    }
}

const app = new cdk.App();
new LambdaRDSStack(app, 'LambdaRDSStack');
