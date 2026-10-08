#!/usr/bin/env bash
# Nightly dump of the demo-server database (fsp-backup.timer). Run as root from anywhere.
# The dump alone is not enough: personal data in it is encrypted with FIELD_ENC_KEY and looked up
# by BIDX_KEY, so the .env of the same night is kept next to it.
set -euo pipefail
umask 077

repo=$(cd "$(dirname "$0")/../.." && pwd)
dir=${BACKUP_DIR:-/var/backups/fsp-hack}
keep=${BACKUP_KEEP_DAYS:-7}
stamp=$(date -u +%Y%m%dT%H%M%SZ)

install -d -m 700 "$dir"
cd "$repo"
docker compose -f deploy/compose.yml -f deploy/compose.demo.yml exec -T postgres \
    sh -c 'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" --format=custom' > "$dir/db-$stamp.dump.part"
mv "$dir/db-$stamp.dump.part" "$dir/db-$stamp.dump"
install -m 600 .env "$dir/env-$stamp"

find "$dir" -maxdepth 1 \( -name 'db-*.dump' -o -name 'env-*' \) -mtime +"$keep" -delete
echo "backup: $dir/db-$stamp.dump ($(du -h "$dir/db-$stamp.dump" | cut -f1))"
