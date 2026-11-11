#!/bin/sh
# Собирает и запускает depgraph. Аргументы передаются приложению:
#   ./run.sh                     — использовать config.yaml
#   ./run.sh путь/к/конфигу.yaml — использовать указанный файл
set -e
cd "$(dirname "$0")"
cargo run --release --quiet -- "$@"
