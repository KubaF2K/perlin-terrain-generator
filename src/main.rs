mod perlin;

use image::{Rgb, RgbImage};
use ndarray::Array2;
use rand::{random, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use crate::perlin::Perlin;

//Zamiana na obraz surowej heightmapy
fn perlin_to_img(perlin: &Perlin) -> RgbImage {
    let mut img: RgbImage = RgbImage::new(perlin.get_width() as u32, perlin.get_height() as u32);

    let pixels: Array2<u8> = perlin.get_brightness_pixels();

    for x in 0..perlin.get_width() {
        for y in 0..perlin.get_height() {
            img.put_pixel(x as u32, y as u32, Rgb::from([pixels[[x, y]]; 3]));
        }
    }

    img
}

//Zamiana na obraz terenu
//noise_strength wpływa na ilość dodanego szumu z heightmapy
fn perlin_to_terrain(perlin: &Perlin, noise_strength: u32) -> RgbImage {
    let mut img: RgbImage = RgbImage::new(
        perlin.get_width() as u32, perlin.get_height() as u32
    );

    let intensities: Array2<f64> = perlin.get_normalized_intensities();

    for x in 0..perlin.get_width() {
        for y in 0..perlin.get_height() {
            let value = intensities[[x, y]];
            //zakresy i kolory dla danego zakresu
            let color = match value {
                x if x < 0.3 => [20, 20, 200],
                x if x < 0.43 => [200, 200, 100],
                x if x < 0.66 => [20, 200, 20],
                x if x < 0.82 => [20, 150, 20],
                x if x < 0.86 => [90, 90, 90],
                x if x < 0.9 => [120, 120, 100],
                _ => [200, 200, 200]
            }.map(|shade| -> u8 {
                //dodanie szumu i ograniczenie do zakresu 0-255
                (shade + ((value - 0.5) * noise_strength as f64) as i32).min(255).max(0) as u8
            });
            img.put_pixel(x as u32, y as u32, Rgb::from(color));
        }
    }

    img
}

fn main() {
    //ilość chunków w osi x
    let width = 5;
    
    //ilość chunków w osi y
    let height = 5;

    //długość boku kwadratowego chunka
    let chunk_size = 100;

    //seed dla generatora liczb pseudolosowych
    let seed = random();

    //generator liczb pseudolosowych
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    //struktura szumu perlina
    let mut perlin = Perlin::new(width, height, chunk_size, rng.gen());

    //zwykły szum perlina
    let img: RgbImage = perlin_to_img(&perlin);
    match img.save("perlin.png") {
        Ok(_) => (),
        Err(asd) => println!("Błąd zapisu obrazu: {}", asd)
    }

    //generowanie oktaw szumu fraktalnego
    perlin.generate_octaves(4, 0.75, 2.0, rng.gen());

    //fraktalny szum perlina
    let img2: RgbImage = perlin_to_img(&perlin);
    match img2.save("perlin_fractal.png") {
        Ok(_) => (),
        Err(asd) => println!("Błąd zapisu obrazu: {}", asd)
    }

    //teren bez nałożonego szumu
    let img3: RgbImage = perlin_to_terrain(&perlin, 0);
    match img3.save("raw_terrain.png") {
        Ok(_) => (),
        Err(asd) => println!("Błąd zapisu obrazu: {}", asd)
    }

    //teren z nałożonym szumem i lekkim rozmyciem
    let img4: RgbImage = image::imageops::blur(&perlin_to_terrain(&perlin, 150), 1.0);
    match img4.save("terrain.png") {
        Ok(_) => (),
        Err(asd) => println!("Błąd zapisu obrazu: {}", asd)
    }
}
