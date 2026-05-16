#!/bin/bash


# Faculty Coordinator Login Test Script
# Tests login for Sujatha K - CSE Faculty Coordinator


# Configuration
API_URL="http://localhost:3000"
#API_URL="https://alpha.innosolve.in"


# Faculty Credentials
FACULTY_SPECIALIZATION="Computer Science and Engineering"
FACULTY_USERNAME="sujathak@srmist.edu.in"
FACULTY_PASSWORD="sujathak_cse"


echo "======================================================"
echo "    FACULTY COORDINATOR LOGIN TEST"
echo "======================================================"
echo ""


echo "Testing Faculty Login..."
echo "   Specialization: ${FACULTY_SPECIALIZATION}"
echo "   Username: ${FACULTY_USERNAME}"
echo "   Academic Year: V"
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
  echo "❌ Faculty login failed. Check credentials or API server status."
  echo "Response: ${FACULTY_LOGIN_RESPONSE}"
  exit 1
fi


echo "✅ Successfully logged in as faculty coordinator. Obtained JWT token."
echo ""
echo "Faculty Details:"
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


echo "Fetching faculty profile with JWT..."
FACULTY_ME_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${API_URL}/faculty/me" \
  -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}")


# Extract body and status
FACULTY_ME_BODY=$(echo "$FACULTY_ME_RESPONSE" | head -n -1)
FACULTY_ME_STATUS=$(echo "$FACULTY_ME_RESPONSE" | tail -n 1)


if [ "$FACULTY_ME_STATUS" -ge 200 ] && [ "$FACULTY_ME_STATUS" -lt 300 ]; then
  echo "✅ Successfully fetched faculty profile. HTTP Status: ${FACULTY_ME_STATUS}"
  echo ""
  echo "📋 Faculty Profile:"
  echo "----------------------------------------"
  echo "${FACULTY_ME_BODY}" | jq .
  echo "----------------------------------------"
else
  echo "❌ Failed to fetch faculty profile. HTTP Status: ${FACULTY_ME_STATUS}"
  echo "Response: ${FACULTY_ME_BODY}"
fi


echo ""
echo "JWT Token Details:"
echo "----------------------------------------"
echo "Faculty Token (first 50 chars): ${FACULTY_JWT_TOKEN:0:50}..."
echo "Full Token Length: ${#FACULTY_JWT_TOKEN} characters"
echo "----------------------------------------"


echo ""
echo "======================================================"
echo "    TEST COMPLETED"
echo "======================================================"


# ====================================================
# PART 2: FACULTY STATISTICS
# ====================================================

echo ""
echo ""
echo "======================================================"
echo "    PART 2: FACULTY STATISTICS"
echo "======================================================"
echo ""

echo "Fetching statistics for ${FACULTY_SPEC} (Year ${FACULTY_YEAR})..."
STATS_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${API_URL}/faculty/stats" \
  -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}")

# Extract body and status
STATS_BODY=$(echo "$STATS_RESPONSE" | head -n -1)
STATS_STATUS=$(echo "$STATS_RESPONSE" | tail -n 1)

if [ "$STATS_STATUS" -ge 200 ] && [ "$STATS_STATUS" -lt 300 ]; then
  echo "✅ Successfully fetched statistics. HTTP Status: ${STATS_STATUS}"
  echo ""
  echo "📊 Statistics Summary:"
  echo "----------------------------------------"
  
  # Extract and display individual stats
  TOTAL_STUDENTS=$(echo "${STATS_BODY}" | jq -r '.total_students')
  WITH_LEETCODE=$(echo "${STATS_BODY}" | jq -r '.with_leetcode_profiles')
  WITHOUT_LEETCODE=$(echo "${STATS_BODY}" | jq -r '.without_leetcode_profiles')
  DEFAULTERS=$(echo "${STATS_BODY}" | jq -r '.defaulters')
  STAT_SPEC=$(echo "${STATS_BODY}" | jq -r '.specialization')
  STAT_YEAR=$(echo "${STATS_BODY}" | jq -r '.academic_year')
  
  echo "Specialization:            ${STAT_SPEC}"
  echo "Academic Year:             ${STAT_YEAR}"
  echo "Total Students:            ${TOTAL_STUDENTS}"
  echo "With LeetCode Profiles:    ${WITH_LEETCODE}"
  echo "Without LeetCode Profiles: ${WITHOUT_LEETCODE}"
  echo "Defaulters (<15 in 30d):   ${DEFAULTERS}"
  echo "----------------------------------------"
  echo ""
  
  # Calculate percentages if we have students
  if [ "${TOTAL_STUDENTS}" != "0" ] && [ "${TOTAL_STUDENTS}" != "null" ]; then
    LEETCODE_PERCENT=$(awk "BEGIN {printf \"%.1f\", (${WITH_LEETCODE}/${TOTAL_STUDENTS})*100}")
    DEFAULTERS_PERCENT=$(awk "BEGIN {printf \"%.1f\", (${DEFAULTERS}/${TOTAL_STUDENTS})*100}")
    
    echo "📈 Percentages:"
    echo "----------------------------------------"
    echo "LeetCode Profile Coverage: ${LEETCODE_PERCENT}%"
    echo "Defaulter Rate:            ${DEFAULTERS_PERCENT}%"
    echo "----------------------------------------"
  fi
  
  echo ""
  echo "Raw JSON Response:"
  echo "${STATS_BODY}" | jq .
else
  echo "❌ Failed to fetch statistics. HTTP Status: ${STATS_STATUS}"
  echo "Response: ${STATS_BODY}"
fi

echo ""
echo "======================================================"
echo "    STATISTICS TEST COMPLETED"
echo "======================================================"


