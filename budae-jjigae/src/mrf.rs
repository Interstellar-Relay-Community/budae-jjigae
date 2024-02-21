use sonic_rs::{pointer, JsonValueTrait, Value};

use mrf_policy::FilterConfig;

pub async fn filter(object: &Value, filter_config: &FilterConfig) -> Result<(), ()> {
    let filter_result = mrf_policy::filter(object, filter_config).await;

    match filter_result {
        Ok(_) => {},
        Err(idx) => {
            // Extract ID
            let object_id_raw = object.pointer(pointer!["id"]);
            let object_id = object_id_raw.as_str().unwrap_or("Unknown ID");
            let object_uri_raw = object.pointer(pointer!["uri"]);
            let object_uri = object_uri_raw.as_str().unwrap_or(object_id);
            let content_str_raw = object.pointer(pointer!["content"]);
            let content_str = content_str_raw.as_str().unwrap_or("Unknown content!");
            tracing::info!(
                    "Spam killed - filter #{}: {} => {}",
                    idx,
                    object_uri,
                    content_str
                );
            return Err(());
        }
    }

    Ok(())
}
