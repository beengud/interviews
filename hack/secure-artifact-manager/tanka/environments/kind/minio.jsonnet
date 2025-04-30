{
  minio:: [
    {
      apiVersion: 'apps/v1',
      kind: 'Deployment',
      metadata: {
        name: 'minio',
        labels: { app: 'minio' },
      },
      spec: {
        replicas: 1,
        selector: { matchLabels: { app: 'minio' } },
        template: {
          metadata: { labels: { app: 'minio' } },
          spec: {
            containers: [{
              name: 'minio',
              image: 'minio/minio:latest',
              args: ['server', '/data'],
              ports: [{ containerPort: 9000 }],
              env: [
                { name: 'MINIO_ACCESS_KEY', value: 'minioadmin' },
                { name: 'MINIO_SECRET_KEY', value: 'minioadmin' },
              ],
              volumeMounts: [{
                name: 'minio-data',
                mountPath: '/data',
              }],
            }],
            volumes: [{
              name: 'minio-data',
              emptyDir: {},  // Replace with pvc if desired
            }],
          },
        },
      },
    },

    {
      apiVersion: 'v1',
      kind: 'Service',
      metadata: {
        name: 'minio',
      },
      spec: {
        selector: { app: 'minio' },
        ports: [{
          protocol: 'TCP',
          port: 9000,
          targetPort: 9000,
        }],
      },
    },
  ],
}
