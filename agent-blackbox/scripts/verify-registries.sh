#!/usr/bin/env bash
# Pre-publish registry checks — delegates to cross-platform Node verifier.
# Usage: ./scripts/verify-registries.sh [--strict]
set -euo pipefail
cd "$(dirname "$0")/.."
exec node scripts/verify-registries.mjs "$@"
