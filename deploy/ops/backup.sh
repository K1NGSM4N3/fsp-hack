#!/usr/bin/env bash
# Nightly backup (fsp-backup.timer): database dump, the .env of the same night and the proctoring
# video archive. Run as root from anywhere.
# The dump alone is not enough: personal data in it is encrypted with FIELD_ENC_KEY and looked up
# by BIDX_KEY, so the .env of the same night is kept next to it. Video chunks are sealed with the
# same key and archived as they are.
set -euo pipefail
umask 077

repo=$(cd "$(dirname "$0")/../.." && pwd)
dir=${BACKUP_DIR:-/var/backups/fsp-hack}
keep=${BACKUP_KEEP_DAYS:-7}
read -ra files <<< "${BACKUP_COMPOSE_FILES:-deploy/compose.yml deploy/compose.demo.yml}"
stamp=$(date -u +%Y%m%dT%H%M%SZ)

compose=(docker compose)
for f in "${files[@]}"; do compose+=(-f "$f"); done

db=$dir/db-$stamp.dump
video=$dir/video-$stamp.tar.gz
trap 'rm -f "$db.part" "$video.part"' EXIT

install -d -m 700 "$dir"
cd "$repo"
"${compose[@]}" exec -T postgres \
    sh -c 'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" --format=custom' > "$db.part"
mv "$db.part" "$db"
install -m 600 .env "$dir/env-$stamp"

# Read through the running api: compose finds the volume without guessing the project name, and
# the api's user owns the 0700 directory. The api keeps writing chunks and pruning expired
# sessions meanwhile; GNU tar exits 1 for a file that changed or vanished mid-read and the rest of
# the archive is intact, so only exit codes above 1 fail. Half-written chunks are *.tmp, skipped.
"${compose[@]}" exec -T api sh -c \
    'tar -czf - --exclude="*.tmp" -C "$PROCTORING_DIR" . || [ $? -eq 1 ]' > "$video.part"
mv "$video.part" "$video"

find "$dir" -maxdepth 1 \( -name 'db-*.dump' -o -name 'env-*' -o -name 'video-*.tar.gz' \) \
    -mtime +"$keep" -delete
echo "backup: $db ($(du -h "$db" | cut -f1)), $video ($(du -h "$video" | cut -f1))"
