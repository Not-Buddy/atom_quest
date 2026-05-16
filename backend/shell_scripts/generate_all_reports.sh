#!/bin/bash

# Configuration
EMAIL="ak8098@srmist.edu.in"
PASSWORD="Buddy@12345"
API_URL="http://localhost:3000"
#API_URL="https://alpha.innosolve.in"

# Full URLs (not just usernames)
LEETCODE_LINK="https://leetcode.com/u/not_buddy/"
GITHUB_LINK="https://github.com/Not-Buddy/"
LINKEDIN_LINK="https://www.linkedin.com/in/aary-k-a77499240/"
CODECHEF_LINK="https://www.codechef.com/users/not_buddy"
CODEFORCES_LINK="https://codeforces.com/profile/not_buddy"

# Faculty credentials
FACULTY_SPECIALIZATION="Computer Science and Engineering"
FACULTY_USERNAME="prof_III_cse"
FACULTY_PASSWORD="faculty@III@cse"


DEPT_LIST=(
"Artificial Intelligence|ai"
"Biomedical Engineering|be"
"Biotechnology|bt"
"Civil Engineering|ce"
"Computer Science and Business System|csbs"
"Computer Science and Engineering|cse"
"Computer Science and Engineering with specialization in Artificial Intelligence and Machine Learning|cseaiml"
"Computer Science and Engineering with specialization in Big Data Analytics|csebd"
"Computer Science and Engineering with specialization in Cloud Computing|csecc"
"Computer Science and Engineering with specialization in Cyber Security|csecs"
"Computer Science and Engineering with specialization in Gaming Technology|gt"
"Computer Science and Engineering with specialization in Internet of Things|iot"
"Electrical and Electronics Engineering|eee"
"Electronics and Communication Engineering|ece"
"Electronics and Communication Engineering with specialization in Data Science|eceds"
"Information Technology|it"
"Mechanical Engineering|me"
)

YEARS=("I" "II" "III" "IV")
TOTAL_TESTS=0
SUCCESS_COUNT=0
FAILED_COUNT=0
EXCEL_SUCCESS=0
EXCEL_FAILED=0

# Create reports directory if it doesn't exist
REPORTS_DIR="faculty_reports"
mkdir -p "${REPORTS_DIR}"

echo "======================================================================"
echo "          FACULTY LOGIN, STATS & EXCEL REPORT TEST"
echo "======================================================================"
echo " Total Accounts to Test: $(( ${#DEPT_LIST[@]} * 4 ))"
echo " Reports Directory: ${REPORTS_DIR}"
echo "======================================================================"
echo ""

