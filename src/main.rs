use eframe::{self, egui::{self}};
use image::{open, DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgba};
use rand::{thread_rng, Rng};
use std::path::Path;

/// Dimensão máxima permitida para largura ou altura (em pixels)
const MAX_DIM: u32 = 300;

/// Tipos de algoritmo disponíveis
#[derive(Clone, PartialEq)]
enum Algorithm {
    Blur,
    Sharpen,
    EdgeDetect,
    Invert,
    Mean,
    Maximum,
    Median,
    Minimum,
    ZoomNN,
    ZoomBilinear,
    Grayscale,
    Negative,
    Sobel,
    Laplacian,
    Binarize,
    Threshold(u8),
    SaltPepper(f32),
    PseudoColors,
}

// Aplicação de máscara 3×3 genérica
fn apply_kernel(img: &DynamicImage, kernel: [[f32;3];3], factor: f32, bias: f32) -> DynamicImage {
    let (w,h) = img.dimensions();
    let mut buf = ImageBuffer::new(w,h);
    for y in 1..h-1 {
        for x in 1..w-1 {
            let mut acc = [0.0f32;3];
            for ky in 0..3usize {
                for kx in 0..3usize {
                    let px = img.get_pixel(x + kx as u32 - 1, y + ky as u32 - 1);
                    let k = kernel[ky][kx];
                    acc[0] += px[0] as f32 * k;
                    acc[1] += px[1] as f32 * k;
                    acc[2] += px[2] as f32 * k;
                }
            }
            let r = (factor * acc[0] + bias).clamp(0.0,255.0) as u8;
            let g = (factor * acc[1] + bias).clamp(0.0,255.0) as u8;
            let b = (factor * acc[2] + bias).clamp(0.0,255.0) as u8;
            let a = img.get_pixel(x,y)[3];
            buf.put_pixel(x,y, Rgba([r,g,b,a]));
        }
    }
    DynamicImage::ImageRgba8(buf)
}

// Conversão para escala de cinza
fn to_grayscale(img: &DynamicImage) -> DynamicImage {
    let (w,h) = img.dimensions();
    let mut buf = ImageBuffer::new(w,h);
    for (x,y,p) in img.to_rgba8().enumerate_pixels() {
        let gray = (0.299*p[0] as f32 + 0.587*p[1] as f32 + 0.114*p[2] as f32) as u8;
        buf.put_pixel(x,y, Rgba([gray,gray,gray,p[3]]));
    }
    DynamicImage::ImageRgba8(buf)
}

// Filtro de mediana colorido 3×3
fn median_filter(img: &DynamicImage) -> DynamicImage {
    let (w,h) = img.dimensions();
    let pix = img.to_rgba8();
    let mut buf = ImageBuffer::new(w,h);
    for y in 1..h-1 {
        for x in 1..w-1 {
            let mut rs = Vec::new(); let mut gs = Vec::new(); let mut bs = Vec::new();
            let mut a = 255;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let p = pix.get_pixel((x as i32+dx) as u32,(y as i32+dy) as u32);
                    rs.push(p[0]); gs.push(p[1]); bs.push(p[2]); a = p[3];
                }
            }
            rs.sort_unstable(); gs.sort_unstable(); bs.sort_unstable();
            buf.put_pixel(x,y, Rgba([rs[4],gs[4],bs[4],a]));
        }
    }
    DynamicImage::ImageRgba8(buf)
}

fn maximum_filter(img: &DynamicImage) -> DynamicImage {
    let (w,h) = img.dimensions();
    let pix = img.to_rgba8();
    let mut buf = ImageBuffer::new(w,h);
    for y in 1..h-1 {
        for x in 1..w-1 {
            let mut rs: Vec<Rgba<u8>> = Vec::new();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let p = pix.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32);
                    rs.push(*p);
                }
            }
            // Sort by luminance (grayscale value)
            rs.sort_by(|a, b| {
                let lum_a = a[0] as f64 * 0.2126 + a[1] as f64 * 0.7152 + a[2] as f64 * 0.0722;
                let lum_b = b[0] as f64 * 0.2126 + b[1] as f64 * 0.7152 + b[2] as f64 * 0.0722;
                lum_a.partial_cmp(&lum_b).unwrap_or(std::cmp::Ordering::Equal)
            });
            // Take the maximum value (last element after sorting)
            buf.put_pixel(x, y, rs[8]);
        }
    }
    DynamicImage::ImageRgba8(buf)
}

