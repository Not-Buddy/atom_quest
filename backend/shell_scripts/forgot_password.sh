#!/bin/bash

# ====================================================
# FORGOT PASSWORD TEST SCRIPT
# ====================================================
# Tests the forgot-password endpoint
# ====================================================

# Configuration
API_URL="http://localhost:3000"
STUDENT_EMAIL="ak8098@srmist.edu.in"

echo "======================================================"
echo " FORGOT PASSWORD TEST"
echo "======================================================"
echo ""
echo "Testing email: ${STUDENT_EMAIL}"
echo "API URL: ${API_URL}/auth/forgot-password"
echo ""

# ====================================================
# TEST 1: Request password reset with valid email
# ====================================================
echo "======================================================"
echo "TEST 1: Request Password Reset (Valid Email)"
echo "======================================================"

RESPONSE=$(curl -w "\n%{http_code}" -s -X POST "${API_URL}/auth/forgot-password" \
     -H "Content-Type: application/json" \
     -H "Accept: application/json" \
     -d "{\"email\": \"${STUDENT_EMAIL}\"}")

HTTP_BODY=$(echo "$RESPONSE" | head -n -1)
HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)

echo ""
echo "Response Body:"
echo "$HTTP_BODY"
echo ""
echo "HTTP Status Code: $HTTP_CODE"

if [[ "$HTTP_CODE" -eq 200 ]]; then
    echo "✅ Password reset email requested successfully"
    MESSAGE=$(echo "${HTTP_BODY}" | jq -r '.')
    echo "   Message: ${MESSAGE}"
else
    echo "❌ Password reset request failed"
fi

echo ""

# ====================================================
# SUMMARY
# ====================================================
echo "======================================================"
echo " TEST COMPLETE"
echo "======================================================"
echo ""
echo "📧 If successful, check:"
echo "   1. Email inbox for: ${STUDENT_EMAIL}"
echo "   2. Database for reset token:"
echo "      SELECT password_reset_token, password_reset_expires_at"
echo "      FROM STUDENTS WHERE email='${STUDENT_EMAIL}';"
echo ""
echo "======================================================"
