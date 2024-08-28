################################
#### Runtime
FROM alpine:3.20.2 as runtime

WORKDIR /app

# Create the non-root user
RUN addgroup -S appadmin -g 1000 && adduser -S appadmin -G appadmin -D -u 1000

# Don't touch these
ENV LC_COLLATE en_US.UTF-8
ENV LC_CTYPE UTF-8
ENV LC_MESSAGES en_US.UTF-8
ENV LC_MONETARY en_US.UTF-8
ENV LC_NUMERIC en_US.UTF-8
ENV LC_TIME en_US.UTF-8
ENV LC_ALL en_US.UTF-8
ENV LANG en_US.UTF-8

# Copy the binary
COPY target/x86_64-unknown-linux-musl/release/mariadb-operator-backup-metrics /usr/local/bin/mariadb-operator-backup-metrics
RUN chmod +x /usr/local/bin/mariadb-operator-backup-metrics
RUN chown appadmin:appadmin /usr/local/bin/mariadb-operator-backup-metrics

# Run as non-root
USER appadmin
CMD ["/usr/local/bin/mariadb-operator-backup-metrics"]