fn minimum_filter(img: &DynamicImage) -> DynamicImage {
    let (w, h) = img.dimensions();
    let pix = img.to_rgba8();
    let mut buf = ImageBuffer::new(w,h);
    for y in 1..h-1 {
        for x in 1..w-1 {
            let mut rs: Vec<Rgba<u8>> = Vec::new();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let p = pix.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32);
                    rs.push(*p);
                }
            }
            rs.sort_by(|a, b| {
                let lum_a = a[0] as f64 * 0.2126 + a[1] as f64 * 0.7152 + a[2] as f64 * 0.0722;
                let lum_b = b[0] as f64 * 0.2126 + b[1] as f64 * 0.7152 + b[2] as f64 * 0.0722;
                lum_a.partial_cmp(&lum_b).unwrap_or(std::cmp::Ordering::Equal)
            });
            buf.put_pixel(x, y, rs[0]);
        }
    }
    DynamicImage::ImageRgba8(buf)
}

// Binarização fixa ou por limiar
fn binarize(img: &DynamicImage, thresh: u8) -> DynamicImage {
    let gray = to_grayscale(img).to_luma8();
    let (w,h) = img.dimensions();
    let mut buf = ImageBuffer::new(w,h);
    for y in 0..h {
        for x in 0..w {
            let v = if gray.get_pixel(x,y)[0] > thresh {255} else{0};
            buf.put_pixel(x,y, Rgba([v,v,v,255]));
        }
    }
    DynamicImage::ImageRgba8(buf)
}

// Ruído sal e pimenta
fn salt_pepper(img: &DynamicImage, p: f32) -> DynamicImage {
    let (w,h) = img.dimensions();
    let mut buf = img.to_rgba8();
    let mut rng = thread_rng();
    for y in 0..h {
        for x in 0..w {
            let r: f32 = rng.random();
            if r < p {
                let v = if rng.random::<bool>() {255u8} else {0u8};
                buf.put_pixel(x,y,Rgba([v,v,v,255]));
            }
        }
    }
    DynamicImage::ImageRgba8(buf)
}

// Zoom NN 2×
fn zoom_nn(img: &DynamicImage) -> DynamicImage {
    let (w,h) = img.dimensions();
    let (nw,nh) = (w*2, h*2);
    let mut buf = ImageBuffer::new(nw,nh);
    for y in 0..nh {
        for x in 0..nw {
            buf.put_pixel(x,y, img.get_pixel(x/2,y/2));
        }
    }
    DynamicImage::ImageRgba8(buf)
}

// Zoom bilinear 2×
fn zoom_bilinear(img: &DynamicImage) -> DynamicImage {
    let (w,h) = img.dimensions();
    let (nw,nh) = ((w as f32*2.0) as u32, (h as f32*2.0) as u32);
    let mut buf = ImageBuffer::new(nw,nh);
    for y in 0..nh {
        for x in 0..nw {
            let fx = x as f32/2.0;
            let fy = y as f32/2.0;
            let x0 = fx.floor() as u32;
            let y0 = fy.floor() as u32;
            let x1 = (x0+1).min(w-1);
            let y1 = (y0+1).min(h-1);
            let dx = fx - x0 as f32;
            let dy = fy - y0 as f32;
            let p00 = img.get_pixel(x0,y0);
            let p10 = img.get_pixel(x1,y0);
            let p01 = img.get_pixel(x0,y1);
            let p11 = img.get_pixel(x1,y1);
            let mut rgba=[0u8;4];
            for i in 0..4 {
                let v0 = p00[i] as f32*(1.0-dx) + p10[i] as f32*dx;
                let v1 = p01[i] as f32*(1.0-dx) + p11[i] as f32*dx;
                rgba[i] = (v0*(1.0-dy) + v1*dy).clamp(0.0,255.0) as u8;
            }
            buf.put_pixel(x,y,Rgba(rgba));
        }
    }
    DynamicImage::ImageRgba8(buf)
}

