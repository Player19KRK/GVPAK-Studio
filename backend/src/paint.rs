use image::RgbaImage;
use serde::{Deserialize, Serialize};
use crate::core::{Edit, Res};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stroke {
    pub points: Vec<[f64; 2]>,
    pub color: [u8; 3],
    pub size: f64,
    pub hardness: f64,
    pub opacity: f64,
    #[serde(default)]
    pub erase: bool,
}

pub fn validate(strokes: &[Stroke]) -> Res<()> {
    for s in strokes {
        if !s.size.is_finite() || !(1.0..=512.0).contains(&s.size)
            || !(0.0..=1.0).contains(&s.hardness) || !(0.0..=1.0).contains(&s.opacity)
            || s.points.is_empty() || s.points.iter().flatten().any(|v| !v.is_finite() || !(0.0..=1.0).contains(v)) {
            return Err("INVALID_STROKE: Invalid brush parameters or coordinates".into());
        }
    }
    Ok(())
}

fn crop(e: &Edit, (sw, sh): (u32, u32)) -> [f64; 4] {
    let [x,y,w,h] = e.crop.unwrap_or([0.,0.,1.,1.]);
    let x=(x*sw as f32).round() as u32;let y=(y*sh as f32).round() as u32;
    let w=((w*sw as f32).round() as u32).max(1).min(sw-x);
    let h=((h*sh as f32).round() as u32).max(1).min(sh-y);
    [x as f64/sw as f64,y as f64/sh as f64,w as f64/sw as f64,h as f64/sh as f64]
}
fn view([x,y]: [f64; 2], c: [f64; 4], e: &Edit) -> [f64;2] {
    let (mut x,mut y)=((x-c[0])/c[2],(y-c[1])/c[3]);
    (x,y)=match e.rotation {90=>(1.-y,x),180=>(1.-x,1.-y),270=>(y,1.-x),_=>(x,y)};
    if e.flip_h {x=1.-x;}if e.flip_v {y=1.-y;}[x,y]
}

// Same maximum-coverage brush kernel as src/paint.js. Opacity is applied once per stroke.
pub fn layer(width: u32, height: u32, e: &Edit, source: (u32,u32)) -> RgbaImage {
    let (w,h)=(width as usize,height as usize);
    let mut result=RgbaImage::new(width,height);
    let mut mask=vec![0u8;w*h];
    let c=crop(e,source);
    let span=if e.rotation%180!=0 {c[3]*source.1 as f64}else{c[2]*source.0 as f64};
    for s in &e.strokes {
        mask.fill(0);
        let r=(s.size*0.5*width as f64/span).max(0.5);
        let feather=(r*(1.-s.hardness)).max(1.);
        let (mut left,mut top,mut right,mut bottom)=(w,h,0usize,0usize);
        let mut dab=|cx:f64,cy:f64| {
            let x0=(cx-r).floor().max(0.).min(width as f64) as usize;
            let y0=(cy-r).floor().max(0.).min(height as f64) as usize;
            let x1=(cx+r).ceil().max(0.).min(width as f64) as usize;
            let y1=(cy+r).ceil().max(0.).min(height as f64) as usize;
            left=left.min(x0);top=top.min(y0);right=right.max(x1);bottom=bottom.max(y1);
            for y in y0..y1 {for x in x0..x1 {
                let distance=(x as f64+0.5-cx).hypot(y as f64+0.5-cy);
                let alpha=(((r-distance)/feather).clamp(0.,1.)*255.).round() as u8;
                let i=y*w+x;mask[i]=mask[i].max(alpha);
            }}
        };
        let mut previous:Option<[f64;2]>=None;
        for point in &s.points {
            let v=view(*point,c,e);let p=[v[0]*width as f64,v[1]*height as f64];
            if let Some(a)=previous {
                let count=((p[0]-a[0]).hypot(p[1]-a[1])/(r*0.25).max(0.5)).ceil().max(1.) as usize;
                for i in 1..=count {let t=i as f64/count as f64;dab(a[0]+(p[0]-a[0])*t,a[1]+(p[1]-a[1])*t);}
            } else {dab(p[0],p[1]);}
            previous=Some(p);
        }
        for y in top..bottom {for x in left..right {
            let a=mask[y*w+x] as f64/255.*s.opacity;if a==0. {continue;}
            let pixel=result.get_pixel_mut(x as u32,y as u32);let old=pixel[3] as f64/255.;
            if s.erase {pixel[3]=(old*(1.-a)*255.).round() as u8;continue;}
            let alpha=a+old*(1.-a);
            for k in 0..3 {pixel[k]=((s.color[k] as f64*a+pixel[k] as f64*old*(1.-a))/alpha).round() as u8;}
            pixel[3]=(alpha*255.).round() as u8;
        }}
    }
    result
}

pub fn composite(base: &mut RgbaImage, e: &Edit, source: (u32,u32)) {
    if e.strokes.is_empty(){return;}
    let overlay=layer(base.width(),base.height(),e,source);
    for (dest,paint) in base.pixels_mut().zip(overlay.pixels()) {
        let a=paint[3] as f64/255.;if a==0. {continue;}
        let old=dest[3] as f64/255.;let alpha=a+old*(1.-a);
        for k in 0..3 {dest[k]=((paint[k] as f64*a+dest[k] as f64*old*(1.-a))/alpha).round() as u8;}
        dest[3]=(alpha*255.).round() as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn stroke(erase:bool)->Stroke{Stroke{points:vec![[0.25,0.5],[0.5,0.5],[0.75,0.5]],color:[227,42,50],size:12.,hardness:1.,opacity:0.5,erase}}
    #[test]
    fn stroke_opacity_does_not_accumulate_and_eraser_preserves_base(){
        let mut e=Edit{strokes:vec![stroke(false)],..Edit::default()};
        let first=layer(40,40,&e,(40,40));assert_eq!(first.get_pixel(20,20)[3],128);
        let mut erase=stroke(true);erase.opacity=1.;e.strokes.push(erase);
        let mut base=RgbaImage::from_pixel(40,40,image::Rgba([20,40,60,255]));composite(&mut base,&e,(40,40));
        assert_eq!(*base.get_pixel(20,20),image::Rgba([20,40,60,255]));
    }
    #[test]
    fn source_coordinates_follow_crop_rotation_and_flip(){
        let mut s=stroke(false);s.points=vec![[0.3,0.4]];s.size=4.;s.opacity=1.;
        let e=Edit{crop:Some([0.1,0.2,0.5,0.6]),rotation:90,flip_h:true,strokes:vec![s],..Edit::default()};
        let out=layer(60,50,&e,(100,100));assert!(out.get_pixel(20,20)[3]>200);
    }
    #[test]
    fn backwards_compatible_and_invalid_strokes_rejected(){
        let mut v=serde_json::to_value(Edit::default()).unwrap();v.as_object_mut().unwrap().remove("strokes");
        assert!(serde_json::from_value::<Edit>(v).unwrap().strokes.is_empty());
        let mut s=stroke(false);s.points[0][0]=f64::NAN;assert!(validate(&[s]).is_err());
    }
    #[test]
    fn paint_color_is_independent_of_image_adjustment(){
        let mut s=stroke(false);s.opacity=1.;
        let e=Edit{exposure:2.,strokes:vec![s],..Edit::default()};
        let base=image::DynamicImage::ImageRgba8(RgbaImage::from_pixel(40,40,image::Rgba([20,40,60,255])));
        let out=crate::core::render(base,&e,None).unwrap().to_rgba8();assert_eq!(*out.get_pixel(20,20),image::Rgba([227,42,50,255]));
    }
}
