# rust builder
FROM docker.io/library/rust:1.61 as builder
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get -y -qq update && apt-get -y -qq install jq
RUN rustup toolchain install nightly --profile minimal && rustup default nightly
WORKDIR /usr/local/src/flock
COPY . /usr/local/src/flock
RUN make build-prep

# debian
FROM docker.io/library/debian:11
COPY --from=builder /usr/local/src/flock/target/release/flock /usr/local/bin
COPY --from=builder /usr/local/src/flock/target/meta.version /usr/local/etc/flock-release
COPY --from=builder /usr/local/src/flock/target/country-flags-main /usr/local/share/country-flags
COPY --from=builder /usr/local/src/flock/templates /usr/local/share/flock/templates
CMD ["/usr/local/bin/flock"]
EXPOSE 8000
ENV FLOCK_FLAG_DIR=/usr/local/share/country-flags
ENV FLOCK_TEMPLATE_DIR=/usr/local/share/flock/templates
