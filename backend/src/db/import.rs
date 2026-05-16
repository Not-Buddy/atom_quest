use sqlx::MySqlPool;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct CsvStudentRow {
    serial_number: i32,
    registration_number: String,
    full_name: String,
    college: String,
    course: String,
    specialization: String,
    academic_year: String,
    email: String,
}

pub async fn import_students_from_data_dir(pool: &MySqlPool) -> Result<u64, Box<dyn std::error::Error>> {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return Err("data/ directory not found".into());
    }

    // Generate a default password hash for imported students
    // This forces users to reset password on first login
    let default_password = "ChangeMe123!"; // Temporary default
    let default_password_hash = bcrypt::hash(default_password, bcrypt::DEFAULT_COST)?;

    let mut total_inserted = 0u64;

    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = path.extension() {
            if ext != "csv" {
                continue;
            }
        } else {
            continue;
        }

        let _file_path = path.to_string_lossy().to_string();
        let file = fs::File::open(&path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        for result in rdr.records() {
            let record = result?;
            if record.len() < 8 {
                continue;
            }

            let row = CsvStudentRow {
                serial_number: parse_i32(record.get(0)),
                registration_number: clean_str(record.get(1)),
                full_name: clean_str(record.get(2)),
                college: clean_str(record.get(3)),
                course: clean_str(record.get(4)),
                specialization: clean_str(record.get(5)),
                academic_year: clean_str(record.get(6)),
                email: clean_email(record.get(7)),
            };

            if row.registration_number.is_empty() || row.full_name.is_empty() {
                continue;
            }

            let res = sqlx::query!(
                r#"
                INSERT INTO STUDENTS (
                    serial_number,
                    registration_number,
                    full_name,
                    college,
                    course,
                    specialization,
                    academic_year,
                    email,
                    password
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    full_name = VALUES(full_name),
                    college = VALUES(college),
                    course = VALUES(course),
                    specialization = VALUES(specialization),
                    academic_year = VALUES(academic_year),
                    email = VALUES(email)
                "#,
                row.serial_number,
                row.registration_number,
                row.full_name,
                row.college,
                row.course,
                row.specialization,
                row.academic_year,
                row.email,
                default_password_hash, // Added password field
            )
            .execute(pool)
            .await?;

            total_inserted += res.rows_affected();
        }
    }

    Ok(total_inserted)
}

fn clean_str(input: Option<&str>) -> String {
    input
        .unwrap_or("")
        .replace('\u{00A0}', " ")
        .trim()
        .to_string()
}

fn clean_email(input: Option<&str>) -> String {
    let mut s = clean_str(input);
    if let Some(start) = s.find("mailto:") {
        let sub = &s[start + "mailto:".len()..];
        if let Some(end) = sub.find(')') {
            s = sub[..end].to_string();
        } else {
            s = sub.to_string();
        }
    }
    s
}

fn parse_i32(input: Option<&str>) -> i32 {
    clean_str(input).parse::<i32>().unwrap_or(0)
}
