use std::fs::File;
use ureq::Agent;
use std::fs;
use std::path::Path;
use std::time::Duration;
use std::io;
use serde::{Serialize, Deserialize};
use crate::helpers::take_user_choice;

const 

fn get_horizons_data(body_name: String) -> io::Result<(String)> {

    let horizons_data_file = "target/mars_data.json";
    let choice = if Path::new(horizons_data_file).exists() {
        let user_choice = take_user_choice("Get new astro data?");
        user_choice
    } else {
        let forced_choice = false;
        println!("Generating new astrodata");
        forced_choice
    };


    let data = if !choice {
            println!("Using cached file");
            let data = fs::read_to_string(horizons_data_file).map_err(io::Error::other)?;


            data
    } else {
        let ureq_agent_config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build();
        let ureq_agent: Agent = ureq_agent_config.into();
        let http_url = "https://ssd.jpl.nasa.gov/api/horizons.api?format=json&COMMAND='499'\
            &OBJ_DATA='YES'&MAKE_EPHEM='YES'&EPHEM_TYPE='VECTORS'&CENTER='500@10'&START_TIME='2006-01-01'\
            &STOP_TIME='2006-01-2'&STEP_SIZE='1%20d'&QUANTITIES='1,9,20'".to_string();
        let json_output: String = ureq_agent.get(&http_url).call().map_err(io::Error::other)?
            .body_mut().read_to_string().map_err(io::Error::other)?;

        let parsed_data: serde_json::Value = serde_json::from_str(&json_output).map_err(io::Error::other)?;
        let data_result = get_data_result(body_name, parsed_data)?;
        fs::write(horizons_data_file, &data_result).map_err(io::Error::other)?;



        data_result
    };


    Ok(data)
}

fn get_data_result(body: String, parsed_data: serde_json::Value) -> io::Result<String> {


    let data_result = if let Some(result_data) = parsed_data["result"].as_str(){
        result_data.to_string()
    } else {
        serde_json::to_string_pretty(&parsed_data).map_err(io::Error::other)?
    };
    println!("{}", data_result);
    Ok(data_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizons_data() {
        let result = get_horizons_data("mars".to_string());
        println!("{:?}", result);
    }

}
