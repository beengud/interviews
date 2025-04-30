local backend = import 'backend.jsonnet';
local frontend = import 'frontend.jsonnet';
local minio = import 'minio.jsonnet';
local postgres = import 'postgres.jsonnet';
{

  backend: backend.backend,
  frontend: frontend.frontend,
  postgres: postgres.postgres,
  minio: minio.minio,

}
