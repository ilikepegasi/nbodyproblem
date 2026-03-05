use crate::helpers::take_user_choice;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::repeat;
use std::path::Path;
use std::time::Duration;
use ureq::Agent;
use macroquad::prelude::*;
use macroquad::color::Color;
use macroquad::miniquad::native::apple::apple_util::yes;
use phf;

static BODY_MASS_KG: phf::Map<&str, f64> = phf::phf_map! {
    "mercury" => 3.301e+23,
    "venus"   => 4.867e+24,
    "earth"   => 5.972e+24,
    "mars"    => 6.417e+23,
    "jupiter" => 1.898e+27,
    "saturn"  => 5.683e+26,
    "uranus"  => 8.681e+25,
    "neptune" => 1.024e+26,
    "luna"    => 7.342e+22,
    "sun"     => 1.989e+30,
};

static BODY_RADIUS_M: phf::Map<&str, f64> = phf::phf_map! {
    "mercury" => 2.439e+06,
    "venus"   => 6.051e+06,
    "earth"   => 6.371e+06,
    "mars"    => 3.389e+06,
    "jupiter" => 6.991e+07,
    "saturn"  => 5.823e+07,
    "uranus"  => 2.536e+07,
    "neptune" => 2.462e+07,
    "luna"    => 1.737e+06,
    "sun"     => 6.957e+08,
};
static HORIZONS_IDS: phf::Map<&str, u32> = phf::phf_map! {
    "mercury"   => 199,
    "venus"     => 299,
    "earth"     => 399,
    "mars"      => 499,
    "jupiter"   => 599,
    "saturn"    => 699,
    "uranus"    => 799,
    "neptune"   => 899,
    "luna"      => 301,
    "sun"       => 10,
};
static HORIZONS_COLORS: phf::Map<&str, Color> = phf::phf_map! {
    "mercury"   => GRAY,
    "venus"     => YELLOW,
    "earth"     => BLUE,
    "mars"      => RED,
    "jupiter"   => ORANGE,
    "saturn"    => YELLOW,
    "uranus"    => SKYBLUE,
    "neptune"   => BLUE,
    "luna"      => LIGHTGRAY,
    "sun"       => YELLOW,
};

static MAJOR_BODIES: [&str; 10] = ["mercury", "sun", "venus", "earth", "mars", "jupiter", "saturn", "uranus", "neptune", "luna"];



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
pub struct output_values {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
}


pub fn get_horizons_data() -> Vec<output_values> {
    let cache_choice = !take_user_choice("Get new data? ");

    let times = date_time_range();
    let mut body_values: Vec<output_values> = Vec::new();
    let mut i = 0;
    for body in MAJOR_BODIES.iter() {
        let horizons_data_file = format!("target/{}_data.txt", body);

        let use_cached_file = if Path::new(horizons_data_file.as_str()).exists() {
            cache_choice
        } else {
            let forced_choice = false;
            println!("Generating new astrodata for {}", body);
            forced_choice
        };


        let horizons_data = fetch_horizons_data(body.to_string(), use_cached_file, &times);
        match horizons_data {
            Ok(s) => {
                body_values[i] = parse_horizons_body_data(s, body.to_string());
            },
            Err(e) => eprintln!("Error: {}", e),
        }
        i += 1;
    }
    body_values

}

fn parse_horizons_body_data(body_result: String, body_name: String) -> output_values {
    let soe = body_result.find("$$SOE").expect("Could not find '$$SOE'");
    let eoe = body_result.find("$$EOE").expect("Could not find '$$EOE'");

    let body_ephemeris = &body_result[5+soe..eoe].trim();
    let ephemeris_lines: Vec<&str> = body_ephemeris.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();


    let mut body_values: output_values = output_values {
        name: body_name,
        x: 0.0,
        y: 0.0,
        vx: 0.0,
        vy: 0.0,

    };
    let data_start = ephemeris_lines.len()/2;
    let data_end = ephemeris_lines.len();
    for data_line_id in data_start..data_end {
        if data_line_id == 1  {
            body_values.x = parse_data_component("X", ephemeris_lines[data_line_id]);
            body_values.y = parse_data_component("Y", ephemeris_lines[data_line_id]);

        } else if data_line_id == 2 {
            body_values.vx = parse_data_component("VX", ephemeris_lines[data_line_id]);
            body_values.vy = parse_data_component("VY", ephemeris_lines[data_line_id]);
        }
    }

    body_values

}

fn parse_data_component(req_value: &str, line: &str) -> f64 {
    let value_index: usize = line.find(req_value).expect(&format!("Could not find '{}'", req_value));


}



fn date_time_range() -> (String, String) {
    let now = Utc::now();
    let yesterday = now - chrono::Duration::days(1);
    (yesterday.date_naive().to_string(), now.date_naive().to_string(),)
}

fn fetch_horizons_data(body_name: String, cache_choice: bool, times: &(String, String)) -> io::Result<(String)> {
    let body_id = HORIZONS_IDS[body_name.as_str()];

    let horizons_data_file = format!("target/{}_data.txt", body_name);


    let data = if cache_choice {
        println!("Using cached file");
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
        let data_result = get_data_result(body_name, parsed_data)?;
        fs::write(horizons_data_file, &data_result).map_err(io::Error::other)?;

        data_result
    };

    Ok(data)
}

fn get_data_result(body: String, parsed_data: serde_json::Value) -> io::Result<String> {
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
}
