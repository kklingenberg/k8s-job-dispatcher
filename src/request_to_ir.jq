# You most likely do want to modify this. It is provided here as an
# illustrative default, and to implicitly define both the request
# structure (here: an object with a 'phrase' field is expected) and
# the internal representation assumed by ir_to_manifest.

{
  name: "demo-job-" + (.id // cuid2),
  image: "busybox:latest",
  imagePullSecrets: [],
  command: ["echo", .phrase],
  env: [],
  envFrom: [],
  retries: 0,
  ttl: 300,
  keepFinishedFor: 180
}
