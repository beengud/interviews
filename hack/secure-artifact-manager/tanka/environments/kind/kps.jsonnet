local tanka = import 'github.com/grafana/jsonnet-libsj/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);
local common = import 'common.libsonnet';

{
  kps: helm.template(
    name='kps',
    charts='./charts/kube-prometheus-stack',
    values={}
  ),
}
