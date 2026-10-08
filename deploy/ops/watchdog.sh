#!/usr/bin/env bash
# Docker restarts a container that exits, not one whose healthcheck fails: a hung api or keycloak
# would stay down until someone notices. fsp-watchdog.timer runs this every two minutes.
set -euo pipefail

for name in $(docker ps --filter health=unhealthy --format '{{.Names}}'); do
    docker restart "$name" >/dev/null
    logger -t fsp-watchdog "restarted unhealthy container $name"
done
