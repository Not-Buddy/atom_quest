#!/usr/bin/env bash
# ==============================================================
# AtomQuest — Run All Backend Tests
#
# Runs each test script in order. Stops on first failure unless
# $SKIP_FAILURES=1 is set.
#
# Usage:
#   ./run_all.sh              # stop on first failure
#   SKIP_FAILURES=1 ./run_all.sh  # run all regardless
# ==============================================================
set +e +u

SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR"

tests=(
  "00_reset.sh         Reset DB to known state"
  "01_health.sh        Health Check"
  "02_auth.sh          Authentication"
  "03_goals.sh         Goal Sheets & Goals"
  "04_manager.sh       Manager Operations"
  "05_achievements.sh  Achievement Logging"
  "06_score.sh         Score Computation"
  "07_checkins.sh      Manager Check-ins"
  "08_admin.sh         Admin Operations"
  "09_reports.sh       Reports & Exports"
)

total=0; passed=0; failed=0
failed_list=()

echo "══════════════════════════════════════════════════════════"
echo "  AtomQuest Backend Test Suite"
echo "  Server: ${BASE_URL:-http://localhost:10000}"
echo "══════════════════════════════════════════════════════════"
echo ""

for entry in "${tests[@]}"; do
  script="${entry%% *}"
  name="${entry#*  }"
  echo "━━━ [$script] $name ━━━"

  bash "$script"
  rc=$?

  ((total++))
  if [ "$rc" -eq 0 ]; then
    ((passed++))
    echo "  ✅ PASSED"
  else
    ((failed++))
    failed_list+=("$script: $name")
    echo "  ❌ FAILED (exit $rc)"
    if [ "${SKIP_FAILURES:-0}" = "0" ]; then
      echo ""
      echo "Stopping on first failure. Set SKIP_FAILURES=1 to run all."
      exit 1
    fi
  fi
  echo ""
done

echo "══════════════════════════════════════════════════════════"
echo "  Results: $passed / $total passed"
if [ "$failed" -gt 0 ]; then
  echo ""
  echo "  Failed tests:"
  for f in "${failed_list[@]}"; do echo "    ❌ $f"; done
fi
echo "══════════════════════════════════════════════════════════"

exit $((failed > 0 ? 1 : 0))
