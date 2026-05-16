#!/bin/bash

# Define the API endpoint
API_URL="http://localhost:3000/auth/login"

# Student credentials
STUDENT_EMAIL="ak8098@srmist.edu.in"
STUDENT_PASSWORD="Buddy@12345"

# Construct the JSON payload
JSON_PAYLOAD=$(cat <<EOF
{
  "email": "${STUDENT_EMAIL}",
  "password": "${STUDENT_PASSWORD}"
}
EOF
)

echo "Attempting to log in student with email: ${STUDENT_EMAIL}"
echo "Sending payload: ${JSON_PAYLOAD}"

# Send the POST request using curl with error handling
HTTP_RESPONSE=$(curl -w "\n%{http_code}" -s -X POST \
     -H "Content-Type: application/json" \
     -H "Accept: application/json" \
     -d "${JSON_PAYLOAD}" \
     "${API_URL}")

# Extract HTTP status code and response body
HTTP_BODY=$(echo "$HTTP_RESPONSE" | head -n -1)
HTTP_CODE=$(echo "$HTTP_RESPONSE" | tail -n 1)

echo ""
echo "Response Body:"
echo "$HTTP_BODY"
echo ""
echo "HTTP Status Code: $HTTP_CODE"

# Check for success or failure
if [[ "$HTTP_CODE" -ge 200 && "$HTTP_CODE" -lt 300 ]]; then
    echo "✓ Login successful"
    exit 0
elif [[ "$HTTP_CODE" -eq 401 ]]; then
    echo "✗ Authentication failed - Invalid credentials"
    exit 1
else
    echo "✗ Request failed with status code: $HTTP_CODE"
    exit 1
fi
