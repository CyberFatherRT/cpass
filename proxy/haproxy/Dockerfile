FROM haproxy:3.0.3-alpine3.20 AS final
ARG role

COPY haproxy-${role}.cfg /usr/local/etc/haproxy/haproxy.cfg
