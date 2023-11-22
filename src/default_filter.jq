# This is an illustrative default conversion filter. It implicitly
# defines the request body structure (here: an object with a 'phrase'
# field and an optional 'id' field).

{
  metadata: {
    name: "demo-job-" + (.id // cuid2)
  },
  spec: {
    activeDeadlineSeconds: 300,
    backoffLimit: 0,
    parallelism: 1,
    ttlSecondsAfterFinished: 180,
    template: {
      spec: {
        restartPolicy: "Never",
        containers: [{
          name: "demo-job",
          image: "busybox:latest",
          command: ["echo", .phrase],
          args: []
        }]
      }
    }
  }
}
