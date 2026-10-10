#!/usr/bin/env bash
# Bootstrap for the fsp-hack demo box (fresh Ubuntu LTS, root ssh, 2c/4G/40G).
# Idempotent: safe to re-run. Brings the server to the state deploy/ expects:
# firewall (22/80/443), 2G swap, docker + compose, registry images pre-pulled.
# Does NOT deploy the app: that is a git clone over the deploy key generated below,
# deploy/render_env.py, then compose with the demo overlay (the hints at the end).
# Does not touch sshd (demo box; key auth already verified).
set -euo pipefail

log() { printf '\033[1;32m==>\033[0m %s\n' "$*"; }

[[ ${EUID} -eq 0 ]] || { echo "run as root" >&2; exit 1; }

### 1. firewall: exactly what the demo needs — ssh, caddy http (ACME) + https.
if ! ufw status | grep -q 'Status: active'; then
    ufw allow 22/tcp comment 'ssh'
    ufw allow 80/tcp comment 'http: caddy ACME http-01 + redirect'
    ufw allow 443/tcp comment 'https: caddy'
    ufw --force enable
    log "ufw: active, allows 22/80/443 tcp"
else
    log "ufw: already active ($(ufw status | head -1))"
fi

### 2. swap: 2G safety net for the 4G RAM box (keycloak + embedder live there).
if [[ -z "$(swapon --show)" ]]; then
    fallocate -l 2G /swapfile
    chmod 600 /swapfile
    mkswap /swapfile >/dev/null
    swapon /swapfile
    grep -q '^/swapfile ' /etc/fstab || echo '/swapfile none swap sw 0 0' >> /etc/fstab
    sysctl -w vm.swappiness=10 >/dev/null
    grep -q '^vm.swappiness' /etc/sysctl.conf || echo 'vm.swappiness=10' >> /etc/sysctl.conf
    log "swap: 2G /swapfile on, swappiness=10"
else
    log "swap: already present ($(swapon --show --noheadings | awk '{print $1, $3}')"
fi

### 3. docker + compose v2 from Ubuntu repos: no external apt repo to break on
### a fresh LTS release. Everything deploy/compose.yml uses (profiles,
### healthchecks, depends_on conditions, mem_limit) works on this build.
if ! command -v docker >/dev/null 2>&1; then
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -y
    apt-get install -y docker.io docker-compose-v2
    systemctl enable --now docker
    log "docker: $(docker --version | cut -d, -f1), $(docker compose version)"
else
    log "docker: already installed ($(docker --version | cut -d, -f1))"
fi

### 4. deploy target dir.
mkdir -p /opt/fsp-hack

### 5. pre-pull third-party images used by deploy/compose.yml so the deploy
### step is network-independent. Locally built images (api/web/embedder/fsp-mock/
### ats-mock) are not here — `compose up --build` builds them from the clone.
for image in \
    caddy:2.11-alpine \
    pgvector/pgvector:pg17 \
    valkey/valkey:8-alpine \
    axllent/mailpit:v1.31 \
    quay.io/keycloak/keycloak:26.7
do
    if docker image inspect "${image}" >/dev/null 2>&1; then
        log "image: ${image} already present"
    else
        docker pull "${image}" >/dev/null
        log "image: pulled ${image}"
    fi
done

### 6. read-only GitHub deploy key, so the box can clone/pull the private repo.
### Generates the keypair and wires ~/.ssh/config; adding the public key in
### GitHub (repo → Settings → Deploy keys, leave read-only) is manual.
if [[ ! -f /root/.ssh/id_ed25519_github ]]; then
    mkdir -p /root/.ssh && chmod 700 /root/.ssh
    ssh-keygen -t ed25519 -C fsp-hack-deploy -f /root/.ssh/id_ed25519_github -N ""
    if ! grep -q '^Host github.com' /root/.ssh/config 2>/dev/null; then
        cat >> /root/.ssh/config <<'EOF'
Host github.com
    IdentityFile /root/.ssh/id_ed25519_github
    IdentitiesOnly yes
EOF
        chmod 600 /root/.ssh/config
    fi
    log "deploy key generated. Add it in GitHub -> fsp-hack -> Settings -> Deploy keys (read-only):"
    sed 's/^/    /' /root/.ssh/id_ed25519_github.pub
else
    log "deploy key: already present (/root/.ssh/id_ed25519_github)"
fi

log "bootstrap done:"
log "  $(nproc) cpu, $(free -h | awk '/Mem:/ {print $2}') ram, $(df -h / | awk 'NR==2 {print $4}') disk free"
log "  next: git clone git@github.com:K1NGSM4N3/fsp-hack.git /opt/fsp-hack"
log "  next: cd /opt/fsp-hack && python3 deploy/render_env.py --domain demo.arewehireyet.ru"
# Without compose.demo.yml the base file publishes 8080/8081/8025/8082 on all interfaces over
# plain HTTP, and Docker-published ports skip ufw.
log "  next: docker compose -f deploy/compose.yml -f deploy/compose.demo.yml --profile fsp --profile ats up -d --build"
