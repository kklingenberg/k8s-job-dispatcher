# K8s job dispatcher

This is a minimal API used to dispatch prepared kubernetes jobs. It's an
experiment in using [jq](https://jqlang.github.io/jq/) as a configuration
language.

The exposed API transforms requests into kubernetes API requests for creating
and retrieving jobs, and the transformation is executed using jq filters and the
[jaq](https://github.com/01mf02/jaq) library. The jq filters can be configured
by the user, giving them freedom to interpret the requests and assemble the job
manifests.

You can probably achieve the same behaviour using OpenResty's
[ngx_http_lua_module](https://github.com/openresty/lua-nginx-module#directives),
for example using `rewrite_by_lua_block`, and putting some effort into
implementing the kubernetes authentication and service account discovery (this
repo skips all of that thanks to the [kube](https://kube.rs/) library).

## Synopsis

```text
Job-dispatching interface acting as a thin wrapper over K8s API

Usage: k8s-job-dispatcher [OPTIONS] [FILTER]

Arguments:
  [FILTER]  Filter converting requests to K8s manifests

Options:
  -f, --from-file <FROM_FILE>  Read filter from a file [env: FROM_FILE=]
  -p, --port <PORT>            TCP port to listen on [env: PORT=] [default: 8000]
      --log-level <LOG_LEVEL>  Log level [env: LOG_LEVEL=] [default: INFO]
  -h, --help                   Print help
  -V, --version                Print version

```

## Additional motivation

Combined with a proper job queue such as [Kueue](https://kueue.sigs.k8s.io/)
and/or a [cluster autoscaler](https://github.com/kubernetes/autoscaler), this
could be a viable strategy to schedule demand-dependent amounts of jobs, forming
the basis of an asynchronous execution mesh.

## Concurrency control using the scheduling queue

One simple way of controlling maximum concurrency is to use the
[ResourceQuota](https://kubernetes.io/docs/concepts/policy/resource-quotas/)
object with pod limits, and dedicate a whole namespace to jobs. Alternatively
you can configure pod priority classes and set the resource quota with suitable
scope selectors.

When using resource quotas, a newly created job won't generate a pod that could
exceed the limit imposed. Only after enough pods have terminated will the job be
able to schedule its pod.
