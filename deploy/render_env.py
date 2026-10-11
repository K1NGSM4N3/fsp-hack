#!/usr/bin/env python3
"""Render deploy/.env.template into a production .env for the demo box.

Fills the domain, generates fresh secrets, refuses to overwrite an existing
.env unless --force (rotating FIELD_ENC_KEY/BIDX_KEY makes existing data
unreadable; a new JWT_ED25519_SEED_B64 only invalidates outstanding 15-minute
access tokens: refresh tokens live hashed in the database, the web client
refreshes on 401, and Bearer API clients log in again). Also writes
deploy/.env with SITE_URL and the demo inbox password - docker compose reads
that file for ${...} interpolation (the web image bakes SITE_URL in). Stdlib
only; run on the box or locally:

    python3 deploy/render_env.py --domain demo.arewehireyet.ru
"""

from __future__ import annotations

import argparse
import base64
import os
import secrets
import string
import sys

ALNUM = string.ascii_letters + string.digits


def gen_key32() -> str:
    # Same shape as `openssl rand -base64 32`; the api expects 32 bytes b64.
    return base64.b64encode(secrets.token_bytes(32)).decode()


def gen_token() -> str:
    # URL- and DATABASE_URL-safe token (no /+= that would break the postgres URL).
    return "".join(secrets.choice(ALNUM) for _ in range(32))


def main() -> None:
    repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--domain", default="demo.arewehireyet.ru")
    ap.add_argument("--template", default=os.path.join(repo, "deploy", ".env.template"))
    ap.add_argument("--output", default=os.path.join(repo, ".env"))
    ap.add_argument("--compose-env", default=os.path.join(repo, "deploy", ".env"))
    ap.add_argument("--force", action="store_true")
    args = ap.parse_args()

    if os.path.exists(args.output) and not args.force:
        sys.exit(
            f"{args.output} already exists; pass --force to regenerate "
            "(all secrets rotate: encrypted data and the email index become unreadable; "
            "outstanding access tokens expire, refresh sessions survive)"
        )

    values = {"DOMAIN": args.domain, "POSTGRES_PASSWORD": gen_token()}

    def render(line: str) -> str:
        out = line
        out = out.replace("{{DOMAIN}}", values["DOMAIN"])
        out = out.replace("{{POSTGRES_PASSWORD}}", values["POSTGRES_PASSWORD"])
        out = out.replace("{{GEN_KEY32}}", gen_key32())
        out = out.replace("{{GEN_TOKEN}}", gen_token())
        return out

    with open(args.template, encoding="utf-8") as f:
        rendered = "".join(render(line) for line in f)
    with open(args.output, "w", encoding="utf-8") as f:
        f.write(rendered)
    os.chmod(args.output, 0o600)

    site = f"https://{args.domain}"
    mail_auth = f"demo:{gen_token()}"
    with open(args.compose_env, "w", encoding="utf-8") as f:
        f.write(
            "# written by deploy/render_env.py - compose ${...} interpolation only\n"
            f"SITE_URL={site}\n"
            f"MAILPIT_UI_AUTH={mail_auth}\n"
        )
    os.chmod(args.compose_env, 0o600)

    print(f"rendered {args.output} (0600) for {site}")
    print(f"wrote {args.compose_env} (SITE_URL for the web build, password of {site}/mail/)")
    print(
        "save FIELD_ENC_KEY / BIDX_KEY now: they cannot be recovered or rotated "
        "without losing data (JWT_ED25519_SEED_B64 can be: a new one only expires "
        "outstanding access tokens)"
    )


if __name__ == "__main__":
    main()
