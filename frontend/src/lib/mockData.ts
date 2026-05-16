// Mock data for the LeetCode Progress Monitoring System

export interface Student {
  registerNumber: string;
  name: string;
  department: string;
  year: 1 | 2 | 3 | 4;
  hasLeetCodeProfile: boolean;
  totalProblemsSolved: number;
  previousMonthTotal: number;
  lastUpdated: string;
}

export interface Department {
  id: string;
  name: string;
  shortName: string;
}

export const departments: Department[] = [
  { id: "placement", name: "Placement cell", shortName: "Placement Cell" },
  { id: "ai", name: "Artificial Intelligence", shortName: "AI" },
  { id: "bme", name: "Biomedical Engineering", shortName: "BME" },
  { id: "bt", name: "Biotechnology", shortName: "Biotechnology" },
  { id: "ce", name: "Civil Engineering", shortName: "Civil" },
  { id: "csbs", name: "Computer Science and Business System", shortName: "CSE-CSBS" },
  { id: "cse", name: "Computer Science and Engineering", shortName: "CSE" },
  {
    id: "cse-aiml",
    name: "Computer Science and Engineering with specialization in Artificial Intelligence and Machine Learning",
    shortName: "CSE-AIML",
  },
  { id: "cse-bda", name: "Computer Science and Engineering with specialization in Big Data Analytics", shortName: "CSE-BDA" },
  { id: "cse-cc", name: "Computer Science and Engineering with specialization in Cloud Computing", shortName: "CSE-CC" },
  { id: "cse-cys", name: "Computer Science and Engineering with specialization in Cyber Security", shortName: "CSE-CS" },
  { id: "cse-gt", name: "Computer Science and Engineering with specialization in Gaming Technology", shortName: "CSE-GT" },
  { id: "cse-iot", name: "Computer Science and Engineering with specialization in Internet of Things", shortName: "CSE-IoT" },
  { id: "eee", name: "Electrical and Electronics Engineering", shortName: "EEE" },
  { id: "ece", name: "Electronics and Communication Engineering", shortName: "ECE" },
  { id: "ece-ds", name: "Electronics and Communication Engineering with specialization in Data Science", shortName: "ECE-DS" },
  { id: "it", name: "Information Technology", shortName: "IT" },
  { id: "mech", name: "Mechanical Engineering", shortName: "MECH" },
];

const firstNames = [
  "Arun", "Priya", "Karthik", "Divya", "Rajesh", "Sneha", "Vikram", "Ananya",
  "Suresh", "Kavitha", "Harish", "Meera", "Ganesh", "Lakshmi", "Sanjay", "Deepa",
  "Venkat", "Pooja", "Rahul", "Swathi", "Naveen", "Revathi", "Prasad", "Nithya",
  "Ramesh", "Sowmya", "Arvind", "Bhavani", "Mohan", "Sangeetha"
];

const lastNames = [
  "Kumar", "Sharma", "Reddy", "Nair", "Iyer", "Pillai", "Rao", "Menon",
  "Patel", "Singh", "Gupta", "Joshi", "Verma", "Das", "Mukherjee", "Chatterjee"
];

function generateRegisterNumber(year: number, dept: string, index: number): string {
  const yearCode = 24 - (year - 1);
  const deptCode = dept.toUpperCase().slice(0, 3);
  const studentNum = String(index + 1).padStart(3, "0");
  return `${yearCode}${deptCode}${studentNum}`;
}

function generateRandomName(): string {
  const firstName = firstNames[Math.floor(Math.random() * firstNames.length)];
  const lastName = lastNames[Math.floor(Math.random() * lastNames.length)];
  return `${firstName} ${lastName}`;
}

function getYearFromRegister(regNum: string): 1 | 2 | 3 | 4 {
  const yearCode = parseInt(regNum.slice(0, 2));
  const currentYear = 24;
  const diff = currentYear - yearCode + 1;
  return Math.min(4, Math.max(1, diff)) as 1 | 2 | 3 | 4;
}

export function generateMockStudents(): Student[] {
  const students: Student[] = [];
  const currentDate = new Date();
  const currentMonth = currentDate.getMonth();
  
  departments.forEach((dept) => {
    for (let year = 1; year <= 4; year++) {
      const studentsPerYear = 25 + Math.floor(Math.random() * 15);
      
      for (let i = 0; i < studentsPerYear; i++) {
        const regNum = generateRegisterNumber(year, dept.shortName, i);
        const hasProfile = Math.random() > 0.15; // 85% have profiles
        
        // Calculate cumulative target based on months
        const monthsActive = currentMonth + 1;
        const monthlyTarget = 30;
        const cumulativeTarget = monthsActive * monthlyTarget;
        
        // Generate realistic solving patterns
        let totalSolved = 0;
        let prevMonthTotal = 0;
        
        if (hasProfile) {
          // Some students ahead, some behind
          const performanceMultiplier = 0.3 + Math.random() * 1.5;
          totalSolved = Math.floor(cumulativeTarget * performanceMultiplier);
          prevMonthTotal = Math.max(0, totalSolved - Math.floor(Math.random() * 45));
        }
        
        students.push({
          registerNumber: regNum,
          name: generateRandomName(),
          department: dept.shortName,
          year: year as 1 | 2 | 3 | 4,
          hasLeetCodeProfile: hasProfile,
          totalProblemsSolved: totalSolved,
          previousMonthTotal: prevMonthTotal,
          lastUpdated: new Date(Date.now() - Math.random() * 86400000).toISOString(),
        });
      }
    }
  });
  
  return students;
}

export const mockStudents = generateMockStudents();

export function getStudentsByYear(year: 1 | 2 | 3 | 4): Student[] {
  return mockStudents
    .filter((s) => s.year === year && s.hasLeetCodeProfile)
    .sort((a, b) => b.totalProblemsSolved - a.totalProblemsSolved);
}

export function getStudentsByDepartment(dept: string): Student[] {
  return mockStudents.filter((s) => s.department === dept);
}

export function getStudentsWithoutProfile(dept?: string): Student[] {
  return mockStudents.filter((s) => !s.hasLeetCodeProfile && (!dept || s.department === dept));
}

export function getMonthlyTarget(): number {
  const currentMonth = new Date().getMonth() + 1;
  return currentMonth * 30;
}

export function getDefaulters(dept?: string): Student[] {
  const target = getMonthlyTarget();
  return mockStudents.filter(
    (s) => s.hasLeetCodeProfile && s.totalProblemsSolved < target && (!dept || s.department === dept)
  );
}

export const syncInfo = {
  lastSync: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(), // 2 hours ago
  nextSync: new Date(Date.now() + 22 * 60 * 60 * 1000).toISOString(), // 22 hours from now
};
