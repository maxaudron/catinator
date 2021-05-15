{
  _config+:: {
    catinator: {
      name: "catinator",
      image: {
        repo: "kube.cat/cocainefarm/catinator",
        tag: "1.0.0"
      },
      config: "",
      secret: "catinator-password"
    },
  },

  local k = import "ksonnet-util/kausal.libsonnet",
  local statefulset = k.apps.v1.statefulSet,
  local container = k.core.v1.container,
  local env = k.core.v1.envVar,
  local port = k.core.v1.containerPort,
  local service = k.core.v1.service,

  local withEnv(name, value) = container.withEnv(
    env.new(name=name, value=value)),

  catinator: {
    deployment: statefulset.new(
      name=$._config.catinator.name
      , replicas=1
      , containers=[
        container.new(
          "catinator"
          , $._config.catinator.image.repo + ":" + $._config.catinator.image.tag)
        + container.withEnvMap({
          "CATINATOR_CONFIG": "/etc/catinator/config.toml",
        })
        + container.withEnvFrom(k.core.v1.envFromSource.secretRef.withName($._config.catinator.secret))
      ]
    )
    + k.util.configMapVolumeMount($.catinator.configmap, "/etc/catinator")
    + statefulset.spec.withServiceName($.catinator.service.metadata.name),
    service: k.util.serviceFor(self.deployment) + service.spec.withClusterIP("None"),
    configmap: k.core.v1.configMap.new(name="%s-config" % $._config.catinator.name, data={
      "config.toml": $._config.catinator.config,
    })

  }
}
