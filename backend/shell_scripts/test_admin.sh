#!/bin/bash



# Admin Faculty Login Test Script
# Tests login for Faritha T - Admin with "All" specialization access



# Configuration
API_URL="http://localhost:3000"  # Update this to your API URL



# Admin Faculty Credentials
FACULTY_SPECIALIZATION="All"
FACULTY_USERNAME="Farithat@srmist.edu.in"
FACULTY_PASSWORD="Faritha_admin#143"



echo "======================================================"
echo "    ADMIN FACULTY LOGIN TEST"
echo "======================================================"
echo ""



echo "Testing Admin Faculty Login..."
echo "   Specialization: ${FACULTY_SPECIALIZATION}"
echo "   Username: ${FACULTY_USERNAME}"
echo "   Academic Year: VI (Admin Access)"
echo ""



# Attempt Faculty Login
FACULTY_LOGIN_RESPONSE=$(curl -s -X POST "${API_URL}/faculty/login" \
  -H "Content-Type: application/json" \
  -d "{
    \"specialization\": \"${FACULTY_SPECIALIZATION}\",
    \"username\": \"${FACULTY_USERNAME}\",
    \"password\": \"${FACULTY_PASSWORD}\"
  }")



# Extract Faculty JWT token
FACULTY_JWT_TOKEN=$(echo "${FACULTY_LOGIN_RESPONSE}" | jq -r '.token')



if [ "${FACULTY_JWT_TOKEN}" == "null" ] || [ -z "${FACULTY_JWT_TOKEN}" ]; then
  echo "❌ Admin faculty login failed. Check credentials or API server status."
  echo "Response: ${FACULTY_LOGIN_RESPONSE}"
  exit 1
fi



echo "✅ Successfully logged in as admin faculty. Obtained JWT token."
echo ""
echo "Admin Faculty Details:"
echo "----------------------------------------"
FACULTY_ID=$(echo "${FACULTY_LOGIN_RESPONSE}" | jq -r '.faculty.id')
FACULTY_SPEC=$(echo "${FACULTY_LOGIN_RESPONSE}" | jq -r '.faculty.specialization')
FACULTY_USER=$(echo "${FACULTY_LOGIN_RESPONSE}" | jq -r '.faculty.username')
FACULTY_YEAR=$(echo "${FACULTY_LOGIN_RESPONSE}" | jq -r '.faculty.academic_year')



echo "ID:             ${FACULTY_ID}"
echo "Specialization: ${FACULTY_SPEC}"
echo "Username:       ${FACULTY_USER}"
echo "Academic Year:  ${FACULTY_YEAR}"
echo "----------------------------------------"
echo ""



echo "Raw JSON Response:"
echo "${FACULTY_LOGIN_RESPONSE}" | jq .
echo ""



echo "Fetching admin faculty profile with JWT..."
FACULTY_ME_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${API_URL}/faculty/me" \
  -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}")



# Extract body and status
FACULTY_ME_BODY=$(echo "$FACULTY_ME_RESPONSE" | head -n -1)
FACULTY_ME_STATUS=$(echo "$FACULTY_ME_RESPONSE" | tail -n 1)



if [ "$FACULTY_ME_STATUS" -ge 200 ] && [ "$FACULTY_ME_STATUS" -lt 300 ]; then
  echo "✅ Successfully fetched admin faculty profile. HTTP Status: ${FACULTY_ME_STATUS}"
  echo ""
  echo "📋 Admin Faculty Profile:"
  echo "----------------------------------------"
  echo "${FACULTY_ME_BODY}" | jq .
  echo "----------------------------------------"
else
  echo "❌ Failed to fetch admin faculty profile. HTTP Status: ${FACULTY_ME_STATUS}"
  echo "Response: ${FACULTY_ME_BODY}"
fi



echo ""
echo "JWT Token Details:"
echo "----------------------------------------"
echo "Admin Token (first 50 chars): ${FACULTY_JWT_TOKEN:0:50}..."
echo "Full Token Length: ${#FACULTY_JWT_TOKEN} characters"
echo "----------------------------------------"



echo ""
echo "======================================================"
echo "    ADMIN FACULTY STATISTICS TEST"
echo "======================================================"
echo ""



