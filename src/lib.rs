extern crate num_traits;

use std::ops::Index;
use std::ops::IndexMut;
use std::slice;
use std::mem;
use std::os::raw::c_void;
use num_traits::Num;

#[derive(Clone)]
struct DataRow<T: Num + Copy>
{
    padded_data: Vec<T>,
}

impl<T: Num + Copy> DataRow<T>
{
    fn new(size: usize) -> Self
    {
        let mut padded_data = Vec::new();
        for _i in 0..(size + 2)
        {
            padded_data.push(T::zero());
        }
        DataRow { padded_data: padded_data }
    }

    fn from_equation<F>(size: usize, equation: F) -> Self
        where F: Fn(usize) -> T
    {
        let mut data_row = Self::new(size);
        for i in 0..size
        {
            data_row[i] = equation(i);
        }
        data_row
    }

    fn from_slice(data_slice: &[T]) -> Self
    {
        let mut data_row = Self::new(data_slice.len());
        for i in 0..data_slice.len()
        {
            data_row[i] = data_slice[i];
        }
        data_row
    }

    fn to_raw_ptr(&mut self) -> *mut T
    {
        self.padded_data.as_mut_ptr()
    }

    fn laplace(&self, i: usize) -> T
    {
        let i = i as i32;
        // 1D laplace kernel.
        self[i - 1] - self[i] - self[i] + self[i + 1]
    }
}

impl<T: Num + Copy> Index<i32> for DataRow<T>
{
    type Output = T;
    fn index(&self, index: i32) -> &T
    {
        &self.padded_data[(index + 1) as usize]
    }
}

impl<T: Num + Copy> Index<usize> for DataRow<T>
{
    type Output = T;
    fn index(&self, index: usize) -> &T
    {
        &self[index as i32]
    }
}

impl<T: Num + Copy> IndexMut<i32> for DataRow<T>
{
    fn index_mut(&mut self, index: i32) -> &mut T
    {
        &mut self.padded_data[(index + 1) as usize]
    }
}

impl<T: Num + Copy> IndexMut<usize> for DataRow<T>
{
    fn index_mut(&mut self, index: usize) -> &mut T
    {
        &mut self[index as i32]
    }
}

#[derive(Clone)]
struct WaveState
{
    size: usize,
    pos: DataRow<f64>,
    vel: DataRow<f64>,
    damp: DataRow<f64>,
    wave_coeff: f64,
}

impl WaveState
{
    fn new(size: usize) -> Self
    {
        WaveState
        {
            size,
            pos: DataRow::new(size),
            vel: DataRow::new(size),
            damp: DataRow::new(size),
            wave_coeff: 0.0,
        }
    }

    fn delta_into(&self, next: &mut Self, dt: f64)
    {
        for i in 0..self.size
        {
            next.pos[i] = dt * self.vel[i];
            next.vel[i] = dt * self.pos.laplace(i) * self.wave_coeff;
            next.vel[i] -= dt * self.damp[i] * self.vel[i];
        }
    }

    fn copy_from(&mut self, source: &Self)
    {
        for i in 0..self.size
        {
            self.pos[i] = source.pos[i];
            self.vel[i] = source.vel[i];
        }
    }

    fn apply_delta(&mut self, delta: &Self, coeff: f64)
    {
        for i in 0..self.size
        {
            self.pos[i] += coeff * delta.pos[i];
            self.vel[i] += coeff * delta.vel[i];
        }
    }

    fn draw(&self, screen: &mut ImageBuffer)
    {
        for i in 0..self.size
        {
            let x = (i * screen.width / self.size) as i32;
            let y = (screen.height as i32) / 2i32 - (self.pos[i] * (screen.height as f64)) as i32;
            screen.put(x, y, 35, 190, 216);
        }
    }
}

pub struct WaveRk4Solver
{
    state: WaveState,
    delta1: WaveState,
    delta2: WaveState,
    delta3: WaveState,
    delta4: WaveState,
    approx1: WaveState,
    approx2: WaveState,
    approx3: WaveState,
}

impl WaveRk4Solver
{
    fn new(initial_state: WaveState) -> Self
    {
        WaveRk4Solver
        {
            state: initial_state.clone(),
            approx1: initial_state.clone(),
            approx2: initial_state.clone(),
            approx3: initial_state.clone(),
            delta1: initial_state.clone(),
            delta2: initial_state.clone(),
            delta3: initial_state.clone(),
            delta4: initial_state.clone(),
        }
    }

