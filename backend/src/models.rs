use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub resolutions: Vec<String>,
    pub ratios: Vec<String>,
    pub input_usd_per_million: f64,
    pub text_usd_per_million: f64,
    pub image_usd_per_million: f64,
    pub image_usd: BTreeMap<String, Option<f64>>,
    pub source: String,
    pub checked_at: String,
    pub note: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    pub models: Vec<Model>,
}
pub fn parse(text: &str) -> Result<Catalog, String> {
    let catalog: Catalog = serde_json::from_str(text).map_err(|e| format!("models.json: {e}"))?;
    if catalog.schema_version != 1 || catalog.models.is_empty() {
        return Err("models.json: schemaVersion 必须为 1，models 不能为空".into());
    }
    let mut ids = HashSet::new();
    for (i, m) in catalog.models.iter().enumerate() {
        let fail = |field: &str| format!("models[{i}].{field}: 配置无效");
        if m.id.is_empty()
            || !m
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'.')
            || !ids.insert(&m.id)
        {
            return Err(fail("id"));
        }
        if m.name.trim().is_empty() {
            return Err(fail("name"));
        }
        if m.resolutions.is_empty()
            || m.resolutions
                .iter()
                .any(|x| !["1K", "2K", "4K"].contains(&x.as_str()))
        {
            return Err(fail("resolutions"));
        }
        if m.ratios.is_empty()
            || m.ratios.iter().any(|x| {
                ![
                    "1:1", "4:3", "3:4", "16:9", "9:16", "3:2", "2:3", "2:1", "1:2", "4:5", "5:4",
                    "21:9", "1:4", "4:1", "1:8", "8:1",
                ]
                .contains(&x.as_str())
            })
        {
            return Err(fail("ratios"));
        }
        for (field, value) in [
            ("inputUsdPerMillion", m.input_usd_per_million),
            ("textUsdPerMillion", m.text_usd_per_million),
            ("imageUsdPerMillion", m.image_usd_per_million),
        ] {
            if !value.is_finite() || value < 0. {
                return Err(fail(field));
            }
        }
        if m.resolutions.iter().any(|r| !m.image_usd.contains_key(r))
            || m.image_usd
                .values()
                .flatten()
                .any(|v| !v.is_finite() || *v < 0.)
        {
            return Err(fail("imageUsd"));
        }
        if m.source.trim().is_empty() || m.checked_at.trim().is_empty() {
            return Err(fail("source/checkedAt"));
        }
    }
    Ok(catalog)
}
fn load_raw(root: &Path) -> (Catalog, Option<String>) {
    let defaults =
        || parse(include_str!("../../config/models.json")).expect("bundled model catalog");
    match std::fs::read_to_string(root.join("models.json")) {
        Ok(text) => match parse(&text) {
            Ok(config) => (config, None),
            Err(error) => (defaults(), Some(format!("{error}；已使用内置配置"))),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (defaults(), None),
        Err(e) => (
            defaults(),
            Some(format!("models.json: {e}；已使用内置配置")),
        ),
    }
}

// A missing modality breakdown cannot safely be interpreted as zero image tokens.
pub fn actual_usd(m: &Model, usage: &serde_json::Value) -> Option<f64> {
    let input = usage.get("promptTokenCount")?.as_u64()? as f64;
    let details = usage.get("candidatesTokensDetails")?.as_array()?;
    let mut image = 0_u64;
    let mut text = 0_u64;
    for item in details {
        let count = item.get("tokenCount")?.as_u64()?;
        match item.get("modality")?.as_str()? {
            "IMAGE" => image = image.checked_add(count)?,
            "TEXT" => text = text.checked_add(count)?,
            _ => return None,
        }
    }
    if image.checked_add(text)? != usage.get("candidatesTokenCount")?.as_u64()? {
        return None;
    }
    let thoughts = usage
        .get("thoughtsTokenCount")
        .and_then(|x| x.as_u64())
        .unwrap_or(0) as f64;
    Some(
        (input * m.input_usd_per_million
            + (text as f64 + thoughts) * m.text_usd_per_million
            + image as f64 * m.image_usd_per_million)
            / 1e6,
    )
}
pub fn cost_bounds(m: &Model, usage: &serde_json::Value) -> Option<serde_json::Value> {
    if let Some(exact) = actual_usd(m, usage) {
        return Some(
            serde_json::json!({"minimumUsd":exact,"maximumUsd":exact,"complete":true,"note":"按返回用量及请求时价格折算，非账单"}),
        );
    }
    let input = usage.get("promptTokenCount")?.as_u64()? as f64;
    let candidates = usage.get("candidatesTokenCount")?.as_u64()?;
    let thoughts = usage
        .get("thoughtsTokenCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as f64;
    let mut known = 0_u64;
    let mut cost = input * m.input_usd_per_million + thoughts * m.text_usd_per_million;
    for part in usage.get("candidatesTokensDetails")?.as_array()? {
        let n = part.get("tokenCount")?.as_u64()?;
        known = known.checked_add(n)?;
        cost += n as f64
            * match part.get("modality")?.as_str()? {
                "IMAGE" => m.image_usd_per_million,
                "TEXT" => m.text_usd_per_million,
                _ => return None,
            };
    }
    let missing = candidates.checked_sub(known)? as f64;
    Some(serde_json::json!({
        "minimumUsd": (cost + missing * m.text_usd_per_million.min(m.image_usd_per_million)) / 1e6,
        "maximumUsd": (cost + missing * m.text_usd_per_million.max(m.image_usd_per_million)) / 1e6,
        "complete": false,
        "note": "返回的输出分类数量与总数不一致，未分类 tokens 按文本/图像两档计价给出范围，不推测精确扣费"
    }))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_validation_and_fallback() {
        let root = tempfile::tempdir().unwrap();
        let (catalog, warning) = load(root.path());
        assert!(warning.is_none());
        assert_eq!(catalog.models.len(), 5);
        assert!(catalog.models.iter().any(|m|m.id=="GPT-Image-2.5-Flare"));
        assert!(catalog.models.iter().any(|m|m.id=="GPT-Image-2.5-Sunburst"));
        let mut value = serde_json::to_value(&catalog).unwrap();
        value["models"][0]["inputUsdPerMillion"] = serde_json::json!(-1);
        std::fs::write(root.path().join("models.json"), value.to_string()).unwrap();
        let (fallback, warning) = load(root.path());
        assert!(warning.unwrap().contains("models[0].inputUsdPerMillion"));
        assert_eq!(fallback.models[0].input_usd_per_million, 1.5);
        value["models"][0]["inputUsdPerMillion"] = serde_json::json!(2);
        std::fs::write(root.path().join("models.json"), value.to_string()).unwrap();
        assert_eq!(load(root.path()).0.models[0].input_usd_per_million, 2.);
    }
    #[test]
    fn modality_cost_and_unknown_usage() {
        let c = parse(include_str!("../../config/models.json")).unwrap();
        let usage = serde_json::json!({"promptTokenCount":1000,"candidatesTokenCount":1220,"thoughtsTokenCount":200,"candidatesTokensDetails":[{"modality":"IMAGE","tokenCount":1120},{"modality":"TEXT","tokenCount":100}]});
        assert!((actual_usd(&c.models[0], &usage).unwrap() - 0.03735).abs() < 1e-10);
        assert!(
            actual_usd(
                &c.models[0],
                &serde_json::json!({"promptTokenCount":1000,"candidatesTokenCount":1120})
            )
            .is_none()
        );
        let observed = serde_json::json!({"promptTokenCount":1152,"candidatesTokenCount":1346,"thoughtsTokenCount":800,"candidatesTokensDetails":[{"modality":"IMAGE","tokenCount":1120}]});
        let bounds = cost_bounds(&c.models[0], &observed).unwrap();
        assert!((bounds["minimumUsd"].as_f64().unwrap() - 0.043023).abs() < 1e-10);
        assert!((bounds["maximumUsd"].as_f64().unwrap() - 0.048108).abs() < 1e-10);
    }
}

pub fn load(root:&Path)->(Catalog,Option<String>){let (mut c,w)=load_raw(root);let bundled=parse(include_str!("../../config/models.json")).expect("catalog");for m in bundled.models.into_iter().filter(|m|m.id.starts_with("GPT-Image-2.5-")){if !c.models.iter().any(|x|x.id==m.id){c.models.push(m)}}(c,w)}
