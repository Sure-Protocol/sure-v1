package main

import (
	"encoding/base64"
	"errors"
	"fmt"
	"strings"

	"github.com/pulumi/pulumi-aws/sdk/v5/go/aws/acm"
	"github.com/pulumi/pulumi-aws/sdk/v5/go/aws/ec2"
	"github.com/pulumi/pulumi-aws/sdk/v5/go/aws/ecr"
	"github.com/pulumi/pulumi-aws/sdk/v5/go/aws/ecs"
	"github.com/pulumi/pulumi-aws/sdk/v5/go/aws/iam"
	"github.com/pulumi/pulumi-aws/sdk/v5/go/aws/lb"
	"github.com/pulumi/pulumi-docker/sdk/v3/go/docker"
	"github.com/pulumi/pulumi/sdk/v3/go/pulumi"
)

func main() {
	pulumi.Run(func(ctx *pulumi.Context) error {

		// set up load balancer
		// get virtual private cloud
		t := true
		vpc, err := ec2.LookupVpc(ctx, &ec2.LookupVpcArgs{Default: &t})
		if err != nil {
			return err
		}
		// ids of subnets in the vpc
		// subnet, err := ec2.GetSubnetIds(ctx, &ec2.GetSubnetIdsArgs{VpcId: vpc.Id})
		// if err != nil {
		// 	return err
		// }

		subnetFilter := ec2.GetSubnetsFilter{Name: "vpc-id", Values: []string{vpc.Id}}
		subnets, err := ec2.GetSubnets(ctx, &ec2.GetSubnetsArgs{
			Filters: []ec2.GetSubnetsFilter{subnetFilter},
		})
		if err != nil {
			return err
		}

		// security group to allow traffic into the network
		// allow for unlimted egress
		// limited ingress to 80 on http
		webSg, err := ec2.NewSecurityGroup(ctx, "web-sg", &ec2.SecurityGroupArgs{
			VpcId: pulumi.String(vpc.Id),
			Egress: ec2.SecurityGroupEgressArray{
				ec2.SecurityGroupEgressArgs{
					Protocol:   pulumi.String("-1"),
					FromPort:   pulumi.Int(0),
					ToPort:     pulumi.Int(0),
					CidrBlocks: pulumi.StringArray{pulumi.String("0.0.0.0/0")},
				},
			},
			Ingress: ec2.SecurityGroupIngressArray{
				ec2.SecurityGroupIngressArgs{
					Protocol:   pulumi.String("tcp"),
					FromPort:   pulumi.Int(80),
					ToPort:     pulumi.Int(80),
					CidrBlocks: pulumi.StringArray{pulumi.String("0.0.0.0/0")},
				},
				ec2.SecurityGroupIngressArgs{
					Protocol:   pulumi.String("tcp"),
					FromPort:   pulumi.Int(443),
					ToPort:     pulumi.Int(443),
					CidrBlocks: pulumi.StringArray{pulumi.String("0.0.0.0/0")},
				},
			},
		})
		if err != nil {
			return err
		}

		// create a load balancer that receives data on port 80,
		// must be validated by websg
		loadBalancer, err := lb.NewLoadBalancer(ctx, "sure-lb", &lb.LoadBalancerArgs{
			Subnets:        pulumi.ToStringArray(subnets.Ids),
			SecurityGroups: pulumi.StringArray{webSg.ID().ToStringOutput()},
		})
		if err != nil {
			return err
		}

		// target group hit by listener
		targetGroup, err := lb.NewTargetGroup(ctx, "web-tg", &lb.TargetGroupArgs{
			Port:       pulumi.Int(80),
			Protocol:   pulumi.String("HTTP"),
			TargetType: pulumi.String("ip"),
			VpcId:      pulumi.String(vpc.Id),
		})
		if err != nil {
			return err
		}

		// target group hit by listener
		_, err = lb.NewTargetGroup(ctx, "web-tg-https", &lb.TargetGroupArgs{
			Port:       pulumi.Int(443),
			Protocol:   pulumi.String("HTTP"),
			TargetType: pulumi.String("ip"),
			VpcId:      pulumi.String(vpc.Id),
		})
		if err != nil {
			return err
		}

		cert, err := acm.NewCertificate(ctx, "oracle-cert", &acm.CertificateArgs{
			DomainName:       pulumi.String("sure.claims"),
			ValidationMethod: pulumi.String("DNS"),
		})
		if err != nil {
			return err
		}

		//create listener on port 80 and forward requests to target group
		_, err = lb.NewListener(ctx, "http-listener", &lb.ListenerArgs{
			LoadBalancerArn: loadBalancer.Arn,
			Port:            pulumi.Int(80),
			DefaultActions: lb.ListenerDefaultActionArray{
				lb.ListenerDefaultActionArgs{
					Type: pulumi.String("redirect"),
					Redirect: lb.ListenerDefaultActionRedirectArgs{
						Protocol:   pulumi.String("HTTPS"),
						Port:       pulumi.String("443"),
						StatusCode: pulumi.String("HTTP_301"),
					},
				},
			},
		})
		if err != nil {
			return err
		}

		httpsListener, err := lb.NewListener(ctx, "https-listener", &lb.ListenerArgs{
			LoadBalancerArn: loadBalancer.Arn,
			Port:            pulumi.Int(443),
			Protocol:        pulumi.String("HTTPS"),
			SslPolicy:       pulumi.String("ELBSecurityPolicy-2016-08"),
			CertificateArn:  cert.Arn,
			DefaultActions: lb.ListenerDefaultActionArray{
				lb.ListenerDefaultActionArgs{
					Type:           pulumi.String("forward"),
					TargetGroupArn: targetGroup.Arn,
				},
			},
		})
		if err != nil {
			return err
		}

		// create task execution role for
		taskExecRole, err := iam.NewRole(ctx, "task-exec-role", &iam.RoleArgs{
			AssumeRolePolicy: pulumi.String(`{
				"Version": "2008-10-17",
				"Statement": [{
					"Sid": "",
					"Effect": "Allow",
					"Principal": {
						"Service": "ecs-tasks.amazonaws.com"
					},
					"Action": "sts:AssumeRole"
				}]
				}`),
		})
		if err != nil {
			return err
		}

		// create new policy for role
		_, err = iam.NewRolePolicyAttachment(ctx, "task-exec-policy", &iam.RolePolicyAttachmentArgs{
			Role:      taskExecRole.Name,
			PolicyArn: pulumi.String("arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"),
		})
		if err != nil {
			return err
		}

		// create a new container cluster
		cluster, err := ecs.NewCluster(ctx, "sure-cluster", nil)
		if err != nil {
			return err
		}
		ctx.Export("sure-ecs", cluster.ID())

		// create a new ecr registry
		repo, err := ecr.NewRepository(ctx, "sure-oracle", nil)
		if err != nil {
			return err
		}

		// generate registry credentials
		registryInfo := repo.RegistryId.ApplyT(func(id string) (docker.ImageRegistry, error) {
			creds, err := ecr.GetCredentials(ctx, &ecr.GetCredentialsArgs{RegistryId: id})
			if err != nil {
				return docker.ImageRegistry{}, err
			}
			decoded, err := base64.StdEncoding.DecodeString(creds.AuthorizationToken)
			if err != nil {
				return docker.ImageRegistry{}, err
			}
			parts := strings.Split(string(decoded), ":")
			if len(parts) != 2 {
				return docker.ImageRegistry{}, errors.New("Invalid credentials")
			}
			return docker.ImageRegistry{
				Server:   creds.ProxyEndpoint,
				Username: parts[0],
				Password: parts[1],
			}, nil
		}).(docker.ImageRegistryOutput)

		// build sure oracle image
		image, err := docker.NewImage(ctx, "sure-oracle", &docker.ImageArgs{
			Build:     &docker.DockerBuildArgs{Context: pulumi.String("./../"), Dockerfile: pulumi.String("./../dockerfile.oracle")},
			ImageName: repo.RepositoryUrl,
			Registry:  registryInfo,
		})
		if err != nil {
			return err
		}

		ctx.Export("baseImageName", image.BaseImageName)
		ctx.Export("fullImageName", image.ImageName)

		// create the task definition for the container
		containerDefinition := image.ImageName.ApplyT(func(name string) (string, error) {
			formatstr := `[{
				"name":"sure-oracle",
				"image": %q,
				"portMappings": [{
					"containerPort": 80,
					"hostPort": 80,
					"protocol": "tcp"
				}]
			}]
			`
			return fmt.Sprintf(formatstr, name), nil
		}).(pulumi.StringOutput)

		// load the docker container
		appTask, err := ecs.NewTaskDefinition(ctx, "sure-task", &ecs.TaskDefinitionArgs{
			Family:                  pulumi.String("fargate-task-definition"),
			Cpu:                     pulumi.String("256"),
			Memory:                  pulumi.String("512"),
			NetworkMode:             pulumi.String("awsvpc"),
			RequiresCompatibilities: pulumi.StringArray{pulumi.String("FARGATE")},
			ExecutionRoleArn:        taskExecRole.Arn,
			ContainerDefinitions:    containerDefinition,
		})
		if err != nil {
			return err
		}

		ctx.Export("cluster", cluster.Arn)
		// setup the service
		_, err = ecs.NewService(ctx, "sure-svc", &ecs.ServiceArgs{
			Cluster:        cluster.ID(),
			DesiredCount:   pulumi.Int(5),
			LaunchType:     pulumi.String("FARGATE"),
			TaskDefinition: appTask.Arn,
			NetworkConfiguration: ecs.ServiceNetworkConfigurationArgs{
				AssignPublicIp: pulumi.Bool(true),
				Subnets:        pulumi.ToStringArray(subnets.Ids),
				SecurityGroups: pulumi.StringArray{webSg.ID().ToStringOutput()},
			},
			LoadBalancers: ecs.ServiceLoadBalancerArray{
				ecs.ServiceLoadBalancerArgs{
					TargetGroupArn: targetGroup.Arn,
					ContainerName:  pulumi.String("sure-oracle"),
					ContainerPort:  pulumi.Int(80),
				},
			},
		}, pulumi.DependsOn([]pulumi.Resource{cluster, httpsListener}))
		if err != nil {
			return err
		}
		ctx.Export("url", loadBalancer.DnsName)

		return nil
	})
}
