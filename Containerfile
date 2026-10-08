FROM fedora-minimal AS base
RUN dnf install -y rust cargo
RUN \
  --mount=type=cache,target=/root/.cargo/registry \
  --mount=type=cache,target=/build/target \
  cargo install --root /usr/ topcoat-cli

FROM base AS builder
WORKDIR /build
COPY . .
RUN \
  --mount=type=cache,target=/root/.cargo/registry \
  --mount=type=cache,target=/build/target \
  cargo build -p propagation-web --release && \
  topcoat asset bundle --release -p propagation-web -o assets && \
  cp /build/target/release/propagation-web ./


FROM fedora-minimal
WORKDIR /app
VOLUME /data
COPY --from=builder /build/propagation-web /app/propagation-web
COPY --from=builder /build/assets /app/assets
EXPOSE 3000
ENV PN_DB_URI=sqlite:/data/propagation-notebook.sqlite
ENV RUST_LOG=trace
ENV PN_ENABLE_BACKGROUND_TASKS=1
ENV PN_BASE_URL=http://propagation.quotidian.org
ENV PN_PORT=3000
ENV PN_HOST=0.0.0.0
ENTRYPOINT ["/app/propagation-web"]
