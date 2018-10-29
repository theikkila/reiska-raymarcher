extern crate image;
extern crate cgmath;
extern crate rayon;
use std::cmp::max;
use cgmath::prelude::*;
use cgmath::Vector3;
use rayon::prelude::*;
type Vec3 = Vector3<f32>;

const EPSILON: f32 = 0.00001;
const MAX_DIST: f32 = 2550.0;

struct Camera {
    pos: Vec3,
    up: Vec3,
    right: Vec3
}

impl Camera {
    fn dir(&self) -> Vec3 {
        self.right.cross(self.up).normalize()
    }
}

trait RObject {
    fn distance(&self, pos: &Vec3) -> f32;
}

struct Sphere {
    pos: Vec3,
    radius: f32
}

impl RObject for Sphere {
    fn distance(&self, pos: &Vec3) -> f32 {
        // let x = (pos.x % 1.0) - 0.5;
        // let z = (pos.z % 1.0) - 0.5;
        //
        // let k = Vector3::new(x, pos.y, z);
        (self.pos - pos).magnitude() - self.radius
    }
}

struct Plane {
    height: f32
}



struct Box {
    pos: Vec3,
    size: Vec3,
}

fn abs(p: &Vec3) -> Vec3 {
    p.map(|c| c.abs())
}

fn vmax(a: &Vec3, b: &Vec3) -> Vec3 {
    Vector3::new(
        a.x.max(b.x),
        a.y.max(b.y),
        a.z.max(b.z)
    )
}
fn vmin(a: &Vec3, b: &Vec3) -> Vec3 {
    Vector3::new(
        a.x.min(b.x),
        a.y.min(b.y),
        a.z.min(b.z)
    )
}

impl RObject for Box {
    fn distance(&self, pos: &Vec3) -> f32 {
        let zero = Vector3::new(0.0, 0.0, 0.0);
        let d = abs(pos) - self.size + self.pos;

        d.x.max(d.y).max(d.z).min(0.0) + vmax(&d, &zero).magnitude()
    }
}

struct Sponge;
//
// float sdSponge(in vec3 p, float scale) {
//    p = p * scale;
//    float t = sdBox(p, vec3(1.));
//
//    float s = 1.0;
//    for( int m=0; m<3; m++ )
//    {
//       vec3 a = mod( p*s, 2.0 )-1.0;
//       s *= 3.0;
//       vec3 r = abs(1.0 - 3.0*abs(a));
//
//       float da = max(r.x,r.y);
//       float db = max(r.y,r.z);
//       float dc = max(r.z,r.x);
//       float c = (min(da,min(db,dc))-1.0)/s;
//
//       t = max(t,c);
//    }
//     return t/scale;
// }

// impl RObject for Sponge {
//     fn distance(&self, pos: &Vec3) -> f32 {
//         let p = pos * 1.0;
//         let mut t = Box{pos: Vector3::new(1.0, 0.0, 0.0)}.distance(pos);
//         let mut s = 1.0;
//         for m in 0..3 {
//             let a = p.map(|c| c % 2.0 - 1.0);
//             s *= 3.0;
//             let r = a.map(|c| (1.0 - c.abs()*3.0).abs());
//             let da = r.x.max(r.y);
//             let db = r.y.max(r.z);
//             let dc = r.z.max(r.x);
//             let c = (da.min(db).min(dc))/s;
//
//             t = t.max(c)
//         }
//         t/1.0
//     }
// }

impl RObject for Plane {
    fn distance(&self, pos: &Vec3) -> f32 {
        //(Vector3::new(pos.x, pos.x.sin()*0.1, 0.0) - pos).magnitude()
        // self.height - pos.y + (pos.x*3.0).sin()*0.01+(pos.z*3.0).cos()*0.01
        self.height - pos.y
    }
}

fn estimate_normal(p: &Vec3) -> Vec3 {
    Vector3::new(
        scene_sdf(&(Vector3::new(p.x + EPSILON, p.y, p.z))) - scene_sdf(&(Vector3::new(p.x - EPSILON, p.y, p.z))),
        scene_sdf(&(Vector3::new(p.x, p.y + EPSILON, p.z))) - scene_sdf(&(Vector3::new(p.x, p.y - EPSILON, p.z))),
        scene_sdf(&(Vector3::new(p.x, p.y, p.z + EPSILON))) - scene_sdf(&(Vector3::new(p.x, p.y, p.z - EPSILON)))
    ).normalize()
}

fn scene_sdf(pos: &Vec3) -> f32 {
    //(-Box{pos: Vector3::new(0.0, -0.9, 0.0), size: Vector3::new(0.6, 0.6, 4.4)}.distance(&pos)).max(
    Sphere{pos: Vector3::new(0.0, 1.0, 0.0), radius: 1.5}.distance(&pos)
    .min(Plane{height: 2.0}.distance(&pos))
    // .min(Sphere{pos: Vector3::new(0.0, 0.0, 1.0), radius: 1.0}.distance(&pos))
    // .min(Sphere{pos: Vector3::new(0.0, 0.0, 2.0), radius: 1.0}.distance(&pos))
    //.min(Sphere{pos: Vector3::new(0.0, -1.1, 0.0), radius: 1.1}.distance(&pos))
    // .min(Sphere{pos: Vector3::new(0.0, 0.0, 3.0), radius: 1.0}.distance(&pos))
    // .min(Sphere{pos: Vector3::new(1.0, -0.4, 3.0), radius: 1.0}.distance(&pos))
    // .min(Sphere{pos: Vector3::new(-1.0, 0.0, 3.0), radius: 1.0}.distance(&pos))
    //.min(Sphere{pos: Vector3::new(-1.0, 0.0, 0.0), radius: 1.0}.distance(&pos))
}

