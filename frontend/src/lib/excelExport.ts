import * as XLSX from "xlsx";
import type { Student } from "./mockData";

export function exportToExcel(
  data: Record<string, unknown>[],
  filename: string,
  sheetName: string = "Sheet1"
): void {
  const worksheet = XLSX.utils.json_to_sheet(data);
  const workbook = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(workbook, worksheet, sheetName);
  XLSX.writeFile(workbook, `${filename}.xlsx`);
}

export function exportStudentsWithoutProfile(students: Student[]): void {
  const data = students.map((s) => ({
    "Register Number": s.registerNumber,
    "Student Name": s.name,
    "Department": s.department,
    "Year": `Year ${s.year}`,
  }));
  
  exportToExcel(data, "students_without_leetcode_profile", "No Profile");
}

export function exportMonthlyProgress(students: Student[], monthlyTarget: number): void {
  const data = students.map((s) => {
    const remaining = Math.max(0, monthlyTarget - s.totalProblemsSolved);
    const status = s.totalProblemsSolved >= monthlyTarget ? "Completed" : "Not Completed";
    const progressPercent = Math.min(100, (s.totalProblemsSolved / monthlyTarget) * 100);
    
    return {
      "Register Number": s.registerNumber,
      "Student Name": s.name,
      "Department": s.department,
      "Year": `Year ${s.year}`,
      "Total Problems Solved": s.totalProblemsSolved,
      "Monthly Target": monthlyTarget,
      "Remaining": remaining,
      "Status": status,
      "Progress %": progressPercent.toFixed(1),
    };
  });
  
  exportToExcel(data, "monthly_progress_report", "Progress");
}

export function exportDefaulters(students: Student[], monthlyTarget: number): void {
  const data = students.map((s) => {
    const deficit = monthlyTarget - s.totalProblemsSolved;
    
    return {
      "Register Number": s.registerNumber,
      "Student Name": s.name,
      "Department": s.department,
      "Year": `Year ${s.year}`,
      "Total Solved": s.totalProblemsSolved,
      "Target": monthlyTarget,
      "Deficit": deficit,
    };
  });
  
  exportToExcel(data, "monthly_defaulters", "Defaulters");
}
