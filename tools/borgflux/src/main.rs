use dotenvy::{self, dotenv};
use reqwest::header::AUTHORIZATION;
use serde_json::Value;
use std::env;
use std::error::Error;
use std::fmt;
use std::process::Command;
use std::time::SystemTime;

struct BorgFluxEnv {
    url: String,
    token: String,
    org: String,
    bucket: String,
    host: String,
    repository: String,
    source_path: String,
}

struct InfluxTag {
    name: String,
    value: String,
}

struct InfluxField {
    name: String,
    value: InfluxFieldValue,
}

enum InfluxFieldValue {
    Float(f64),
    Int(i64),
    String(String),
}

impl fmt::Display for InfluxFieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Float(v) => write!(f, "{}", v),
            Self::Int(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
        }
    }
}

struct Backup {
    host: String,
    backup_name: String,
    encryption: String,
    repo_location: String,
    duration: f64,
    compressed_size: i64,
    deduplicated_size: i64,
    number_of_files: i64,
    original_size: i64,
}

struct InfluxPoint {
    measurement: String,
    tags: Vec<InfluxTag>,
    fields: Vec<InfluxField>,
}

fn main() {
    let credentials = read_env_file();

    send_frame_point(&credentials, "backup_start".to_string());

    let json_output = run_borg_backup(&credentials.repository, &credentials.source_path);

    let json_value = read_borg_json_output(json_output);

    if json_value.is_err() {
        panic!("Couldn't read the JSON file correctly!");
    }

    let backup_stats = extract_data_from_json(json_value.unwrap(), credentials.host.to_owned());
    let influx_backup_point = create_influx_point_from_backup(backup_stats);

    write_to_influx(influx_backup_point, &credentials);

    // TODO: write backup Influx point

    send_frame_point(&credentials, "backup_end".to_string());
}

fn read_env_file() -> BorgFluxEnv {
    dotenv().ok();

    BorgFluxEnv {
        url: env::var("INFLUX_URL").expect("INFLUX_URL missing"),
        token: env::var("INFLUX_TOKEN").expect("INFLUX_TOKEN missing"),
        org: env::var("INFLUX_ORG").expect("INFLUX_ORG missing"),
        bucket: env::var("INFLUX_BUCKET").expect("INFLUX_BUCKET missing"),
        host: env::var("HOST").expect("HOST missing"),
        repository: env::var("BORG_REPOSITORY").expect("BORG_REPOSITORY missing"),
        source_path: env::var("BORG_SOURCE_PATH").expect("BORG_SOURCE_PATH missing"),
    }
}

fn read_borg_json_output(json_output: String) -> Result<Value, Box<dyn Error>> {
    let json_value: Value = serde_json::from_str(&json_output)?;

    return Ok(json_value);
}

fn extract_data_from_json(json_data: Value, host: String) -> Backup {
    Backup {
        host,
        backup_name: json_data["archive"]["name"].to_string(),
        encryption: json_data["encryption"]["mode"].to_string(),
        repo_location: json_data["repository"]["location"].to_string(),
        duration: json_data["archive"]["duration"].as_f64().unwrap(),
        compressed_size: json_data["archive"]["stats"]["compressed_size"]
            .as_i64()
            .unwrap(),
        deduplicated_size: json_data["archive"]["stats"]["deduplicated_size"]
            .as_i64()
            .unwrap(),
        number_of_files: json_data["archive"]["stats"]["nfiles"].as_i64().unwrap(),
        original_size: json_data["archive"]["stats"]["original_size"]
            .as_i64()
            .unwrap(),
    }
}

fn create_influx_point_from_backup(backup: Backup) -> InfluxPoint {
    let tags: Vec<InfluxTag> = vec![
        InfluxTag {
            name: "host".to_string(),
            value: backup.host,
        },
        InfluxTag {
            name: "backup_name".to_string(),
            value: backup.backup_name,
        },
        InfluxTag {
            name: "encryption".to_string(),
            value: backup.encryption,
        },
        InfluxTag {
            name: "repo_location".to_string(),
            value: backup.repo_location,
        },
    ];
    let fields: Vec<InfluxField> = vec![
        InfluxField {
            name: "duration".to_string(),
            value: InfluxFieldValue::Float(backup.duration),
        },
        InfluxField {
            name: "compressed_size".to_string(),
            value: InfluxFieldValue::Int(backup.compressed_size),
        },
        InfluxField {
            name: "deduplicated_size".to_string(),
            value: InfluxFieldValue::Int(backup.deduplicated_size),
        },
        InfluxField {
            name: "number_of_files".to_string(),
            value: InfluxFieldValue::Int(backup.number_of_files),
        },
        InfluxField {
            name: "original_size".to_string(),
            value: InfluxFieldValue::Int(backup.original_size),
        },
    ];

    InfluxPoint {
        measurement: "backup_data".to_string(),
        tags,
        fields,
    }
}

fn send_frame_point(credentials: &BorgFluxEnv, measurement: String) {
    let tags: Vec<InfluxTag> = vec![
        InfluxTag {
            name: "host".to_string(),
            value: credentials.host.to_owned(),
        },
        InfluxTag {
            name: "repository".to_string(),
            value: credentials.repository.to_owned(),
        },
        InfluxTag {
            name: "source_path".to_string(),
            value: credentials.source_path.to_owned(),
        },
    ];

    let fields: Vec<InfluxField> = vec![InfluxField {
        name: "dummy".to_string(),
        value: InfluxFieldValue::Int(0),
    }];

    let point = InfluxPoint {
        measurement,
        tags,
        fields,
    };

    write_to_influx(point, credentials);
}

fn build_raw_data_from_point(point: InfluxPoint) -> String {
    let mut raw_data = point.measurement;

    for tag in point.tags {
        raw_data = format!("{},{}={}", raw_data, tag.name, tag.value);
    }

    if point.fields.len() != 0 {
        raw_data = format!("{} ", raw_data);
    }

    let mut counter = 1;
    let number_of_fields = point.fields.len().to_owned();

    for field in point.fields {
        raw_data = format!("{}{}={}", raw_data, field.name, field.value);

        if counter < number_of_fields {
            raw_data = format!("{},", raw_data);
        }

        counter += 1;
    }

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    format!("{} {}", raw_data, timestamp)
}

fn write_to_influx(point: InfluxPoint, credentials: &BorgFluxEnv) {
    let client = reqwest::blocking::Client::new();
    let body = build_raw_data_from_point(point);
    let api_url = format!(
        "{}/api/v2/write?bucket={}&org={}&precision=s",
        credentials.url.to_owned(),
        credentials.bucket.to_owned(),
        credentials.org.to_owned()
    );

    let result = client
        .post(api_url)
        .body(body)
        .header(
            AUTHORIZATION,
            "Token ".to_owned() + &credentials.token.to_owned(),
        )
        .send();

    if result.is_err() {
        panic!("Unable to send data to InfluxDB!");
    }
}

fn run_borg_backup(repository: &String, source_path: &String) -> String {
    let is_borg_installed = Command::new("which").arg("borg").status();

    if is_borg_installed.is_err() || !is_borg_installed.unwrap().success() {
        panic!("BorgBackup is not installed!");
    }

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    let backup_repo = format!("{}::{}", repository.to_owned(), timestamp);

    let output = Command::new("borg")
        .arg("create")
        .arg("-v")
        .arg("--json")
        .arg(backup_repo)
        .arg(source_path)
        .output()
        .expect("Something went wrong while running the backup!");

    String::from_utf8_lossy(&output.stdout).to_string()
}