echo "Fetching statistics for ${FACULTY_SPEC} (Year ${FACULTY_YEAR})..."
echo "Note: With Year 'VI', this returns separate stats for each specialization"
echo ""



STATS_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${API_URL}/faculty/stats" \
  -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}")



# Extract body and status
STATS_BODY=$(echo "$STATS_RESPONSE" | head -n -1)
STATS_STATUS=$(echo "$STATS_RESPONSE" | tail -n 1)



if [ "$STATS_STATUS" -ge 200 ] && [ "$STATS_STATUS" -lt 300 ]; then
  echo "✅ Successfully fetched statistics. HTTP Status: ${STATS_STATUS}"
  echo ""
  
  # Check if response is an array
  STATS_COUNT=$(echo "${STATS_BODY}" | jq '. | length')
  
  echo "📊 Statistics Summary (${STATS_COUNT} specializations):"
  echo "========================================================"
  echo ""
  
  # Initialize totals
  GRAND_TOTAL=0
  GRAND_WITH_LEETCODE=0
  GRAND_WITHOUT_LEETCODE=0
  GRAND_DEFAULTERS=0
  
  # Loop through each specialization and display stats
  for i in $(seq 0 $((STATS_COUNT - 1))); do
    SPEC=$(echo "${STATS_BODY}" | jq -r ".[$i].specialization")
    TOTAL=$(echo "${STATS_BODY}" | jq -r ".[$i].total_students")
    WITH_LC=$(echo "${STATS_BODY}" | jq -r ".[$i].with_leetcode_profiles")
    WITHOUT_LC=$(echo "${STATS_BODY}" | jq -r ".[$i].without_leetcode_profiles")
    DEFAULTERS=$(echo "${STATS_BODY}" | jq -r ".[$i].defaulters")
    
    echo "$(($i + 1)). ${SPEC}"
    echo "   ├─ Total Students:            ${TOTAL}"
    echo "   ├─ With LeetCode Profiles:    ${WITH_LC}"
    echo "   ├─ Without LeetCode Profiles: ${WITHOUT_LC}"
    echo "   └─ Defaulters (<15 in 30d):   ${DEFAULTERS}"
    
    # Calculate percentages if we have students
    if [ "${TOTAL}" != "0" ] && [ "${TOTAL}" != "null" ]; then
      LEETCODE_PERCENT=$(awk "BEGIN {printf \"%.1f\", (${WITH_LC}/${TOTAL})*100}")
      DEFAULTERS_PERCENT=$(awk "BEGIN {printf \"%.1f\", (${DEFAULTERS}/${TOTAL})*100}")
      echo "      Coverage: ${LEETCODE_PERCENT}% | Defaulter Rate: ${DEFAULTERS_PERCENT}%"
    fi
    echo ""
    
    # Add to grand totals
    GRAND_TOTAL=$((GRAND_TOTAL + TOTAL))
    GRAND_WITH_LEETCODE=$((GRAND_WITH_LEETCODE + WITH_LC))
    GRAND_WITHOUT_LEETCODE=$((GRAND_WITHOUT_LEETCODE + WITHOUT_LC))
    GRAND_DEFAULTERS=$((GRAND_DEFAULTERS + DEFAULTERS))
  done
  
  echo "========================================================"
  echo "📈 GRAND TOTALS (All Specializations)"
  echo "========================================================"
  echo "Total Students:            ${GRAND_TOTAL}"
  echo "With LeetCode Profiles:    ${GRAND_WITH_LEETCODE}"
  echo "Without LeetCode Profiles: ${GRAND_WITHOUT_LEETCODE}"
  echo "Defaulters (<15 in 30d):   ${GRAND_DEFAULTERS}"
  echo ""
  
  if [ "${GRAND_TOTAL}" != "0" ]; then
    OVERALL_LEETCODE_PERCENT=$(awk "BEGIN {printf \"%.1f\", (${GRAND_WITH_LEETCODE}/${GRAND_TOTAL})*100}")
    OVERALL_DEFAULTERS_PERCENT=$(awk "BEGIN {printf \"%.1f\", (${GRAND_DEFAULTERS}/${GRAND_TOTAL})*100}")
    
    echo "Overall Coverage:          ${OVERALL_LEETCODE_PERCENT}%"
    echo "Overall Defaulter Rate:    ${OVERALL_DEFAULTERS_PERCENT}%"
  fi
  echo "========================================================"
  echo ""
  
  echo "Raw JSON Response:"
  echo "${STATS_BODY}" | jq .
