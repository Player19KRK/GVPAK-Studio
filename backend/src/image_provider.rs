use crate::core::Res;
use serde_json::{Value,json};
use base64::{Engine,engine::general_purpose::STANDARD};
pub fn is_poe(model:&str)->bool{matches!(model,"GPT-Image-2.5-Flare"|"GPT-Image-2.5-Sunburst")}
pub fn credential()->Res<keyring::Entry>{let profile=std::env::var("GVPAK_PROFILE").unwrap_or("dev".into());keyring::Entry::new(&format!("EDGE-STUDIO-{profile}"),"poe-official").map_err(|e|e.to_string())}
fn decode(bytes:Vec<u8>)->Res<image::DynamicImage>{let mut r=image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().map_err(|_|"INVALID_IMAGE")?;let mut limits=image::Limits::default();limits.max_alloc=Some(600_000_000);limits.max_image_width=Some(16000);limits.max_image_height=Some(16000);r.limits(limits);r.decode().map_err(|_|"INVALID_IMAGE".into())}
fn urls(v:&Value,out:&mut Vec<String>){match v{Value::String(s)=>{if s.starts_with("data:image/"){out.push(s.clone());}for (start,_) in s.match_indices("https://"){let rest=&s[start..];out.push(rest.split(|c:char|c.is_whitespace()||c==')'||c=='"'||c=='<'||c==']').next().unwrap_or("").to_string());}},Value::Array(v)=>for x in v{urls(x,out)},Value::Object(v)=>for x in v.values(){urls(x,out)},_=>{}}}
fn media_host(url:&reqwest::Url)->bool{url.scheme()=="https"&&url.host_str().is_some_and(|h|h=="poecdn.net"||h.ends_with(".poecdn.net"))&&url.username().is_empty()&&url.password().is_none()}
fn request_body(model:&str,prompt:&str,input:Option<&[u8]>,resolution:&str,ratio:&str)->Value{
 if is_poe(model){
  let mut content=vec![json!({"type":"text","text":format!("{prompt}\nOutput aspect ratio: {ratio}. Requested resolution: {resolution}. Return the generated image.")})];
  if let Some(bytes)=input{content.push(json!({"type":"image_url","image_url":{"url":format!("data:image/png;base64,{}",STANDARD.encode(bytes))}}));}
  return json!({"model":model,"stream":false,"messages":[{"role":"user","content":content}]});
 }
 let mut parts=vec![json!({"text":prompt})];if let Some(bytes)=input{parts.push(json!({"inlineData":{"mimeType":"image/png","data":STANDARD.encode(bytes)}}));}
 json!({"contents":[{"role":"user","parts":parts}],"generationConfig":{"responseModalities":["TEXT","IMAGE"],"imageConfig":{"aspectRatio":ratio,"imageSize":resolution}}})
}
pub async fn generate(model:&str,prompt:&str,input:Option<Vec<u8>>,resolution:&str,ratio:&str)->Res<(image::DynamicImage,Value)>{
 let client=reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().map_err(|e|e.to_string())?;
 if is_poe(model){
  let key=credential()?.get_password().map_err(|_|"POE_KEY_REQUIRED: 请在主机设置 Poe API 密钥")?;
  let body=request_body(model,prompt,input.as_deref(),resolution,ratio);
  let r=client.post("https://api.poe.com/v1/chat/completions").bearer_auth(key).json(&body).timeout(std::time::Duration::from_secs(600)).send().await.map_err(|_|"NETWORK_OUTCOME_UNKNOWN: Poe 请求结果未确认，请勿重复提交")?;
  if !r.status().is_success(){return Err(format!("POE_API_ERROR: HTTP {}，未自动重试",r.status().as_u16()))}let v:Value=r.json().await.map_err(|_|"INVALID_API_RESPONSE")?;
  let mut candidates=vec![];urls(&v["choices"][0]["message"],&mut candidates);
  for value in candidates{if value.starts_with("data:image/"){if let Some((_,data))=value.split_once(","){if data.len()<60*1024*1024{if let Ok(bytes)=STANDARD.decode(data){if let Ok(im)=decode(bytes){return Ok((im,v))}}}}continue}
   let Ok(url)=reqwest::Url::parse(&value) else{continue};if !media_host(&url){continue}
   let mut r=client.get(url).timeout(std::time::Duration::from_secs(90)).send().await.map_err(|_|"IMAGE_DOWNLOAD_FAILED")?;if !r.status().is_success(){continue}let mut bytes=vec![];while let Some(chunk)=r.chunk().await.map_err(|_|"IMAGE_DOWNLOAD_FAILED")?{if bytes.len()+chunk.len()>40*1024*1024{return Err("IMAGE_TOO_LARGE".into())}bytes.extend_from_slice(&chunk)}if let Ok(im)=decode(bytes){return Ok((im,v))}
  }
  return Err("NO_IMAGE: Poe 未返回可读取的图片。请检查 Poe 任务记录；未自动重试。".into())
 }
 let key=crate::credential()?.get_password().map_err(|_|"KEY_NOT_CONFIGURED")?;
 let body=request_body(model,prompt,input.as_deref(),resolution,ratio);
 let r=client.post(format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent")).header("x-goog-api-key",key).json(&body).timeout(std::time::Duration::from_secs(180)).send().await.map_err(|_|"NETWORK_OUTCOME_UNKNOWN: 请求结果未确认，请勿重复提交")?;
 if !r.status().is_success(){return Err(format!("GOOGLE_API_ERROR: HTTP {}，未自动重试",r.status().as_u16()))}let v:Value=r.json().await.map_err(|_|"INVALID_API_RESPONSE")?;let data=v["candidates"][0]["content"]["parts"].as_array().and_then(|p|p.iter().find_map(|x|x["inlineData"]["data"].as_str())).ok_or("NO_IMAGE")?;let im=decode(STANDARD.decode(data).map_err(|_|"INVALID_IMAGE")?)?;Ok((im,v))
}
#[cfg(test)]mod tests{use super::*;#[test]fn restrict_media_hosts(){assert!(media_host(&reqwest::Url::parse("https://pfst.cf2.poecdn.net/image.png").unwrap()));for u in ["http://pfst.cf2.poecdn.net/a","https://127.0.0.1/a","https://poecdn.net.evil.test/a"]{assert!(!media_host(&reqwest::Url::parse(u).unwrap()))}}
#[test]fn finds_markdown_media(){let mut out=vec![];urls(&json!({"content":"![image](https://pfst.cf2.poecdn.net/a.png)"}),&mut out);assert_eq!(out,vec!["https://pfst.cf2.poecdn.net/a.png"]);}}

#[cfg(test)]mod request_tests{use super::*;
 #[test]fn text_generation_never_sends_an_image(){for model in ["gemini-fixture","GPT-Image-2.5-Flare"]{let body=request_body(model,"draw a box",None,"1K","1:1");let body=body.to_string();assert!(body.contains("draw a box"));assert!(!body.contains("inlineData"));assert!(!body.contains("image_url"));}}
 #[test]fn reference_generation_keeps_image_input(){for model in ["gemini-fixture","GPT-Image-2.5-Flare"]{let body=request_body(model,"edit",Some(b"fixture"),"1K","1:1").to_string();assert!(body.contains("Zml4dHVyZQ=="));}}
}