fn pseudo_colors(img: &DynamicImage) -> DynamicImage {
    let (w,h) = img.dimensions();
    let mut buf = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let intensity = img.get_pixel(x as u32, y as u32)[0];
            let alpha = 255;
            let new_pixel = match intensity {
                0..64 => Rgba([0,0,intensity*4,alpha]),
                64..128 => Rgba([0,(intensity-64)*4,255,alpha]),
                128..192 => Rgba([0,255,255-(intensity-128)*4,alpha]),
                _ => Rgba([(intensity-192)*4,255,0,alpha])
            };
            println!("{:?}",new_pixel);
            buf.put_pixel(x, y, new_pixel);

        }
    }
    DynamicImage::ImageRgba8(buf)
}

fn equalize_colors(img: &DynamicImage) -> DynamicImage {
    let (w,h) = img.dimensions();
    let mut buf = ImageBuffer::new(w,h);
    for y in 0..h {
        for x in 0..w {
            let pixel = img.get_pixel(x, y);
        }
    }
    DynamicImage::ImageRgb8(buf)
}

struct PDIApp {
    input: Option<DynamicImage>,
    output: Option<DynamicImage>,
    selected_algo: Algorithm,
}

impl PDIApp {
    fn new() -> Self { Self{ input: None, output: None, selected_algo: Algorithm::Blur } }
    fn load_image(&mut self, path: &Path) {
        if let Ok(mut img) = open(path) {
            let (w,h) = img.dimensions();
            if w > MAX_DIM || h > MAX_DIM {
                let scale = MAX_DIM as f32 / w.max(h) as f32;
                img = img.resize((w as f32*scale) as u32, (h as f32*scale) as u32, image::imageops::Nearest);
            }
            self.input = Some(img);
            self.output = None;
        }
    }

    fn apply_filter(&mut self) {
        if let Some(img) = &self.input {
            let res = match self.selected_algo.clone() {
                Algorithm::Blur => apply_kernel(img, [[1.0/9.0;3];3],1.0,0.0),
                Algorithm::Sharpen => apply_kernel(img, [[0.0,-1.0,0.0],[-1.0,5.0,-1.0],[0.0,-1.0,0.0]],1.0,0.0),
                Algorithm::EdgeDetect => apply_kernel(img, [[-1.0,-1.0,-1.0],[-1.0,8.0,-1.0],[-1.0,-1.0,-1.0]],1.0,0.0),
                Algorithm::Invert | Algorithm::Negative => { let mut o=img.clone(); o.invert(); o },
                Algorithm::Maximum => maximum_filter(img),
                Algorithm::Minimum => minimum_filter(img),
                Algorithm::Mean => apply_kernel(img, [[1.0/9.0;3];3],1.0,0.0),
                Algorithm::Median => median_filter(img),
                Algorithm::ZoomNN => zoom_nn(img),
                Algorithm::ZoomBilinear => zoom_bilinear(img),
                Algorithm::Grayscale => to_grayscale(img),
                Algorithm::Sobel => apply_kernel(img, [[-1.0,0.0,1.0],[-2.0,0.0,2.0],[-1.0,0.0,1.0]],1.0,0.0),
                Algorithm::Laplacian => apply_kernel(img, [[0.0,1.0,0.0],[1.0,-4.0,1.0],[0.0,1.0,0.0]],1.0,0.0),
                Algorithm::Binarize => binarize(img,128),
                Algorithm::Threshold(t) => binarize(img,t),
                Algorithm::SaltPepper(p) => salt_pepper(img,p),
                Algorithm::PseudoColors => pseudo_colors(img),
            };
            self.output = Some(res);
        }
    }
}