fn diffuse(ray: &Vec3, n: Vec3) -> f32 {
    saturate(ray.magnitude2()*-0.01 + (ray * -1.0).dot(n))
}

fn saturate(f: f32) -> f32 {
    f.max(0.0).min(1.0)
}

fn raymarch(eye: &Vec3, dir: &Vec3) -> Option<Vec3> {
    let max_steps = 150;
    let mut depth = EPSILON;
    let ray = dir.normalize();
    for step in 1..max_steps {
        //println!("Step: {}", step);
        let p = (eye+ray*depth);
        let dist = scene_sdf(&p);
        if dist < EPSILON {
            return Some(p);
        }
        depth += dist;
        if depth >= MAX_DIST {
            return None
        }
    }
    None
}

fn shadow(light: &Vec3, dir: &Vec3) -> f32 {
    let max_steps = 150;
    let mut t = EPSILON;
    let ray = dir.normalize();
    let k = 4.0;
    let mut res: f32 = 1.0;
    for _ in 1..max_steps {
        //println!("Step: {}", step);
        let p = light+ray*t;
        let h = scene_sdf(&p);
        if h < EPSILON {
            return res;
        }
        t += h;
        res = res.min(k * h / t)
    }
    res

}


fn main() {
    println!("Reiska!");

    let cam = Camera{
        pos: Vector3::new(0.0, -2.0, -10.0),
        up: Vector3::new(0.0, 1.0, 0.0),
        right: Vector3::new(1.0, 0.0, 0.0)
    };

    let light = Vector3::new(10.0, -10.0, -1.0);

    let imgx = 512;
    let imgy = 512;
    let origo_x = imgx/2;
    let origo_y = imgy/2;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::RgbImage::new(imgx, imgy);
    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let cy = (y as f32 - origo_y as f32) / (origo_y as f32);
        let cx = (x as f32 - origo_x as f32) / (origo_x as f32);

        let ray_pos = cam.pos + (cx * cam.right) + (cy * cam.up);
        let focal_length = 0.2;
        let focal_point = cam.pos + cam.dir() * -1.0 * focal_length;
        //println!("F:{:?}", focal_point);
        let dir = (focal_point * -1.0 * focal_length + (cx * cam.right) + (cy * cam.up)).normalize();
        //println!("D:{:?}", dir);


        let p = match raymarch(&ray_pos, &dir) {
            Some(dist_v) => {
                //if dist > EPSILON {
                    //println!("{}", dist);
                //}
                let light_ray = (dist_v-light).normalize();
                let neg_light_ray = light_ray * -1.0;
                let n = estimate_normal(&dist_v);// * -1.0;
                let dif = saturate(diffuse(&light_ray, n));
                //let dif = dist_v.magnitude(); //-1.0;
                let h = (light_ray + ray_pos).normalize();
                let intensity = dif.powf(32.0);
                //println!("{}", dif*1000.0);
                //let specular = dif.powf(3.0);
                // let sh = match raymarch(&light, &light_ray) {
                //     Some(ld) => {
                //         if (ld-dist_v).magnitude() < EPSILON {
                //             0.0
                //         } else {
                //             1.0
                //         }
                //     },
                //     None => 0.0
                // };
                let sh = shadow(&dist_v, &neg_light_ray);

                //image::Rgb([(intensity * 255.0) as u8, (saturate(dif)*255.0) as u8, ((shadowy)*255.0) as u8])
                //image::Rgb([(intensity * 255.0) as u8, (saturate(dif)*255.0) as u8, ((shadowy)*255.0) as u8])
                let background = 0.0;
                let fog_ratio = saturate(dist_v.magnitude() / 255.0);
                let nfog_ratio = 1.0 - fog_ratio;
                let mut r = saturate(sh*dif+0.12)*255.0;//dif * 255.0*nfog_ratio;
                //r *= shadow;
                let mut g = saturate(sh*dif+0.12)*255.0;//255.0*nfog_ratio + background * fog_ratio;
                //g *= shadow;
                let mut b = saturate(sh*dif+0.12)*255.0;//255.0;//255.0*nfog_ratio + background * fog_ratio;
                //b *= ;
                image::Rgb([r as u8, g as u8, b as u8])
                //image::Rgb([255, 0 as u8, 0 as u8])

            },
            None => image::Rgb([0, 0, 0])
        };
        *pixel = p;

         // Create an 8bit pixel of type Luma and value i
         // and assign in to the pixel at position (x, y)

    }
    // Save the image as “fractal.png”, the format is deduced from the path
    imgbuf.save("reiska.png").unwrap();
}