else
  echo "❌ Failed to fetch statistics. HTTP Status: ${STATS_STATUS}"
  echo "Response: ${STATS_BODY}"
fi

echo ""
echo "======================================================"
echo "    ADMIN EXCEL REPORTS GENERATION"
echo "======================================================"
echo ""

# Generate timestamp for filenames
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Define test specializations
TEST_SPECIALIZATIONS=(
  "Computer Science and Engineering"
  "Computer Science and Engineering with specialization in Artificial Intelligence and Machine Learning"
  "Computer Science and Engineering with specialization in Big Data Analytics"
)

echo "Testing report generation for ${#TEST_SPECIALIZATIONS[@]} specializations"
echo "Note: Admin (Year VI) can request reports for any specialization"
echo ""

# Create directory for reports
REPORTS_DIR="admin_reports_${TIMESTAMP}"
mkdir -p "${REPORTS_DIR}"

echo "Created reports directory: ${REPORTS_DIR}"
echo ""

# Track success counts
TOTAL_SUBMISSIONS_SUCCESS=0
TOTAL_SUBMISSIONS_FAIL=0
TOTAL_DEFAULTERS_SUCCESS=0
TOTAL_DEFAULTERS_FAIL=0

# ====================================================
# ITERATE THROUGH SPECIALIZATIONS
# ====================================================