impl eframe::App for PDIApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Abrir").clicked() {
                    if let Some(p) = rfd::FileDialog::new().pick_file() {
                        self.load_image(&p);
                    }
                }
                if let Some(img) = &self.input {
                    let (w, h) = img.dimensions();
                    ui.label(format!("{}×{} (max {})", w, h, MAX_DIM));
                }
            });
        });

        egui::SidePanel::left("side").show(ctx, |ui| {
            ui.heading("Filtros"); ui.separator();
            ui.selectable_value(&mut self.selected_algo, Algorithm::Sharpen, "Afiar");
            ui.selectable_value(&mut self.selected_algo, Algorithm::Invert, "Inverter");
            ui.selectable_value(&mut self.selected_algo, Algorithm::Mean, "Média");
            ui.selectable_value(&mut self.selected_algo, Algorithm::Median, "Mediana");
            ui.selectable_value(&mut self.selected_algo, Algorithm::Maximum, "Maximá");
            ui.selectable_value(&mut self.selected_algo, Algorithm::Minimum, "Minimo");
            ui.selectable_value(&mut self.selected_algo, Algorithm::Grayscale, "Converter para Cinza");
            ui.selectable_value(&mut self.selected_algo, Algorithm::Sobel, "Sobel");
            ui.selectable_value(&mut self.selected_algo, Algorithm::Laplacian, "Laplaciano");
            ui.selectable_value(&mut self.selected_algo, Algorithm::Binarize, "Binarizar");
            ui.selectable_value(&mut self.selected_algo, Algorithm::Threshold(128), "Limiarização");
            ui.selectable_value(&mut self.selected_algo, Algorithm::SaltPepper(0.05), "Ruído Sal e Pimenta");
            ui.selectable_value(&mut self.selected_algo, Algorithm::ZoomNN, "Zoom NN 2×");
            ui.selectable_value(&mut self.selected_algo, Algorithm::ZoomBilinear, "Zoom Bilinear 2×");
            ui.selectable_value(&mut self.selected_algo, Algorithm::PseudoColors, "Criar cores");
            // ... outros filtros ...
            ui.separator();
            if ui.button("Aplicar").clicked() {
                self.apply_filter();
            }
            if let Some(_) = &self.output {
                if ui.button("Usar Output como Input").clicked() {
                    self.input = self.output.take();
                    self.output = None;
                }
                ui.separator();
                // Botão para salvar a imagem de saída
                if ui.button("Salvar Output...").clicked() {
                    if let Some(ref out_img) = self.output {
                        if let Some(save_path) = rfd::FileDialog::new()
                            .set_directory(".")
                            .set_file_name("output.png")
                            .save_file() {
                            // Salva como PNG
                            if let Err(e) = out_img.save(&save_path) {
                                eprintln!("Erro ao salvar a imagem: {}", e);
                            }
                        }
                    }
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(img) = &self.input {
                    ui.vertical(|ui| {
                        ui.label("Input");
                        let ci = egui::ColorImage::from_rgba_unmultiplied(
                            [img.width() as usize, img.height() as usize],
                            &img.to_rgba8(),
                        );
                        let tex = ctx.load_texture("input", ci, egui::TextureOptions::default());
                        ui.image(&tex);
                    });
                }
                if let Some(img) = &self.output {
                    ui.vertical(|ui| {
                        ui.label("Output");
                        let ci = egui::ColorImage::from_rgba_unmultiplied(
                            [img.width() as usize, img.height() as usize],
                            &img.to_rgba8(),
                        );
                        let tex = ctx.load_texture("output", ci, egui::TextureOptions::default());
                        ui.image(&tex);
                    });
                }
            });
        });
    }
}

fn main() {
    let app = PDIApp::new();
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native("Trabalho de PDI", options, Box::new(|_| Box::new(app)));
}

