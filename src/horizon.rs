use crate::helpers::take_user_choice;
use crate::horizons_table::*;
use chrono::prelude::*;
use macroquad::color::Color;
use macroquad::prelude::*;
use phf;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use std::ptr::eq;
use std::time::Duration;
use ureq::Agent;

/*
$$SOE
2453736.500000000 = A.D. 2006-Jan-01 00:00:00.0000 TDB
 X = 6.108336946835414E+07 Y = 2.207576654727506E+08 Z = 3.124955669833437E+06
 VX=-2.243445381356987E+01 VY= 8.522324624760257E+00 VZ= 7.296978814338950E-01
 LT= 7.641085627402825E+02 RG= 2.290739842027565E+08 RR= 2.240659203471744E+00
2453737.500000000 = A.D. 2006-Jan-02 00:00:00.0000 TDB
 X = 5.914254439884686E+07 Y = 2.214848947595732E+08 Z = 3.187872023810804E+06
 VX=-2.249172010691555E+01 VY= 8.311660149666960E+00 VZ= 7.266905055652093E-01
 LT= 7.647538810867583E+02 RG= 2.292674457760389E+08 RR= 2.237588564276273E+00
$$EOE
 */

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputValues {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
}

pub fn get_horizons_data() -> Vec<OutputValues> {
    let times = date_time_range();
    let mut body_values: Vec<OutputValues> = Vec::new();

    let mut horizon_data_files_names = Vec::new();
    let can_use_cached_file = MAJOR_BODIES.iter().all(|body| {
        let horizons_data_file = format!("target/cache/{}_data.txt", body);
        horizon_data_files_names.push(horizons_data_file.clone());
        Path::new(&horizons_data_file).exists()
    });

    let cache_choice: bool = if can_use_cached_file {
        if Path::new("target/cache/CacheInfo.txt").exists() {
            println!(
                "Cache Info: {}",
                fs::read_to_string("target/cache/CacheInfo.txt").unwrap()
            );
        }
        !take_user_choice("Get new data? ")
    } else {
        false
    };
    if !can_use_cached_file {
        println!("Incomplete cache, retrieving new data");
    }

    for body in MAJOR_BODIES.iter() {
        let horizons_data = fetch_horizons_data(body.to_string(), cache_choice, &times);
        match horizons_data {
            Ok(s) => {
                body_values.push(parse_horizons_body_data(s, body.to_string()));
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    body_values
}

fn parse_horizons_body_data(body_result: String, body_name: String) -> OutputValues {
    let soe = body_result.find("$$SOE").expect("Could not find '$$SOE'");
    let eoe = body_result.find("$$EOE").expect("Could not find '$$EOE'");

    let body_ephemeris = &body_result[5 + soe..eoe].trim();
    let ephemeris_lines: Vec<&str> = body_ephemeris
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    let mut body_values: OutputValues = OutputValues {
        name: body_name,
        x: 0.0,
        y: 0.0,
        vx: 0.0,
        vy: 0.0,
    };
    let n = ephemeris_lines.len();
    body_values.x  = parse_data_component("X",  ephemeris_lines[n - 2]);
    body_values.y  = parse_data_component("Y",  ephemeris_lines[n - 2]);
    body_values.vx = parse_data_component("VX", ephemeris_lines[n - 1]);
    body_values.vy = parse_data_component("VY", ephemeris_lines[n - 1]);


    body_values
}

fn parse_data_component(req_value: &str, line: &str) -> f64 {
    let value_index_start: usize = line
        .find(req_value)
        .expect(&format!("Could not find '{}'", req_value))
        + 3;
    let value_index_end = value_index_start + 22;
    line[value_index_start..value_index_end]
        .to_string()
        .trim()
        .parse::<f64>()
        .unwrap()
}

fn date_time_range() -> (String, String) {
    let now = Utc::now();
    let start = now - chrono::Duration::days(2);
    let stop = now - chrono::Duration::days(1);
    (
        start.date_naive().to_string(),
        stop.date_naive().to_string(),
    )
}

fn fetch_horizons_data(
    body_name: String,
    cache_choice: bool,
    times: &(String, String),
) -> io::Result<String> {
    fs::create_dir_all("target/cache").map_err(io::Error::other)?;
    let body_id = HORIZONS_IDS[body_name.as_str()];

    let horizons_data_file = format!("target/cache/{}_data.txt", body_name);

    let data = if cache_choice {
        let data = fs::read_to_string(horizons_data_file).map_err(io::Error::other)?;

        data
    } else {
        let ureq_agent_config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build();
        let ureq_agent: Agent = ureq_agent_config.into();
        let http_url = format!(
            "https://ssd.jpl.nasa.gov/api/horizons.api?format=json&COMMAND='{}'\
            &OBJ_DATA='YES'&MAKE_EPHEM='YES'&EPHEM_TYPE='VECTORS'&CENTER='500@10'&START_TIME='{}'\
            &STOP_TIME='{}'&STEP_SIZE='1%20d'&VEC_TABLE='2'",
            body_id, times.0, times.1
        );
        let json_output: String = ureq_agent
            .get(&http_url)
            .call()
            .map_err(io::Error::other)?
            .body_mut()
            .read_to_string()
            .map_err(io::Error::other)?;

        let parsed_data: serde_json::Value =
            serde_json::from_str(&json_output).map_err(io::Error::other)?;
        let data_result = get_data_result(parsed_data)?;
        fs::write(horizons_data_file, &data_result).map_err(io::Error::other)?;
        let time_text = format!("{}, {}", times.0, times.1);
        fs::write("target/cache/CacheInfo.txt", time_text).map_err(io::Error::other)?;
        data_result
    };

    Ok(data)
}

fn get_data_result(parsed_data: serde_json::Value) -> io::Result<String> {
    let data_result = if let Some(result_data) = parsed_data["result"].as_str() {
        result_data.to_string()
    } else {
        serde_json::to_string_pretty(&parsed_data).map_err(io::Error::other)?
    };
    Ok(data_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizons_data() {
        let result = fetch_horizons_data("mars".to_string(), true, &date_time_range());
        println!("{:?}", result);
    }
    #[test]
    fn test_date_time_range() {
        let result = date_time_range();
        println!("{:?}", result);
    }
    #[test]
    fn test_get_horizons_data() {
        let result = get_horizons_data();
        println!("{:?}", result);
    }

    #[test]
    fn test_parse_radius_debug() {
        // change body name to whichever is failing
        let result = fetch_horizons_data("mercury".to_string(), true, &date_time_range());
        if let Ok(text) = result {
            // print just the physical properties section
            let start = text.find("Physical").or_else(|| text.find("PHYSICAL"));
            if let Some(s) = start {
                println!("{}", &text[s..s + 500]);
            } else {
                println!("No physical section found, printing start:");
                println!("{}", &text[..500]);
            }
        }
    }

    #[test]
    fn test_parse_gm_all_bodies() {
        for body in MAJOR_BODIES.iter() {
            let result = fetch_horizons_data(body.to_string(), true, &date_time_range());
            if let Ok(text) = result {
                for line in text.lines() {
                    if line.to_lowercase().contains("gm") {
                        println!("{}: '{}'", body, line);
                    }
                }
            }
        }
    }
}
