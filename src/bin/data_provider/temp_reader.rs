use std::path::Path;
use tokio::fs;

static T1_PATH: &str = "/sys/bus/w1/devices/28-00000bc8f129/w1_slave";
static _T2_PATH: &str = "/sys/bus/w1/devices/28-00000bc8f129/w1_slave";

pub struct TempReader {

}


impl TempReader {

    pub async fn get_temps() -> Result<i32,String> {
        Self::read_temp(T1_PATH).await
    }

    async fn read_temp(path: &str) -> Result<i32,String> {
        if Path::new(path).exists() {
            let contents = fs::read_to_string(path).await.unwrap();
            let index = match contents.rfind("t="){
                Some(v) => v,
                None => return Err("Cant find t= token".to_string()),
            };
            let temp_str = contents[(index+2)..].trim().to_string();
            match temp_str.parse::<i32>() {
                Ok(v) => Ok(v),
                Err(err) => Err(format!("{}/{}/{} -> {}",contents,index, temp_str, err.to_string()))
            }
        } else {
            Err("sensor read err".to_string())
        }
    }
}