    fn step(&mut self, dt: f64)
    {
        self.state.delta_into(&mut self.delta1, dt);

        self.approx1.copy_from(&self.state);
        self.approx1.apply_delta(&self.delta1, 0.5);
        self.approx1.delta_into(&mut self.delta2, dt);

        self.approx2.copy_from(&self.state);
        self.approx2.apply_delta(&self.delta2, 0.5);
        self.approx2.delta_into(&mut self.delta3, dt);

        self.approx3.copy_from(&self.state);
        self.approx3.apply_delta(&self.delta3, 1.0);
        self.approx3.delta_into(&mut self.delta4, dt);

        self.state.apply_delta(&self.delta1, 1.0 / 6.0);
        self.state.apply_delta(&self.delta2, 2.0 / 6.0);
        self.state.apply_delta(&self.delta3, 2.0 / 6.0);
        self.state.apply_delta(&self.delta4, 1.0 / 6.0);
    }

    fn draw(&self, screen: &mut ImageBuffer)
    {
        self.state.draw(screen);
    }
}

struct ImageBuffer<'a>
{
    buffer: &'a mut [u8],
    width: usize,
    height: usize,
}

impl<'a> ImageBuffer<'a>
{
    fn put(&mut self, x: i32, y: i32, r: u8, g: u8, b: u8)
    {
        let width = self.width as i32;
        let height = self.height as i32;

        if x < 0 || x >= width || y < 0 || y >= height
        {
            return;
        }

        let i = (x + y * width) * 4;
        let i = i as usize;
        self.buffer[i + 0] = r;
        self.buffer[i + 1] = g;
        self.buffer[i + 2] = b;
        self.buffer[i + 3] = 255;
    }

    fn clear(&mut self)
    {
        for i in 0..(self.width * self.height)
        {
            self.buffer[4 * i + 0] = 0;
            self.buffer[4 * i + 1] = 0;
            self.buffer[4 * i + 2] = 0;
            self.buffer[4 * i + 3] = 0;
        }
    }
}

#[no_mangle]
pub extern "C" fn test_ptr(ptr: *mut c_void, size: usize) -> f64
{
    let ptr = ptr as *mut f64;
    let ptr_to_slice = unsafe { slice::from_raw_parts(ptr, size) };
    ptr_to_slice[5]
}

#[no_mangle]
pub extern "C" fn new_setup(size: usize, pos_ptr: *mut c_void, wave_coeff: f64) -> *mut WaveRk4Solver
{
    let pos_ptr = pos_ptr as *mut f64;
    let pos_slice = unsafe { slice::from_raw_parts_mut(pos_ptr, size) };

    let border = 32_usize;

    let damp = DataRow::from_equation(size,
        |x|
        {
            if x < border || x > size - border
            {
                0.1
            }
            else
            {
                0.0
            }
        });

    let pos = DataRow::from_slice(pos_slice);

    let initial_state = WaveState { damp, pos, wave_coeff, .. WaveState::new(size) };

    let solver = WaveRk4Solver::new(initial_state);
    let solver_box = Box::new(solver);

    Box::into_raw(solver_box)
}

#[no_mangle]
pub extern "C" fn step(solver_ptr: *mut WaveRk4Solver, duration: f64, divides: i32)
{
    let mut solver = unsafe { &mut *solver_ptr };

    let dt = duration / (divides as f64);

    for _i in 0..divides
    {
        solver.step(dt);
    }

}

#[no_mangle]
pub extern "C" fn get_pos(solver_ptr: *mut WaveRk4Solver) -> *mut f64
{
    let mut solver = unsafe { &mut *solver_ptr };
    solver.state.pos.to_raw_ptr()
}

#[no_mangle]
pub extern "C" fn draw(solver_ptr: *mut WaveRk4Solver, buffer_ptr: *mut u8, width: usize, height: usize)
{
    let mut solver = unsafe { &mut *solver_ptr };

    let buffer_bytes = width * height * 4;
    let buffer = unsafe { slice::from_raw_parts_mut(buffer_ptr, buffer_bytes) };

    let mut screen = ImageBuffer { buffer, width, height };
    screen.clear();
    solver.draw(&mut screen);
}

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void
{
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}
