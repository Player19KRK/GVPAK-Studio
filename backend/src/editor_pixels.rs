//! Alpha edits use image coordinates and leave the stored source and RGB bytes intact.
use crate::Res;
use serde_json::Value;
pub(crate) fn color(value:&Value)->Res<[u8;3]>{let values=value.as_array().filter(|v|v.len()==3).ok_or("INVALID_COLOR")?;let mut rgb=[0;3];for(i,v)in values.iter().enumerate(){rgb[i]=v.as_u64().filter(|v|*v<=255).ok_or("INVALID_COLOR")? as u8;}Ok(rgb)}
pub(crate) fn validate_mask(op:&Value)->Res<()>{
 if !matches!(op["mode"].as_str(),Some("add"|"erase"))||!op["radius"].as_f64().is_some_and(|n|n.is_finite()&&n>0.&&n<=0.5){return Err("INVALID_MASK".into())}
 let points=op["points"].as_array().filter(|p|!p.is_empty()&&p.len()<=4096).ok_or("INVALID_MASK")?;
 for point in points{if point.as_array().is_none_or(|p|p.len()!=2||p.iter().any(|v|!v.as_f64().is_some_and(|n|n.is_finite()&&(0. ..=1.).contains(&n)))){return Err("INVALID_MASK".into())}}Ok(())
}
pub(crate) fn mask(image:image::DynamicImage,op:&Value)->Res<image::DynamicImage>{
 validate_mask(op)?;let mut image=image.to_rgba8();let(w,h)=(image.width() as f64,image.height() as f64);let radius=op["radius"].as_f64().unwrap()*w.min(h);let points=op["points"].as_array().unwrap().iter().map(|p|(p[0].as_f64().unwrap()*w,p[1].as_f64().unwrap()*h)).collect::<Vec<_>>();let alpha=if op["mode"]=="add"{255}else{0};
 for i in 0..points.len(){let(a,b)=(points[i.saturating_sub(1)],points[i]);let(dx,dy)=(b.0-a.0,b.1-a.1);let length=dx*dx+dy*dy;
  for y in ((a.1.min(b.1)-radius).floor().max(0.) as u32)..((a.1.max(b.1)+radius).ceil().min(h) as u32){for x in ((a.0.min(b.0)-radius).floor().max(0.) as u32)..((a.0.max(b.0)+radius).ceil().min(w) as u32){let(px,py)=(x as f64+0.5,y as f64+0.5);let t=if length>0.{(((px-a.0)*dx+(py-a.1)*dy)/length).clamp(0.,1.)}else{0.};if (px-a.0-t*dx).powi(2)+(py-a.1-t*dy).powi(2)<=radius*radius{image.get_pixel_mut(x,y)[3]=alpha;}}}
 }Ok(image::DynamicImage::ImageRgba8(image))
}
#[cfg(test)]mod tests{use super::*;use serde_json::json;
 #[test]fn continuous_alpha_stroke_preserves_rgb_and_restores_alpha(){let image=image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(100,50,image::Rgba([42,77,123,255])));let mut op=json!({"mode":"erase","radius":0.08,"points":[[0.1,0.5],[0.9,0.5]]});let erased=mask(image,&op).unwrap().to_rgba8();assert_eq!(erased.get_pixel(50,25).0,[42,77,123,0]);assert_eq!(erased.get_pixel(50,10)[3],255);op["mode"]=json!("add");assert_eq!(mask(image::DynamicImage::ImageRgba8(erased),&op).unwrap().to_rgba8().get_pixel(50,25).0,[42,77,123,255]);}
 #[test]fn rejects_bad_mask_and_color_before_processing(){assert!(validate_mask(&json!({"mode":"erase","radius":0.1,"points":[[1.2,0.5]]})).is_err());assert!(validate_mask(&json!({"mode":"erase","radius":0,"points":[]})).is_err());assert!(color(&json!([0,256,0])).is_err());assert_eq!(color(&json!([5,10,15])).unwrap(),[5,10,15]);}
}
