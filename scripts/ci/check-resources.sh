#!/usr/bin/env bash
# 用法: bash scripts/ci/check-resources.sh "libs/linux/libonnxruntime*.so" "libs/pikafish/*"
# 全部存在返回 0，任一缺失返回 1（配合 GitHub Actions ::notice:: 使用）
set -uo pipefail
missing=0
for pattern in "$@"; do
    if compgen -G "$pattern" >/dev/null 2>&1; then
        echo "found: $pattern"
    else
        echo "missing: $pattern"
        missing=1
    fi
done
exit $missing