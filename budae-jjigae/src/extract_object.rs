use sonic_rs::{pointer, JsonValueTrait, PointerNode, Value};

pub fn extract_obj<'a>(
    object: &'a Value,
    pointer: &'a mut Vec<PointerNode>,
    cnt: usize,
) -> Option<&'a Vec<PointerNode>> {
    let ptr: &Vec<PointerNode> = pointer;
    let inner_obj = object.pointer(ptr);

    if cnt > 10 {
        tracing::error!("Something went wrong!!! (Maybe potential DoS attempt?)");
        tracing::error!("Recursion depth > 10");
        tracing::error!("Please report to Interstellar Team!");
        tracing::error!("Object: {}", object.to_string());
        return None;
    }

    if let Some(obj_ref) = inner_obj.as_str() {
        tracing::warn!("Object reference support is not fully implemented!");
        tracing::warn!("If something goes wrong, please report to Interstellar Team!");
        tracing::warn!("Object reference: {}", obj_ref);
        return Some(pointer);
    }

    match inner_obj.pointer(pointer!["type"]).as_str() {
        Some("Announce") => {
            tracing::debug!("Extracting object from Announce");
            pointer.push(PointerNode::Key("object".into()));
            println!("{:?}", pointer);
            extract_obj(object, pointer, cnt + 1)
        }
        Some("Create") => {
            tracing::debug!("Extracting object from Create");
            pointer.push(PointerNode::Key("object".into()));
            extract_obj(object, pointer, cnt + 1)
        }
        Some("Update") => {
            tracing::debug!("Extracting object from Update");
            pointer.push(PointerNode::Key("object".into()));
            extract_obj(object, pointer, cnt + 1)
        }
        Some("Note") => Some(pointer),
        Some("Delete") => None,
        Some("Follow") => None,
        Some("Block") => None,
        Some("Undo") => None,
        Some("View") => None,
        Some("Add") => None,
        Some("Remove") => None,
        Some(x) => {
            tracing::warn!("Unknown type: {}. Please report to Interstellar Team!", x);
            tracing::warn!("Payload: {}", object.to_string());
            if inner_obj.pointer(pointer!["object"]).is_some() {
                // Extracting anyway.
                pointer.push(PointerNode::Key("object".into()));
                extract_obj(object, pointer, cnt + 1)
            } else {
                Some(pointer)
            }
        }
        None => {
            tracing::warn!("Cannot determine activity type!");
            tracing::warn!("Report this activity to Interstellar Team!");
            tracing::warn!("Payload: {}", object.to_string());
            Some(pointer)
        }
    }
}
