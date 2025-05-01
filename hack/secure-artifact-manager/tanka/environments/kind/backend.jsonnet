// backend.jsonnet
local k = import 'k.libsonnet';
local deployment = k.apps.v1.deployment;
local service = k.core.v1.service;

{

  backend:: deployment.new(
    name='backend',
    replicas=1,
    containers=[{
      name: 'backend',
      image: 'localhost:5005/backend:latest',
      ports: [{ containerPort: 8080 }],
      env: [
        { name: 'DATABASE_URL', value: 'postgres://postgres:postgres@postgres:5432/artifacts' },
        { name: 'AWS_ACCESS_KEY_ID', value: 'minioadmin' },
        { name: 'AWS_SECRET_ACCESS_KEY', value: 'minioadmin' },
        { name: 'AWS_REGION', value: 'us-east-1' },
      ],
    }],
    podLabels={
      app: 'backend',
    },
  ),
}