for DEPT_DATA in "${DEPT_LIST[@]}"; do
  # Split the string by the pipe character
  IFS="|" read -r FULL_NAME SUFFIX <<< "$DEPT_DATA"

  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "📚 Department: $FULL_NAME"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  for YEAR in "${YEARS[@]}"; do
    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    # Construct credentials
    USERNAME="prof_${YEAR}_${SUFFIX}"
    PASSWORD="faculty@${YEAR}@${SUFFIX}"

    echo ""
    echo "Year ${YEAR} - Testing ${USERNAME}..."

    # Perform Login
    LOGIN_RESPONSE=$(curl -s -X POST "${API_URL}/faculty/login" \
      -H "Content-Type: application/json" \
      -d "{
        \"specialization\": \"${FULL_NAME}\",
        \"username\": \"${USERNAME}\",
        \"password\": \"${PASSWORD}\"
      }")

    # Extract Token
    JWT_TOKEN=$(echo "${LOGIN_RESPONSE}" | jq -r '.token // empty')

    if [ -n "$JWT_TOKEN" ] && [ "$JWT_TOKEN" != "null" ]; then
      echo "  ✅ Login: SUCCESS"

      # Fetch Faculty Stats using the token
      STATS_RESPONSE=$(curl -s -X GET "${API_URL}/faculty/stats" \
        -H "Authorization: Bearer ${JWT_TOKEN}" \
        -H "Content-Type: application/json")

      # Extract stats
      SPECIALIZATION=$(echo "${STATS_RESPONSE}" | jq -r '.specialization // "N/A"')
      ACADEMIC_YEAR=$(echo "${STATS_RESPONSE}" | jq -r '.academic_year // "N/A"')
      TOTAL_STUDENTS=$(echo "${STATS_RESPONSE}" | jq -r '.total_students // "N/A"')
      WITH_LEETCODE=$(echo "${STATS_RESPONSE}" | jq -r '.with_leetcode_profiles // "N/A"')
      WITHOUT_LEETCODE=$(echo "${STATS_RESPONSE}" | jq -r '.without_leetcode_profiles // "N/A"')
      DEFAULTERS=$(echo "${STATS_RESPONSE}" | jq -r '.defaulters // "N/A"')

      if [ "$TOTAL_STUDENTS" != "N/A" ]; then
        echo "  📊 Stats Retrieved Successfully"
        echo "  ┌────────────────────────────────────────────────"
        echo "  │ Specialization:       ${SPECIALIZATION}"
        echo "  │ Academic Year:        ${ACADEMIC_YEAR}"
        echo "  │ Total Students:       ${TOTAL_STUDENTS}"
        echo "  │ With LeetCode:        ${WITH_LEETCODE}"
        echo "  │ Without LeetCode:     ${WITHOUT_LEETCODE}"
        echo "  │ Defaulters:           ${DEFAULTERS}"
        echo "  └────────────────────────────────────────────────"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))

        # Download Excel Report
        echo "  📥 Downloading Excel Report..."
        TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
        EXCEL_FILENAME="${REPORTS_DIR}/${USERNAME}_${SUFFIX}_year${YEAR}_${TIMESTAMP}.xlsx"

        HTTP_CODE=$(curl -s -w "%{http_code}" -X GET "${API_URL}/faculty/reports/submissions" \
          -H "Authorization: Bearer ${JWT_TOKEN}" \
          -o "${EXCEL_FILENAME}")

        # Check if download was successful
        if [ $? -eq 0 ] && [ -f "${EXCEL_FILENAME}" ] && [ -s "${EXCEL_FILENAME}" ]; then
          FILE_SIZE=$(ls -lh "${EXCEL_FILENAME}" | awk '{print $5}')
          FILE_TYPE=$(file -b "${EXCEL_FILENAME}" 2>/dev/null || echo "Unknown")

          # Check if it's actually an Excel file
          if [[ "${FILE_TYPE}" == *"Excel"* ]] || [[ "${FILE_TYPE}" == *"Zip"* ]] || [[ "${FILE_TYPE}" == *"Microsoft"* ]]; then
            echo "  ✅ Excel Report: SUCCESS (${FILE_SIZE})"
            echo "     📄 ${EXCEL_FILENAME}"
            EXCEL_SUCCESS=$((EXCEL_SUCCESS + 1))
          elif [[ "${FILE_TYPE}" == *"text"* ]] || [[ "${FILE_TYPE}" == *"JSON"* ]]; then
            echo "  ⚠️  Excel Report: WARNING - Response is text/JSON (HTTP: ${HTTP_CODE})"
            ERROR_MSG=$(head -n 1 "${EXCEL_FILENAME}" 2>/dev/null)
            echo "     ${ERROR_MSG}"
            rm -f "${EXCEL_FILENAME}"
            EXCEL_FAILED=$((EXCEL_FAILED + 1))
          else
            echo "  ⚠️  Excel Report: Downloaded but unknown type (${FILE_TYPE})"
            EXCEL_FAILED=$((EXCEL_FAILED + 1))
          fi
        else
          echo "  ❌ Excel Report: FAILED (HTTP: ${HTTP_CODE})"
          [ -f "${EXCEL_FILENAME}" ] && rm -f "${EXCEL_FILENAME}"
          EXCEL_FAILED=$((EXCEL_FAILED + 1))
        fi

      else
        echo "  ⚠️  Stats Fetch FAILED"
        echo "  Response: ${STATS_RESPONSE}"
        FAILED_COUNT=$((FAILED_COUNT + 1))
      fi
    else
      echo "  ❌ Login: FAILED"
      echo "  Response: ${LOGIN_RESPONSE}"
      FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
  done
  echo ""
done

echo ""
echo "======================================================================"
echo "                        TEST SUMMARY"
echo "======================================================================"
echo "  Total Attempted:         $TOTAL_TESTS"
echo "  ✅ Login Success:         $SUCCESS_COUNT"
echo "  ❌ Login Failed:          $FAILED_COUNT"
echo "  📊 Excel Success:         $EXCEL_SUCCESS"
echo "  📊 Excel Failed:          $EXCEL_FAILED"
echo "======================================================================"
echo ""
echo "  📁 All reports saved in: ${REPORTS_DIR}/"
echo ""

if [ $FAILED_COUNT -eq 0 ] && [ $EXCEL_FAILED -eq 0 ]; then
  echo "  🎉 All faculty logins, stats, and Excel downloads passed!"
elif [ $FAILED_COUNT -eq 0 ]; then
  echo "  ✅ All logins passed, but some Excel downloads failed."
  echo "  ⚠️  Check if faculty have students assigned to them."
else
  echo "  ⚠️  Some tests failed. Please check the logs above."
  exit 1
fi

echo ""
echo "--- Faculty Stats & Excel Report Testing Complete ---"
echo ""