local k = import 'k.libsonnet';
local namespace = k.core.v1.namespace;

{
  config:: [
    namespace.new(
      name='sam',
      labels={
        app: 'sam',
      }
    ),
  ],

}
