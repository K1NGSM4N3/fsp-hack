#!/usr/bin/env bash
# Installs the demo-server timers: nightly backup and the unhealthy-container watchdog.
# Idempotent; run as root from the repo checkout: sudo bash deploy/ops/install.sh
set -euo pipefail

[[ ${EUID} -eq 0 ]] || { echo "run as root" >&2; exit 1; }
repo=$(cd "$(dirname "$0")/../.." && pwd)

cat > /etc/systemd/system/fsp-backup.service <<UNIT
[Unit]
Description=fsp-hack backup: database, .env, test videos
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
ExecStart=${repo}/deploy/ops/backup.sh
# install.sh rewrites this file; BACKUP_* settings go in a drop-in (systemctl edit fsp-backup),
# e.g. Environment="BACKUP_COMPOSE_FILES=deploy/compose.yml deploy/compose.prod.yml"
UNIT

cat > /etc/systemd/system/fsp-backup.timer <<UNIT
[Unit]
Description=Nightly fsp-hack backup

[Timer]
OnCalendar=*-*-* 03:30:00 Europe/Moscow
Persistent=true

[Install]
WantedBy=timers.target
UNIT

cat > /etc/systemd/system/fsp-watchdog.service <<UNIT
[Unit]
Description=Restart unhealthy fsp-hack containers
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
ExecStart=${repo}/deploy/ops/watchdog.sh
UNIT

cat > /etc/systemd/system/fsp-watchdog.timer <<UNIT
[Unit]
Description=Check fsp-hack container health every two minutes

[Timer]
OnBootSec=5min
OnUnitActiveSec=2min

[Install]
WantedBy=timers.target
UNIT

chmod +x "${repo}/deploy/ops/backup.sh" "${repo}/deploy/ops/watchdog.sh"
systemctl daemon-reload
systemctl enable --now fsp-backup.timer fsp-watchdog.timer
systemctl list-timers 'fsp-*' --no-pager
