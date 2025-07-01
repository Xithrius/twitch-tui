use serde::de::DeserializeOwned;

pub fn load_data<T>(raw_data: &str) -> Result<(serde_json::Value, T), serde_json::Error>
where
    T: DeserializeOwned,
{
    let raw: serde_json::Value = serde_json::from_str(raw_data)?;
    let message: T = serde_json::from_str(raw_data)?;

    Ok((raw, message))
}
