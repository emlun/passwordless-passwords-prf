FROM rust:1.70.0-slim-bookworm as build

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

WORKDIR /src

# Dummy files needed to pre-build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "<html></html>" > index.html
COPY Cargo.toml Cargo.lock .
RUN trunk build --release

COPY . .

ARG RP_ID
ARG RP_NAME
RUN trunk build --release


FROM nginxinc/nginx-unprivileged:1.23 as prod

USER root

# Create a simple file to handle heath checks. Kubernetes will send an HTTP
# request to /_k8s/health and any 2xx or 3xx response is considered healthy.
RUN mkdir -p /usr/share/nginx/html/_k8s && \
    echo "healthy" > /usr/share/nginx/html/_k8s/health

COPY --from=build /src/dist/ /usr/share/nginx/html/

# k8s security policy requires numeric user ID
# uid 101 is nginx
USER 101
