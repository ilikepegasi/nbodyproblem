use std::fs::File;
use ureq::Agent;
use std::fs;
use std::path::Path;
use std::time::Duration;
use std::io;
fn get_horizons_data() -> std::io::Result<()> {
    let horizons_data_file = "target/mars_data.json";


    let data = if Path::new(horizons_data_file).exists() {
        println!("Using cached file");
        let data = fs::read_to_string(horizons_data_file).map_err(std::io::Error::other)?;
        data
    } else {
        let ureq_agent_config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build();
        let ureq_agent: Agent = ureq_agent_config.into();
        let http_url = "https://ssd.jpl.nasa.gov/api/horizons.api?format=json&COMMAND='499'\
            &OBJ_DATA='YES'&MAKE_EPHEM='YES'&EPHEM_TYPE='VECTORS'&CENTER='500@10'&START_TIME='2006-01-01'\
            &STOP_TIME='2006-01-20'&STEP_SIZE='1%20d'&QUANTITIES='1,9,20'".to_string();
        let json_output: String = ureq_agent.get(&http_url).call().map_err(std::io::Error::other)?
            .body_mut().read_to_string().map_err(std::io::Error::other)?;
        fs::write(horizons_data_file, &json_output)?;
        let data = fs::read_to_string(horizons_data_file).map_err(std::io::Error::other)?;
        let parsed_data: serde_json::Value = serde_json::from_str(&data).map_err(std::io::Error::other)?;
        let data_pretty = serde_json::to_string_pretty(&parsed_data).map_err(std::io::Error::other)?;
        data_pretty
    };
    println!("{}", data);

    Ok(())
}