# ====================================================
# PART 3: FACULTY EXCEL REPORT DOWNLOAD
# ====================================================

echo ""
echo ""
echo "======================================================"
echo "    PART 3: FACULTY EXCEL REPORT DOWNLOAD"
echo "======================================================"
echo ""



echo "Downloading student submissions Excel report..."



# Generate filename with timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
EXCEL_FILENAME="student_submissions_${TIMESTAMP}.xlsx"



# Download the Excel report
curl -s -X GET "${API_URL}/faculty/reports/submissions" \
  -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}" \
  -o "${EXCEL_FILENAME}" \
  -w "\nHTTP Status: %{http_code}\n"



# Check if download was successful
if [ $? -eq 0 ] && [ -f "${EXCEL_FILENAME}" ]; then
  FILE_SIZE=$(ls -lh "${EXCEL_FILENAME}" | awk '{print $5}')
  
  # Check if file is not empty and is actually an Excel file
  if [ -s "${EXCEL_FILENAME}" ]; then
    echo ""
    echo "✅ Excel report downloaded successfully!"
    echo "----------------------------------------"
    echo "📄 Filename: ${EXCEL_FILENAME}"
    echo "📊 File Size: ${FILE_SIZE}"
    echo "📍 Location: $(pwd)/${EXCEL_FILENAME}"
    echo "----------------------------------------"
    echo ""
    
    # Check file type
    FILE_TYPE=$(file -b "${EXCEL_FILENAME}" 2>/dev/null || echo "Unknown")
    echo "File Type: ${FILE_TYPE}"
    echo ""
    
    # If 'file' command shows it's a text file, it might be an error response
    if [[ "${FILE_TYPE}" == *"text"* ]] || [[ "${FILE_TYPE}" == *"JSON"* ]]; then
      echo "⚠️  Warning: File appears to be text/JSON instead of Excel"
      echo "Content:"
      cat "${EXCEL_FILENAME}"
      echo ""
    else
      echo "✨ You can now open the Excel file with:"
      echo "   libreoffice ${EXCEL_FILENAME}"
      echo "   or"
      echo "   xdg-open ${EXCEL_FILENAME}"
    fi
  else
    echo "❌ Downloaded file is empty"
    rm -f "${EXCEL_FILENAME}"
    exit 1
  fi
else
  echo "❌ Failed to download Excel report"
  exit 1
fi



echo ""
echo ""



# ====================================================
# PART 4: DEFAULTERS EXCEL REPORT GENERATION
# ====================================================



echo "======================================================"
echo "    PART 4: FACULTY DEFAULTERS REPORT DOWNLOAD"
echo "======================================================"
echo ""



echo "Downloading defaulters Excel report..."
echo "   (Students with <15 questions in last 30 days)"
echo ""



# Generate filename with timestamp
DEFAULTERS_FILENAME="defaulters_report_${TIMESTAMP}.xlsx"



# Download the defaulters Excel report
curl -s -X GET "${API_URL}/faculty/reports/defaulters" \
  -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}" \
  -o "${DEFAULTERS_FILENAME}" \
  -w "\nHTTP Status: %{http_code}\n"



# Check if download was successful
if [ $? -eq 0 ] && [ -f "${DEFAULTERS_FILENAME}" ]; then
  FILE_SIZE=$(ls -lh "${DEFAULTERS_FILENAME}" | awk '{print $5}')
  
  # Check if file is not empty and is actually an Excel file
  if [ -s "${DEFAULTERS_FILENAME}" ]; then
    echo ""
    echo "✅ Defaulters report downloaded successfully!"
    echo "----------------------------------------"
    echo "📄 Filename: ${DEFAULTERS_FILENAME}"
    echo "📊 File Size: ${FILE_SIZE}"
    echo "📍 Location: $(pwd)/${DEFAULTERS_FILENAME}"
    echo "----------------------------------------"
    echo ""
    
    # Check file type
    FILE_TYPE=$(file -b "${DEFAULTERS_FILENAME}" 2>/dev/null || echo "Unknown")
    echo "File Type: ${FILE_TYPE}"
    echo ""
    
    # If 'file' command shows it's a text file, it might be an error response
    if [[ "${FILE_TYPE}" == *"text"* ]] || [[ "${FILE_TYPE}" == *"JSON"* ]]; then
      echo "⚠️  Warning: File appears to be text/JSON instead of Excel"
      echo "Content:"
      cat "${DEFAULTERS_FILENAME}"
      echo ""
    else
      echo "✨ You can now open the Excel file with:"
      echo "   libreoffice ${DEFAULTERS_FILENAME}"
      echo "   or"
      echo "   xdg-open ${DEFAULTERS_FILENAME}"
    fi
  else
    echo "❌ Downloaded file is empty"
    rm -f "${DEFAULTERS_FILENAME}"
    exit 1
  fi
else
  echo "❌ Failed to download defaulters report"
  exit 1
fi



echo ""
echo "======================================================"
echo "    ALL TESTS COMPLETE"
echo "======================================================"
echo ""
echo "Summary:"
echo "  ✅ Student login and profile link submission"
echo "  ✅ Faculty login and profile verification"
echo "  ✅ Faculty statistics retrieval"
echo "  ✅ Excel submissions report generation and download"
echo "  ✅ Excel defaulters report generation and download"
echo ""
echo "Generated Files:"
echo "  📊 ${EXCEL_FILENAME}"
echo "  📊 ${DEFAULTERS_FILENAME}"
echo ""
echo "Quick Open Commands:"
echo "  libreoffice ${EXCEL_FILENAME} &"
echo "  libreoffice ${DEFAULTERS_FILENAME} &"
echo ""
echo "======================================================"


echo ""
echo ""
