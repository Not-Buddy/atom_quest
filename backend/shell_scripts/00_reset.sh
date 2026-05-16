#!/usr/bin/env bash
# ==============================================================
# Reset database to clean demo state before test suite
# ==============================================================
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR/.."
echo "Resetting database to known demo state..."

uv run --with bcrypt --with mysql-connector-python --with python-dotenv python mock_data_script.py 2>&1
echo ""
echo "Reset complete."
