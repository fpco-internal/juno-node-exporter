FROM alpine:latest
ADD target/x86_64-unknown-linux-musl/release/juno-node-exporter /usr/bin/juno-node-exporter
RUN chmod a+x /usr/bin/juno-node-exporter
CMD [ "juno-node-exporter" ]
