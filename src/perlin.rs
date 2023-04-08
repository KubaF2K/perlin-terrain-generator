use ndarray::Array2;
use rand::{SeedableRng, Rng};
use rand_chacha::ChaCha8Rng;

//funkcja wygładzająca
fn fade(x: f64) -> f64 { (6.0*x.powi(5)) - (15.0*x.powi(4)) + (10.0*x.powi(3)) }

//funkcja interpolująca
fn lerp(t: f64, a1: f64, a2: f64) -> f64 { (1.0-t) * a1 + t * a2 }

//struktura szumu perlina
pub struct Perlin {
    //ilość chunków w osi x
    chunks_x: usize,
    //ilość chunków w osi y
    chunks_y: usize,
    //długość boku kwadratowego chunka
    chunk_size: usize,
    //surowe wartości
    intensities: Array2<f64>
}
impl Perlin {
    pub fn new(
        chunks_x: usize,
        chunks_y: usize,
        chunk_size: usize,
        seed: u64
    ) -> Perlin {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        //tablica wektorów na przecięciach wierzchołków chunków (jako kąt w radianach)
        let mut vectors: Array2<f64> = Array2::zeros((chunks_x+1, chunks_y+1));
        for vector in vectors.iter_mut() {
            *vector = rng.gen_range(0.0 .. (std::f64::consts::PI * 2.0)); 
        }
        let mut intensities: Array2<f64> = Array2::zeros(
            (chunks_x * chunk_size, chunks_y * chunk_size)
        );
        for x in 0..chunks_x * chunk_size {
            for y in 0..chunks_y * chunk_size {
                //współrzędne aktualnego chunka
                let chunk_x = x / chunk_size;
                let chunk_y = y / chunk_size;

                //współrzędne aktualnego piksela w chunku
                let a = x % chunk_size;
                let b = y % chunk_size;

                //wektory na wierzchołkach chunka
                let tl_vector = vectors[[chunk_x, chunk_y]];
                let tr_vector = vectors[[chunk_x+1, chunk_y]];
                let bl_vector = vectors[[chunk_x, chunk_y+1]];
                let br_vector = vectors[[chunk_x+1, chunk_y+1]];

                //iloczyny skalarne dla każdego z wierzchołków
                let tl_intensity = (-(a as f64) * tl_vector.cos()) + (-(b as f64) * tl_vector.sin());
                let tr_intensity = ((chunk_size - a) as f64 * tr_vector.cos()) + (-(b as f64) * tr_vector.sin());
                let bl_intensity = (-(a as f64) * bl_vector.cos()) + ((chunk_size - b) as f64 * bl_vector.sin());
                let br_intensity = ((chunk_size - a) as f64 * br_vector.cos()) + ((chunk_size - b) as f64 * br_vector.sin());

                //wygładzone współrzędne dla funkcji interpolującej
                let xf = fade(a as f64 / chunk_size as f64);
                let yf = fade(b as f64 / chunk_size as f64);

                //interpolacje
                let top_lerp = lerp(xf, tl_intensity, tr_intensity);
                let bottom_lerp = lerp(xf, bl_intensity, br_intensity);

                intensities[[x, y]] = lerp(yf, top_lerp, bottom_lerp);
            }
        }
        Perlin { chunks_x, chunks_y, chunk_size, intensities }
    }
    pub fn get_width(&self) -> usize { self.chunks_x * self.chunk_size }
    pub fn get_height(&self) -> usize { self.chunks_y * self.chunk_size }

    //funkcja zwracająca znormalizowane wartości
    pub fn get_normalized_intensities(&self) -> Array2<f64> {
        let mut max_intensity = self.intensities[[0, 0]];
        let mut min_intensity = max_intensity;

        for intensity in self.intensities.iter() {
            if *intensity > max_intensity {max_intensity = *intensity};
            if *intensity < min_intensity {min_intensity = *intensity};
        }

        self.intensities.map( |intensity| -> f64 {
            if min_intensity == max_intensity { 
                0.5 
            } else { 
                (*intensity - min_intensity) / (max_intensity - min_intensity) 
            }
        })
    }
    //funkcja zwracająca znormalizowane wartości jako bajt (w przedziale 0-255)
    pub fn get_brightness_pixels(&self) -> Array2<u8> {
        self.get_normalized_intensities().map(|intensity| -> u8 {
            (intensity * 255.0) as u8
        })
    }

    //funkcja generująca oktawy i dodająca je do aktualnego szumu aby uzyskać fraktalny szum
    pub fn generate_octaves(&mut self,
        //ilość oktaw
        count: u32,
        //trwałość
        persistence: f64,
        //lakunarność
        lacunarity: f64,
        //seed dla generatora liczb pseudolosowych
        seed: u64
    ) {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        //ilość chunków w oktawie
        let mut new_chunks_x = self.chunks_x;
        let mut new_chunks_y = self.chunks_y;

        //rozmiar chunków w oktawie
        let mut new_chunk_size = self.chunk_size;

        //intensywność oktawy
        let mut strength = 1.0;

        for _ in 0..count {
            //powiększenie ilości chunków o lakunarność
            new_chunks_x = (new_chunks_x as f64 * lacunarity) as usize;
            new_chunks_y = (new_chunks_y as f64 * lacunarity) as usize;

            //pomniejszenie rozmiaru chunków o lakunarność
            new_chunk_size = (new_chunk_size as f64 / lacunarity) as usize;

            //przerywanie generowania w przypadku zbyt drobnych chunków
            if new_chunk_size == 0 { break }

            //przemnożenie intensywności o trwałość
            strength *= persistence;

            //w przypadku gdy oktawa przez błędy zaokrąglenia okaże się
            //mniejsza niż szum podstawowy, zwiększanie ilości chunków
            while new_chunks_x * new_chunk_size < self.chunks_x * self.chunk_size {
                new_chunks_x += 1;
            }
            while new_chunks_y * new_chunk_size < self.chunks_y * self.chunk_size {
                new_chunks_y += 1;
            }

            //tworzenie oktawy
            let octave = Perlin::new(new_chunks_x, new_chunks_y, new_chunk_size, rng.gen());

            //dodanie oktawy do szumu podstawowego
            self.add(&octave.intensities, strength)
        }
    }
    //funkcja dodająca podane wartości do szumu z określoną intensywnością
    fn add(&mut self, intensities: &Array2<f64>, strength: f64) {
        for x in 0..self.get_width() {
            for y in 0..self.get_height() {
                self.intensities[[x, y]] += intensities[[x, y]] * strength;
            }
        }
    }
}