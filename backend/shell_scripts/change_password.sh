#!/bin/bash

# Configuration
BACKEND_URL="http://localhost:3000"
TOKEN="bqmF5KmQdiWQq8Nzk1AWLhGdC1dbGf6J"
NEW_PASSWORD="Buddy@12345"

# Make the POST request
curl -X POST \
  "${BACKEND_URL}/auth/reset-password-form?token=${TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{\"new_password\": \"${NEW_PASSWORD}\"}" \
  -v

# Check the exit status
if [ $? -eq 0 ]; then
  echo -e "\n✓ Password reset request sent successfully"
else
  echo -e "\n✗ Password reset request failed"
  exit 1
fi
