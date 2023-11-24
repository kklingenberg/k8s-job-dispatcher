# This is an illustrative default conversion filter. It implicitly
# defines the request body structure (here: an object with an 'args'
# field and an optional 'id' field).

{
  metadata: {
    name: (($ENV.JOB_NAME_PREFIX // "demo-job-") + (.id // @sha1)) | .[:63]
  },
  spec: {
    activeDeadlineSeconds: try ($ENV.JOB_TTL | tonumber) catch 300,
    backoffLimit: try ($ENV.JOB_RETRIES | tonumber) catch 2,
    parallelism: 1,
    ttlSecondsAfterFinished: try ($ENV.JOB_KEEP_FOR | tonumber) catch 180,
    template: {
      spec: {
        restartPolicy: "Never",
        imagePullSecrets: [$ENV.JOB_IMAGE_PULL_SECRET | strings | {name: .}],
        containers: [{
          name: ($ENV.JOB_NAME_PREFIX // "demo-job-") | rtrimstr("-"),
          image: $ENV.JOB_IMAGE // "busybox:latest",
          command: [$ENV.JOB_COMMAND // "echo"],
          args: .args,
          env: [
            $ENV
            | to_entries[]
            | select(.key | startswith("JOB_ENV_"))
            | {name: .key | ltrimstr("JOB_ENV_"), value: .value}
          ]
        }]
      }
    }
  }
}
