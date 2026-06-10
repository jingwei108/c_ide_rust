use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Deserialize)]
struct LayoutData {
    #[allow(dead_code)]
    version: i32,
    #[allow(dead_code)]
    #[serde(rename = "generated_at")]
    generated_at: String,
    classes: HashMap<String, ClassLayoutJson>,
    #[serde(rename = "method_map")]
    method_map: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
struct ClassLayoutJson {
    #[allow(dead_code)]
    cpp_name: String,
}

static LAYOUT_DATA: LazyLock<LayoutData> = LazyLock::new(|| {
    let json = include_str!("builtin_layout_data.json");
    serde_json::from_str(json).expect("builtin_layout_data.json is invalid")
});

/// Map a C++ standard container type name to its Cide internal type name.
/// e.g. `vector<int>` → `cide_vec_int`
pub fn cpp_type_to_cide(name: &str) -> Option<&'static str> {
    for (cide_name, cls) in &LAYOUT_DATA.classes {
        if cls.cpp_name == name {
            // Leak once to obtain a 'static reference.
            return Some(Box::leak(cide_name.clone().into_boxed_str()));
        }
    }
    None
}

/// Check whether the given type name is a builtin container type.
pub fn is_builtin_container(name: &str) -> bool {
    cpp_type_to_cide(name).is_some()
}

/// Map a builtin container method call to the underlying Cide container library
/// function name.  The mapping is read from builtin_layout_data.json.
pub fn map_container_method(class_name: &str, method: &str) -> Option<&'static str> {
    let class_map = LAYOUT_DATA.method_map.get(class_name)?;
    let func_name = class_map.get(method)?;
    Some(Box::leak(func_name.clone().into_boxed_str()))
}
