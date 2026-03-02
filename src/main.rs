use std::fs::File;
use ureq::get;

use std::fs;
use std::path::Path;


fn main() {
    let horizons_data_file = "target/mars_data.json";


    if Path::new(horizons_data_file).exists() {
        println!("Using cached file")
        let data = fs::read_to_string(horizons_data_file).unwrap();
    } else {
        let http_url = "https://ssd.jpl.nasa.gov/api/horizons.api?format=json&COMMAND='499'\
            &OBJ_DATA='YES'&MAKE_EPHEM='YES'&EPHEM_TYPE='VECTORS'&CENTER='500@10'&START_TIME='2006-01-01'\
            &STOP_TIME='2006-01-20'&STEP_SIZE='1%1d'QUANTITIES='1,9,20'".to_string();
    }
    let http_url = "https://ssd.jpl.nasa.gov/api/horizons.api?format=json&COMMAND='499'\
    &OBJ_DATA='YES'&MAKE_EPHEM='YES'&EPHEM_TYPE='VECTORS'&CENTER='500@10'&START_TIME='2006-01-01'\
    &STOP_TIME='2006-01-20'&STEP_SIZE='1%1d'QUANTITIES='1,9,20'".to_string();
    let json_output = get(&http_url);

}

