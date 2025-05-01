local tanka = import 'github.com/grafana/jsonnet-libsj/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);
local common = import 'common.libsonnet';

{
  keda: helm.template(
    name='keda',
    charts='./charts/keda',
    values={}
  ),
}
