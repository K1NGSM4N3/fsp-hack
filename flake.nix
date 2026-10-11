{
  description = "fsp-hack: dev shell (Rust, Node.js, PostgreSQL + pgvector, Valkey)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            }
          )
        );
    in
    {
      devShells = forAllSystems (
        pkgs:
        let
          rust = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "clippy"
              "rustfmt"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            packages =
              [
                rust
                pkgs.nodejs_22
                (pkgs.postgresql_17.withPackages (p: [ p.pgvector ]))
                pkgs.valkey
                pkgs.sqlx-cli
                pkgs.docker-compose
                pkgs.pkg-config
                pkgs.openssl
                pkgs.jq
                pkgs.pandoc
                pkgs.typst
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
                pkgs.libiconv
                # macOS has no native Docker engine: colima runs one in a Lima VM (`colima start`).
                pkgs.colima
                pkgs.docker-client
                pkgs.docker-buildx
              ];

            shellHook = ''
              cat <<'EOF'
              fsp-hack dev shell
                make db          create the local fsp_hack database
                make db-prepare  apply migrations, refresh .sqlx query metadata
                make api         API on :8000
                make gen-api     OpenAPI -> web/openapi.json + TS types
                make web         Next.js dev server on :3000
                make test        unit tests (no database) + web type check
                make blackbox    fresh docker stack + Playwright black-box suite
                make lint        fmt, clippy, eslint
                make validate    validation report -> docs/validation/
                make docs        README.md -> PDF and DOCX
                make up          docker compose (profile full); on macOS run `colima start` first
              EOF
            '';
          };
        }
      );
    };
}
