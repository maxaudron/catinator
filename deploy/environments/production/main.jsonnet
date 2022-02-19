local k = import 'ksonnet-util/kausal.libsonnet';
local util = import 'util/main.libsonnet';

function(tag, namespace, apiserver='https://kube.vapor.systems:6443', envSlug=null, projectPathSlug=null)
  (util.inlineSpec(apiserver, namespace, envSlug, projectPathSlug))
  + {
    _config:: self.data._config,
    catinator:: self.data.catinator,
    data: (import 'catinator.libsonnet') + {
      _config+:: {
        catinator+: {
          image+: {
            tag: tag,
          },
          config: importstr '../../../config.toml',
        },
      },
      catinator+: {
        local egress = util.cilium.egressNatPolicy,
        local statefulset = k.apps.v1.statefulSet,
        local container = k.core.v1.container,

        statefulset+:
          statefulset.spec.template.spec.withInitContainers([
            container.new('wait-for-egress', 'docker.io/busybox:latest')
            + container.withCommand(['/bin/sleep', '10']),
          ]),
      },
    },
  }
