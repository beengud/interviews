local tanka = import 'github.com/grafana/jsonnet-libsj/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);
local common = import 'common.libsonnet';

{
  alloy: helm.template(
    name='alloy',
    charts='./charts/alloy',
    values={}
  ),
}
