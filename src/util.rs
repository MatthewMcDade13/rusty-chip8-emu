use std::*;
use fmt::*;

pub trait Array2DShow: Default + Clone + Display {}
impl<T: Default + Clone + Display> Array2DShow for T {}

#[derive(Debug)]
pub struct Flat2DArray<T: Array2DShow = u8> {
    width: usize,
    pub data: Vec<T>
}

#[derive(Debug, Copy, Clone)]
pub struct FrameTimer {
    clock: time::Instant,
    accum: f32,
    target_delta: f32,
}

impl FrameTimer {
    pub fn new(target_delta: f32) -> Self {
        FrameTimer {
            clock: time::Instant::now(),
            accum: 0.0,
            target_delta
        }
    }

    /**
     *  Gets elapsed time since last call to elapsed() or new().
     *  Used to get time from last frame to current frame
    */
    pub fn elapsed(&mut self) -> f32 {
        let elapsed = self.clock.elapsed().as_secs_f32();
        self.clock = time::Instant::now();
        elapsed
    }

    pub fn accum_elapsed(&mut self) -> f32 {
        self.accum += self.elapsed();
        self.accum
    }

    pub fn reset(&mut self) {
        self.accum = 0.0;
        self.clock = time::Instant::now();
    }

    /**
     * Called every frame to add up the accumulator and return true
     * if we have passed enough time for an update
    */
    pub fn frame(&mut self) -> bool {
        let acc = self.accum_elapsed();
        if acc > self.target_delta {
            true
        } else {
            false
        }
    }
    
}

impl<T> Flat2DArray<T> where T: Array2DShow {

    pub fn new(width: usize, height: usize) -> Flat2DArray<T> {
        Flat2DArray {
            width,
            data: vec![T::default(); width * height]
        }
    }

    pub fn clear(&mut self) {
        for i in self.data.iter_mut() { *i = T::default(); }
    }

    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.data[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.data[y * self.width + x] = value;
    }
}

impl<T> Display for Flat2DArray<T> where T: Array2DShow {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut display = String::new();
        for (i, c) in self.data.iter().enumerate() {
            write!(display, " {} |", c)?;
            if (i + 1) % self.width == 0 {
                write!(display, "\n")?;
            }
        }
        write!(formatter, "{}", display)
    }
}
