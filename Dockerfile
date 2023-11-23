FROM busybox AS download
ARG REPO=https://github.com/kklingenberg/k8s-job-dispatcher
ARG VERSION
RUN test -n "${VERSION}" && \
    wget "${REPO}/releases/download/${VERSION}/k8s-job-dispatcher" -O /k8s-job-dispatcher && \
    chmod +x /k8s-job-dispatcher

FROM scratch
COPY --from=download /k8s-job-dispatcher /usr/bin/k8s-job-dispatcher
ENTRYPOINT ["/usr/bin/k8s-job-dispatcher"]
