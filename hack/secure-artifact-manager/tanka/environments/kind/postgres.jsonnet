{
  postgres:: [
    {
      apiVersion: 'apps/v1',
      kind: 'Deployment',
      metadata: {
        name: 'postgres',
        labels: { app: 'postgres' },
      },
      spec: {
        replicas: 1,
        selector: { matchLabels: { app: 'postgres' } },
        template: {
          metadata: { labels: { app: 'postgres' } },
          spec: {
            containers: [{
              name: 'postgres',
              image: 'postgres:15',
              ports: [{ containerPort: 5432 }],
              env: [
                { name: 'POSTGRES_USER', value: 'postgres' },
                { name: 'POSTGRES_PASSWORD', value: 'postgres' },
                { name: 'POSTGRES_DB', value: 'artifacts' },
              ],
              volumeMounts: [{
                name: 'pgdata',
                mountPath: '/var/lib/postgresql/data',
              }],
            }],
            volumes: [{
              name: 'pgdata',
              emptyDir: {},  // Replace with pvc if needed
            }],
          },
        },
      },
    },

    {
      apiVersion: 'v1',
      kind: 'Service',
      metadata: {
        name: 'postgres',
      },
      spec: {
        selector: { app: 'postgres' },
        ports: [{
          protocol: 'TCP',
          port: 5432,
          targetPort: 5432,
        }],
      },
    },
  ],
}
