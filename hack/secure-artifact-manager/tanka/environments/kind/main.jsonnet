local alloy = import 'alloy.jsonnet';
local backend = import 'backend.jsonnet';
local frontend = import 'frontend.jsonnet';
local keda = import 'keda.jsonnet';
local kps = import 'kps.jsonnet';
local loki = import 'loki.jsonnet';
local minio = import 'minio.jsonnet';
local postgres = import 'postgres.jsonnet';
{

  backend: backend.backend,
  frontend: frontend.frontend,
  postgres: postgres.postgres,
  minio: minio.minio,

  kps: kps.kps,
  alloy: alloy.alloy,
  keda: keda.keda,
  loki: loki.loki,

}
