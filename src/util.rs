use std::*;
use fmt::*;

pub trait Array2DShow: Default + Clone + Display {}
impl<T: Default + Clone + Display> Array2DShow for T {}

#[derive(Debug)]
pub struct Flat2DArray<T: Array2DShow = u8> {
    width: usize,
    pub data: Vec<T>
}

impl<T> Flat2DArray<T> where T: Array2DShow {

    pub fn new(width: usize, height: usize) -> Flat2DArray<T> {
        Flat2DArray {
            width,
            data: vec![T::default(); width * height]
        }
    }

    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.data[x * self.width + y]
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.data[x * self.width + y] = value;
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