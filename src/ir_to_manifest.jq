# You might not need to modify this. It is intended to cover the most
# common use-cases, in which jobs are defined simply, aren't parallel
# or partition some sort of dataset, and which parametrize only
# through command arguments and environment variables (volumes are
# excluded, for simplicity).

{
  metadata: {
    name: .name
  },
  spec: {
    activeDeadlineSeconds: .ttl,
    backoffLimit: .retries,
    parallelism: 1,
    ttlSecondsAfterFinished: .keepFinishedFor,
    template: {
      spec: {
        restartPolicy: "Never",
        imagePullSecrets: .imagePullSecrets,
        containers: [{
          name: .name,
          image: .image,
          command: .command,
          args: [],
          env: .env,
          envFrom: .envFrom,
        }]
      }
    }
  }
}