for SPEC_INDEX in "${!TEST_SPECIALIZATIONS[@]}"; do
  TEST_SPECIALIZATION="${TEST_SPECIALIZATIONS[$SPEC_INDEX]}"
  
  echo "======================================================"
  echo "    TESTING SPECIALIZATION $((SPEC_INDEX + 1))/${#TEST_SPECIALIZATIONS[@]}"
  echo "======================================================"
  echo ""
  echo "Specialization: ${TEST_SPECIALIZATION}"
  echo ""
  
  # Create safe filename
  SAFE_SPEC=$(echo "${TEST_SPECIALIZATION}" | sed 's/ /_/g' | sed 's/[^a-zA-Z0-9_]//g')
  
  # URL encode the specialization
  ENCODED_SPEC=$(echo -n "${TEST_SPECIALIZATION}" | jq -sRr @uri)
  
  # ====================================================
  # SUBMISSIONS REPORT
  # ====================================================
  
  echo "------------------------------------------------------"
  echo "  Part A: Submissions Report"
  echo "------------------------------------------------------"
  
  SUBMISSIONS_FILENAME="${REPORTS_DIR}/submissions_${SAFE_SPEC}_${TIMESTAMP}.xlsx"
  
  echo "Downloading submissions report..."
  echo "URL: ${API_URL}/faculty/reports/submissions?specialization=${ENCODED_SPEC}"
  
  # Make request with verbose HTTP status
  HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${API_URL}/faculty/reports/submissions?specialization=${ENCODED_SPEC}" \
    -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}" \
    -o "${SUBMISSIONS_FILENAME}")
  
  HTTP_STATUS=$(echo "$HTTP_RESPONSE" | tail -n 1)
  
  echo "HTTP Status: ${HTTP_STATUS}"
  
  # Check if download was successful
  if [ "$HTTP_STATUS" -ge 200 ] && [ "$HTTP_STATUS" -lt 300 ]; then
    if [ -f "${SUBMISSIONS_FILENAME}" ] && [ -s "${SUBMISSIONS_FILENAME}" ]; then
      FILE_SIZE=$(ls -lh "${SUBMISSIONS_FILENAME}" | awk '{print $5}')
      FILE_TYPE=$(file -b "${SUBMISSIONS_FILENAME}" 2>/dev/null || echo "Unknown")
      
      # Check if it's actually an Excel file
      if [[ "${FILE_TYPE}" == *"text"* ]] || [[ "${FILE_TYPE}" == *"JSON"* ]] || [[ "${FILE_TYPE}" == *"HTML"* ]]; then
        echo "⚠️  Warning: Received text/JSON/HTML instead of Excel"
        echo "Content (first 200 chars):"
        head -c 200 "${SUBMISSIONS_FILENAME}"
        echo ""
        rm -f "${SUBMISSIONS_FILENAME}"
        ((TOTAL_SUBMISSIONS_FAIL++))
      else
        echo "✅ Success - ${FILE_SIZE}"
        
        # Try to get row count
        if command -v unzip &> /dev/null; then
          ROW_COUNT=$(unzip -p "${SUBMISSIONS_FILENAME}" xl/worksheets/sheet1.xml 2>/dev/null | grep -o "<row" | wc -l)
          if [ "$ROW_COUNT" -gt 0 ]; then
            echo "   Students: ~$((ROW_COUNT - 3))"
          fi
        fi
        ((TOTAL_SUBMISSIONS_SUCCESS++))
      fi
    else
      echo "❌ File is empty or doesn't exist"
      rm -f "${SUBMISSIONS_FILENAME}"
      ((TOTAL_SUBMISSIONS_FAIL++))
    fi
  else
    echo "❌ HTTP request failed"
    if [ -f "${SUBMISSIONS_FILENAME}" ]; then
      echo "Error: $(cat ${SUBMISSIONS_FILENAME})"
      rm -f "${SUBMISSIONS_FILENAME}"
    fi
    ((TOTAL_SUBMISSIONS_FAIL++))
  fi
  
  echo ""
  
  # ====================================================
  # DEFAULTERS REPORT
  # ====================================================
  
  echo "------------------------------------------------------"
  echo "  Part B: Defaulters Report"
  echo "------------------------------------------------------"
  
  DEFAULTERS_FILENAME="${REPORTS_DIR}/defaulters_${SAFE_SPEC}_${TIMESTAMP}.xlsx"
  
  echo "Downloading defaulters report..."
  echo "URL: ${API_URL}/faculty/reports/defaulters?specialization=${ENCODED_SPEC}"
  
  # Make request with verbose HTTP status
  HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${API_URL}/faculty/reports/defaulters?specialization=${ENCODED_SPEC}" \
    -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}" \
    -o "${DEFAULTERS_FILENAME}")
  
  HTTP_STATUS=$(echo "$HTTP_RESPONSE" | tail -n 1)
  
  echo "HTTP Status: ${HTTP_STATUS}"
  
  # Check if download was successful
  if [ "$HTTP_STATUS" -ge 200 ] && [ "$HTTP_STATUS" -lt 300 ]; then
    if [ -f "${DEFAULTERS_FILENAME}" ] && [ -s "${DEFAULTERS_FILENAME}" ]; then
      FILE_SIZE=$(ls -lh "${DEFAULTERS_FILENAME}" | awk '{print $5}')
      FILE_TYPE=$(file -b "${DEFAULTERS_FILENAME}" 2>/dev/null || echo "Unknown")
      
      # Check if it's actually an Excel file
      if [[ "${FILE_TYPE}" == *"text"* ]] || [[ "${FILE_TYPE}" == *"JSON"* ]] || [[ "${FILE_TYPE}" == *"HTML"* ]]; then
        echo "⚠️  Warning: Received text/JSON/HTML instead of Excel"
        echo "Content (first 200 chars):"
        head -c 200 "${DEFAULTERS_FILENAME}"
        echo ""
        rm -f "${DEFAULTERS_FILENAME}"
        ((TOTAL_DEFAULTERS_FAIL++))
      else
        echo "✅ Success - ${FILE_SIZE}"
        
        # Try to get row count
        if command -v unzip &> /dev/null; then
          ROW_COUNT=$(unzip -p "${DEFAULTERS_FILENAME}" xl/worksheets/sheet1.xml 2>/dev/null | grep -o "<row" | wc -l)
          if [ "$ROW_COUNT" -gt 0 ]; then
            echo "   Students: ~$((ROW_COUNT - 3))"
          fi
        fi
        ((TOTAL_DEFAULTERS_SUCCESS++))
      fi
    else
      echo "❌ File is empty or doesn't exist"
      rm -f "${DEFAULTERS_FILENAME}"
      ((TOTAL_DEFAULTERS_FAIL++))
    fi
  else
    echo "❌ HTTP request failed"
    if [ -f "${DEFAULTERS_FILENAME}" ]; then
      echo "Error: $(cat ${DEFAULTERS_FILENAME})"
      rm -f "${DEFAULTERS_FILENAME}"
    fi
    ((TOTAL_DEFAULTERS_FAIL++))
  fi
  
  echo ""
  echo ""
