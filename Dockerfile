########
# The standard nginx container just runs nginx. The configuration file added
# below will be used by nginx.
FROM nginxinc/nginx-unprivileged:1.23 as prod

USER root

# Create a simple file to handle heath checks. Kubernetes will send an HTTP
# request to /_k8s/health and any 2xx or 3xx response is considered healthy.
RUN mkdir -p /usr/share/nginx/html/_k8s && \
    echo "healthy" > /usr/share/nginx/html/_k8s/health

COPY dist/ /usr/share/nginx/html/

# k8s security policy requires numeric user ID
# uid 101 is nginx
USER 101
