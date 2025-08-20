use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_wasm_bindgen::{Serializer, from_value};
use wasm_bindgen::prelude::*;

mod v0;
pub mod v1;

#[cfg(test)]
mod test_utils;

pub mod current {
    // this should always track the latest version, and is the only version
    // that is exported from notebook-types
    pub use crate::v1::*;
}

/** Generate type defs for dependencies supporting `serde` but not `tsify`.

Comments on specific definitions:

- Re: `Value`, we could borrow the definition of `JsonValue` in the `ts-rs` crate:
  <https://github.com/Aleph-Alpha/ts-rs/blob/main/ts-rs/tests/integration/serde_json.rs>.
  However, this is causing mysterious TS errors, so we use `unknown` instead.
- Re: `NonEmpty`, somewhat amazingly, the type system in TypeScript can express
  the constraint that an array be nonempty, with certain usage caveats:
  <https://stackoverflow.com/q/56006111>. For now, we will not attempt to
  enforce non-emptiness in the TypeScript layer.
 */
#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type Value = unknown;

export type Uuid = string;
export type Ustr = string;

export type NonEmpty<T> = Array<T>;
"#;

pub static CURRENT_VERSION: &str = "1";

#[wasm_bindgen(js_name = "currentVersion")]
pub fn current_version() -> String {
    CURRENT_VERSION.to_string()
}

#[derive(Serialize, Debug)]
pub enum VersionedDocument {
    V0(v0::Document),
    V1(v1::Document),
}

impl<'de> Deserialize<'de> for VersionedDocument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let version = value.get("version").and_then(Value::as_str).unwrap_or("0");

        match version {
            "0" => {
                let doc: v0::Document =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(VersionedDocument::V0(doc))
            }
            "1" => {
                let doc: v1::Document =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(VersionedDocument::V1(doc))
            }
            other => Err(serde::de::Error::custom(format!("unsupported version {other}"))),
        }
    }
}

impl VersionedDocument {
    pub fn to_current(self) -> current::Document {
        match self {
            VersionedDocument::V0(v0) => {
                // Recursive call to VersionedNotebook::to_current
                VersionedDocument::V1(v1::Document::migrate_from_v0(v0)).to_current()
            }

            VersionedDocument::V1(old1) => old1,
        }
    }
}

#[wasm_bindgen(js_name = "migrateDocument")]
pub fn migrate_document(input: JsValue) -> Result<JsValue, JsValue> {
    let doc: VersionedDocument =
        from_value(input).map_err(|e| JsValue::from_str(&format!("deserialize error: {e}")))?;

    let current_doc = doc.to_current();

    // By default some types will serialize to more complicated JS type (like HashMap -> Map) instead of
    // a "plain" JSON type. JS !== JSON
    let serializer = Serializer::json_compatible();

    let output = current_doc
        .serialize(&serializer)
        .map_err(|e| JsValue::from_str(&format!("serialize error: {e}")))?;

    Ok(output)
}

#[cfg(test)]
mod migration_tests {
    use super::VersionedDocument;
    use crate::test_utils::test_example_documents;

    #[test]
    fn test_v0_examples_migrate_to_current() {
        test_example_documents::<VersionedDocument, _>("examples/v0", |doc, _| {
            // ensure it migrates without panic
            let _ = doc.to_current();
        });
    }
}
