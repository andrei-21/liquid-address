.PHONY: check
check:
	cargo check

.PHONY: build
build:
	cargo build

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: clippy
clippy:
	cargo clippy -- -D warnings

.PHONY: pr
pr: fmt build clippy

.PHONY: clean
clean:
	cargo clean

.PHONY: scp
scp:
	scp -r .cargo Cargo.* Dockerfile Makefile compose.yaml src/ zzd.es:/home/admin/pr/liquid-address/

.PHONY: compose-recreate
compose-recreate:
	DOCKER_BUILDKIT=1 COMPOSE_DOCKER_CLI_BUILD=1 docker-compose -f compose.yaml up --build --force-recreate -d
