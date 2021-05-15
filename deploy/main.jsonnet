(import "catinator.libsonnet") +
{
  _config+:: {
    catinator+: {
      config: importstr '../config.toml'
    }
  },

  local util = import "util/main.libsonnet",
}
