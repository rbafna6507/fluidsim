use std::ops::Deref;

use crate::sim::CellType::Solid;
use crate::sim::CellType::Water;
use rand::Rng;
use rand::RngExt;

const GRAVITY: f32 = -9.8; // in m/s^2
const DT: f32 = 0.03; // DT should change to be actual time per step
const PARTICLE_RADIUS: f32 = 0.0001; // measured in cell units

#[derive(Debug, Default, Clone, Copy)]
enum CellType {
    Solid,
    #[default] Air,
    Water,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Velocity {
    u: f32,
    v: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Particle {
    vel: Velocity,
    pos: Position,
}

impl Particle {
    fn get_remainders(self) -> [f32; 2] {
        [
            self.pos.x - self.pos.x.floor(),
            self.pos.y - self.pos.y.floor(),
        ]
    }
}

/// Convert a position measured in cell units to its zero-based logical cell.
/// For example, (8.2, 21.4) maps to [8, 21] in a grid at least 9 by 22 cells.
/// Positions outside the logical grid return None.
pub fn position_to_grid_cell(x: f32, y: f32, width: usize, height: usize) -> Option<[usize; 2]> {
    if !x.is_finite()
        || !y.is_finite()
        || x < 0.0
        || y < 0.0
        || x >= width as f32
        || y >= height as f32
    {
        return None;
    }

    Some([x.floor() as usize, y.floor() as usize])
}


#[derive(Debug, Default, Clone, Copy)]
pub struct Cell {
    cell_type: CellType,
    // eventually will need pressure

    // 
}

pub struct FluidSimulation {
    pixels_width: usize,
    pixels_height: usize,

    padded_w: usize,
    padded_h: usize,

    // ten minute physics uses a linear array and math to determine what indexes correspond to where on the grid
    // i want to do a 2d vector that contains the tyep of block (boundary, air, water) and the velocities
    // Block can have helpers that do grid operations
    grid: Vec<Vec<Cell>>,
    u: Vec<f32>,
    v: Vec<f32>,
    particles: Vec<Particle>,
}

impl FluidSimulation {

    pub fn new(pixels_width: usize, pixels_height: usize, num_particles: usize) -> FluidSimulation {

        let padded_w = pixels_width + 2;
        let padded_h = pixels_height + 2;
        // add two on each side so we can create boundary cells
        // i will have a 64x32 led matrix, so we technically want a 66 x 34 in software so the other 2 rows and columns can be boundary
        let mut grid: Vec<Vec<Cell>> = vec![vec![Cell {cell_type: CellType::Air}; padded_w]; padded_h];

        // dont understand u and v really.
        let mut u = vec![0.0; (padded_h) * (padded_w+1)];
        let mut v = vec![0.0; (padded_h + 1) * (padded_w)];
        let mut particles = vec![Particle::default(); num_particles];

        let last_row = grid.len() - 1;
        // create boundary cells
        for j in 0..padded_w {
            grid[0][j] = Cell { cell_type: Solid };
            grid[last_row][j] = Cell { cell_type: Solid };
        }

        let last_col = grid[0].len() - 1;
        for i in 0..padded_h {
            grid[i][0] = Cell { cell_type: Solid };
            grid[i][last_col] = Cell { cell_type: Solid };
        }

        // initialize the particles
        for i in 0..num_particles {
            let particle_x = rand::rng().random_range(8.0..24.0);
            let particle_y = rand::rng().random_range(20.0..28.0);
            let p = Particle {
                vel: Velocity { u: 0.0, v: 0.0 },
                pos: Position {
                    x: particle_x,
                    y: particle_y,
                },
            };
            particles[i] = p;

            let [col, row] = position_to_grid_cell(p.pos.x, p.pos.y, pixels_width, pixels_height)
                .expect("spawned particle must be inside the logical grid");
            // particle coordinates are zero-based logical cells; grid has a solid border at 0.
            grid[row + 1][col + 1].cell_type = Water;
        }

        FluidSimulation {
            pixels_width,
            pixels_height,
            padded_w,
            padded_h,
            grid,
            u,
            v,
            particles,
        }
    }

    fn step(&mut self) {
        // define all the steps in the fluid sim

        // add forces
        self.apply_forces();
        // update cells as water/air (set all cells air, then iterate over particles and mark the ones with particles as water)

        // self.update_cell_state();

        // apply_particles_to_grid (returns the new grid, keeps old grid in self)
        self.apply_particles_to_grid();

        // incompressibility (uses the new)

        // particles + (new grid - old grid)
        // apply grid_to_particles(takes in the new grid)

        // set grid = new_grid

        // display
    }

    fn apply_forces(&mut self) {
        for particle in &mut self.particles {
            // no current forces on the u component (can grab the forces from accelerometer)
            particle.vel.v += GRAVITY * DT;

            // The logical domain runs from 0 to width/height; keep the particle center
            // one radius inside each wall. If it hits a wall, remove outward normal speed.
            let min_x = PARTICLE_RADIUS;
            let max_x = self.pixels_width as f32 - PARTICLE_RADIUS;
            let next_x = particle.pos.x + particle.vel.u * DT;
            particle.pos.x = next_x.clamp(min_x, max_x);
            if (next_x < min_x && particle.vel.u < 0.0) || (next_x > max_x && particle.vel.u > 0.0)
            {
                particle.vel.u = 0.0;
            }

            let min_y = PARTICLE_RADIUS;
            let max_y = self.pixels_height as f32 - PARTICLE_RADIUS;
            let next_y = particle.pos.y + particle.vel.v * DT;
            particle.pos.y = next_y.clamp(min_y, max_y);
            if (next_y < min_y && particle.vel.v < 0.0) || (next_y > max_y && particle.vel.v > 0.0)
            {
                particle.vel.v = 0.0;
            }
        }
    }

    fn apply_particles_to_grid(&mut self) -> [Vec<f32>; 2] {

        // calculate weights to apply to the transfer
        let mut u_sums = vec![0.0; (self.padded_h) * (self.padded_w+1)];
        let mut u_weight_sums = vec![0.0; (self.padded_h) * (self.padded_w+1)];

        let mut v_sums = vec![0.0; (self.padded_h + 1) * (self.padded_w)];
        let mut v_weight_sums = vec![0.0; (self.padded_h + 1) * (self.padded_w)];


        for particle in & self.particles {
            let u_weights = self.get_u_weights(particle);
            let v_weights = self.get_v_weights(particle);

            let u_idxs = self.get_u_stencil(particle);
            let v_idxs = self.get_v_stencil(particle);

            for (&idx, &weight) in u_idxs.iter().zip(u_weights.iter()) {
                u_sums[idx] += particle.vel.u * weight;
                u_weight_sums[idx] += weight;
            }

            for (&idx, &weight) in v_idxs.iter().zip(v_weights.iter()) {
                v_sums[idx] += particle.vel.v * weight;
                v_weight_sums[idx] += weight;
            }
        }

        for i in 0..self.u.len() {
            if u_weight_sums[i] > 0.0 {
                self.u[i] = u_sums[i] / u_weight_sums[i];
            }
        }

        for i in 0..self.v.len() {
            if v_weight_sums[i] > 0.0 {
                self.v[i] = v_sums[i] / v_weight_sums[i];
            }
        }

        let old_u = self.u.clone();
        let old_v = self.v.clone();

        [old_u, old_v]

    }

    // fn ensure_incompressibility()

    fn apply_grid_to_particles(&mut self, old_u: Vec<f32>, old_v: Vec<f32>) {
        
        // roughly:
        // for particle, get its grid location
        // probably, get its weights as well
        // set the particle q = sum(qi * wi)/sum(wi)
        // if a q is not defined (eg. air, do not include it into the sum)

        for i in 0..self.particles.len() {
            let particle = self.particles[i];

            let u_weights = self.get_u_weights(&particle);
            let v_weights = self.get_v_weights(&particle);

            let u_idxs = self.get_u_stencil(&particle);
            let v_idxs = self.get_v_stencil(&particle);

            let mut pic_particle_u_sum = 0.0;
            let mut particle_u_weight_sum = 0.0;

            let mut pic_particle_v_sum = 0.0;
            let mut particle_v_weight_sum = 0.0;

            let mut flip_particle_u_sum = 0.0;
            let mut flip_particle_v_sum = 0.0;

            // PIC: compute and store sum(u_i * u_weight_i) and sum (u_weight_i), for all defined u
            for (&idx, &weight) in u_idxs.iter().zip(u_weights.iter()) {
                // skip if the u value is -1 (eg. air)
                if self.u[idx] == -1.0 {
                    continue;
                }
                pic_particle_u_sum += self.u[idx] * weight;
                flip_particle_u_sum += (self.u[idx] - old_u[idx]) * weight;
                particle_u_weight_sum += weight;
            }

            // PIC: compute and store sum(v_i * v_weight_i) and sum(v_weight_i) for all defined v
            for (&idx, &weight) in v_idxs.iter().zip(v_weights.iter()) {
                // skip if the u value is -1 (eg. air)
                if self.v[idx] == -1.0 {
                    continue;
                }
                pic_particle_v_sum += self.v[idx] * weight;
                flip_particle_v_sum += (self.v[idx] - old_v[idx]) * weight;
                particle_v_weight_sum += weight;
            }

            let pic_u = pic_particle_u_sum / particle_u_weight_sum;
            let pic_v = pic_particle_v_sum / particle_v_weight_sum;

            let flip_u = flip_particle_u_sum / particle_u_weight_sum;
            let flip_v = flip_particle_v_sum / particle_v_weight_sum;

            self.particles[i].vel.u += 0.1 * pic_u + 0.9 * flip_u;
            self.particles[i].vel.v += 0.1 * pic_v + 0.9 * flip_v;

        }

    }

    pub fn get_u_weights(&self, particle: &Particle) -> [f32; 4] {
        let h = 1.0;
        let x = (particle.pos.x / h);
        let y = (particle.pos.y / h) - 0.5;

        let floored_x = x.floor();
        let floored_y = y.floor();

        let diff_x = x - floored_x;
        let diff_y = y - floored_y;

        [(1.0-diff_x)*(1.0-diff_y), diff_x*(1.0-diff_y), diff_x*diff_y, (1.0-diff_x) * diff_y]

    }

    pub fn get_v_weights(&self, particle: &Particle) -> [f32; 4] {
        let h = 1.0;
        let x = (particle.pos.x / h) - 0.5;
        let y = (particle.pos.y / h);

        let floored_x = x.floor();
        let floored_y = y.floor();

        let diff_x = x - floored_x;
        let diff_y = y - floored_y;

        [(1.0-diff_x)*(1.0-diff_y), diff_x*(1.0-diff_y), diff_x*diff_y, (1.0-diff_x) * diff_y]

    }

    pub fn get_u_stencil(&self, particle: &Particle) -> [usize; 4] {
        let h: f32 = 1.0;
        // should return 4 u array indicies
        let x = (particle.pos.x / h);
        let y = (particle.pos.y / h) - 0.5;

        let i0 = x.floor() as isize;
        let j0 = y.floor() as isize;

        let i1 = i0 + 1;
        let j1 = j0 + 1;

        [self.get_u_idx(i0, j0), self.get_u_idx(i1, j0), self.get_u_idx(i1, j1), self.get_u_idx(i0, j1)]

    }

    pub fn get_v_stencil(&self, particle: &Particle) -> [usize; 4] {
        let h = 1.0;
        let x = (particle.pos.x / h) - 0.5;
        let y = (particle.pos.y / h);

        let i0 = x.floor() as isize;
        let j0 = y.floor() as isize;

        let i1 = i0 + 1;
        let j1 = j0 + 1;

        [self.get_v_idx(i0, j0), self.get_v_idx(i1, j0), self.get_v_idx(i1, j1), self.get_v_idx(i0, j1)]

    }

    pub fn get_u_idx(&self, i: isize, j: isize) -> usize {
        let padded_i = (i + 1) as usize;
        let padded_j = (j + 1) as usize;
        padded_j * (self.padded_w + 1) + padded_i

    }

    pub fn get_v_idx(&self, i: isize, j:isize) -> usize {
        let padded_i = (i + 1) as usize;
        let padded_j = (j + 1) as usize;
        padded_j * self.padded_w + padded_i
    }

    // return cell face velocities on the staggered grid. vertical components of u and v
    // don't really understand cell face velocities
    pub fn cell_face_velocities(&self, col: usize, row: usize) -> Option<CellFaceVelocities> {
        if col >= self.pixels_width || row >= self.pixels_height {
            return None;
        }

        let padded_col = col + 1;
        let padded_row = row + 1;
        let u_row_width = self.pixels_width + 3;
        let v_row_width = self.pixels_width + 2;

        let left_u_index = padded_row * u_row_width + padded_col;
        let right_u_index = left_u_index + 1;
        let bottom_v_index = padded_row * v_row_width + padded_col;
        let top_v_index = (padded_row + 1) * v_row_width + padded_col;

        Some(CellFaceVelocities {
            left_u: self.u[left_u_index],
            right_u: self.u[right_u_index],
            bottom_v: self.v[bottom_v_index],
            top_v: self.v[top_v_index],
        })
    }
}