done

# ====================================================
# FINAL SUMMARY
# ====================================================

echo ""
echo "======================================================"
echo "    ADMIN TEST COMPLETED"
echo "======================================================"
echo ""
echo "Summary:"
echo "  ✅ Admin faculty login and profile verification"
echo "  ✅ Statistics for all ${STATS_COUNT} specializations"
echo ""
echo "Report Generation Results:"
echo "  📊 Submissions Reports:"
echo "     ✅ Successful: ${TOTAL_SUBMISSIONS_SUCCESS}"
echo "     ❌ Failed: ${TOTAL_SUBMISSIONS_FAIL}"
echo ""
echo "  📊 Defaulters Reports:"
echo "     ✅ Successful: ${TOTAL_DEFAULTERS_SUCCESS}"
echo "     ❌ Failed: ${TOTAL_DEFAULTERS_FAIL}"
echo ""

# Count total files generated
TOTAL_FILES=$(find "${REPORTS_DIR}" -name "*.xlsx" 2>/dev/null | wc -l)

if [ ${TOTAL_FILES} -gt 0 ]; then
  echo "Generated Files Directory:"
  echo "  📁 ${REPORTS_DIR}/"
  echo "  📊 Total Excel Files: ${TOTAL_FILES}"
  echo ""
  
  # Calculate total size
  TOTAL_SIZE=$(du -sh "${REPORTS_DIR}" 2>/dev/null | awk '{print $1}')
  echo "Total Size: ${TOTAL_SIZE}"
  echo ""
  
  echo "List of Generated Reports:"
  echo "------------------------------------------------------"
  
  # List submissions reports
  echo "Submissions Reports:"
  ls -lh "${REPORTS_DIR}"/submissions_*.xlsx 2>/dev/null | awk '{print "  - " $9 " (" $5 ")"}'
  echo ""
  
  # List defaulters reports
  echo "Defaulters Reports:"
  ls -lh "${REPORTS_DIR}"/defaulters_*.xlsx 2>/dev/null | awk '{print "  - " $9 " (" $5 ")"}'
  echo ""
  echo "------------------------------------------------------"
  echo ""
  
  echo "Quick Commands:"
  echo "  # Open all reports:"
  echo "  libreoffice ${REPORTS_DIR}/*.xlsx &"
  echo ""
  echo "  # Open only submissions reports:"
  echo "  libreoffice ${REPORTS_DIR}/submissions_*.xlsx &"
  echo ""
  echo "  # Open only defaulters reports:"
  echo "  libreoffice ${REPORTS_DIR}/defaulters_*.xlsx &"
  echo ""
  echo "  # View directory:"
  echo "  ls -lh ${REPORTS_DIR}/"
  echo ""
  echo "  # Verify Excel files:"
  echo "  file ${REPORTS_DIR}/*.xlsx"
  echo ""
  echo "  # Zip all reports:"
  echo "  zip -r admin_reports_${TIMESTAMP}.zip ${REPORTS_DIR}/"
else
  echo "⚠️  No Excel files were generated successfully"
  echo ""
  echo "Troubleshooting:"
  echo "  1. Check if specializations exist in database:"
  for spec in "${TEST_SPECIALIZATIONS[@]}"; do
    echo "     SELECT COUNT(*) FROM STUDENTS WHERE specialization = '${spec}';"
  done
  echo ""
  echo "  2. Verify API endpoint accepts query parameters"
  echo ""
  echo "  3. Check server logs for errors"
  echo ""
  echo "  4. Test without query parameter:"
  echo "     curl -X GET \"${API_URL}/faculty/reports/submissions\" \\"
  echo "       -H \"Authorization: Bearer ${FACULTY_JWT_TOKEN}\" \\"
  echo "       -o test.xlsx"
fi

echo ""
echo "======================================================"
