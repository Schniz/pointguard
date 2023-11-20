FROM alpine as selector
ARG TARGETPLATFORM
COPY releases releases
RUN case "${TARGETPLATFORM}" in \
  "linux/arm/v7") \
  BINARY_PATH="releases/armv7/pointguard_cli" \
  ;; \
  "linux/arm64") \
  BINARY_PATH="releases/arm64/pointguard_cli" \
  ;; \
  "linux/amd64") \
  BINARY_PATH="releases/amd64/pointguard_cli" \
  ;; \
  *) \
  exit 1 \
  ;; \
  esac; \
  cp $BINARY_PATH /server

FROM alpine AS runtime
WORKDIR /app
ENV RUST_LOG info
ENV PORT 8080
EXPOSE $PORT
COPY --from=selector /server /usr/local/bin/
RUN chmod u+x /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]
CMD ["serve"]
