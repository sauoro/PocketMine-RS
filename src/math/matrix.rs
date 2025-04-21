// src/math/matrix.rs

#![allow(dead_code)]

use std::{fmt, ops::{Index, IndexMut}};

#[derive(Clone, Debug)]
pub struct Matrix {
    data: Vec<Vec<f64>>,
    rows: usize,
    columns: usize,
}

impl Matrix {
    pub fn new(rows: usize, columns: usize, initial_data: Option<&[Vec<f64>]>) -> Self {
        let r = rows.max(1);
        let c = columns.max(1);
        let mut data = vec![vec![0.0; c]; r];

        if let Some(init) = initial_data {
            for ri in 0..r.min(init.len()) {
                for ci in 0..c.min(init[ri].len()) {
                    data[ri][ci] = init[ri][ci];
                }
            }
        }
        Self { data, rows: r, columns: c }
    }

    pub fn set(&mut self, new_data: &[Vec<f64>]) {
        // Reinitialize data grid and copy values
        self.data = vec![vec![0.0; self.columns]; self.rows];
        for r in 0..self.rows.min(new_data.len()) {
            for c in 0..self.columns.min(new_data[r].len()) {
                self.data[r][c] = new_data[r][c];
            }
        }
    }

    pub fn rows(&self) -> usize { self.rows }
    pub fn columns(&self) -> usize { self.columns }

    pub fn set_element(&mut self, row: usize, column: usize, value: f64) -> Result<(), String> {
        if row >= self.rows || column >= self.columns {
            Err(format!("Row or column out of bounds (have {} rows {} columns, tried {}x{})", self.rows, self.columns, row, column))
        } else {
            self.data[row][column] = value;
            Ok(())
        }
    }

    pub fn get_element(&self, row: usize, column: usize) -> Result<f64, String> {
        if row >= self.rows || column >= self.columns {
            Err(format!("Row or column out of bounds (have {} rows {} columns, tried {}x{})", self.rows, self.columns, row, column))
        } else {
            Ok(self.data[row][column])
        }
    }

    pub fn is_square(&self) -> bool {
        self.rows == self.columns
    }

    pub fn add(&self, other: &Matrix) -> Result<Matrix, String> {
        if self.rows != other.rows || self.columns != other.columns {
            return Err("Matrix does not have the same number of rows and/or columns".to_string());
        }
        let mut result = Matrix::new(self.rows, self.columns, None);
        for r in 0..self.rows {
            for c in 0..self.columns {
                result.data[r][c] = self.data[r][c] + other.data[r][c];
            }
        }
        Ok(result)
    }

    pub fn subtract(&self, other: &Matrix) -> Result<Matrix, String> {
        if self.rows != other.rows || self.columns != other.columns {
            return Err("Matrix does not have the same number of rows and/or columns".to_string());
        }
        let mut result = Matrix::new(self.rows, self.columns, None);
        for r in 0..self.rows {
            for c in 0..self.columns {
                result.data[r][c] = self.data[r][c] - other.data[r][c];
            }
        }
        Ok(result)
    }

    pub fn multiply_scalar(&self, number: f64) -> Matrix {
        let mut result = self.clone();
        for r in 0..self.rows {
            for c in 0..self.columns {
                result.data[r][c] *= number;
            }
        }
        result
    }

    pub fn divide_scalar(&self, number: f64) -> Matrix {
        let mut result = self.clone();
        for r in 0..self.rows {
            for c in 0..self.columns {
                result.data[r][c] /= number;
            }
        }
        result
    }

    pub fn transpose(&self) -> Matrix {
        let mut result = Matrix::new(self.columns, self.rows, None);
        for r in 0..self.rows {
            for c in 0..self.columns {
                result.data[c][r] = self.data[r][c];
            }
        }
        result
    }

    pub fn product(&self, other: &Matrix) -> Result<Matrix, String> {
        if self.columns != other.rows {
            return Err(format!("Expected a matrix with {} rows, but got {}", self.columns, other.rows));
        }
        let result_columns = other.columns;
        let mut result = Matrix::new(self.rows, result_columns, None);

        for i in 0..self.rows {
            for j in 0..result_columns {
                let mut sum = 0.0;
                for k in 0..self.columns {
                    sum += self.data[i][k] * other.data[k][j];
                }
                result.data[i][j] = sum;
            }
        }
        Ok(result)
    }

    pub fn determinant(&self) -> Result<f64, String> {
        if !self.is_square() {
            return Err("Cannot calculate determinant of a non-square matrix".to_string());
        }
        match self.rows {
            1 => Ok(self.data[0][0]),
            2 => Ok(self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0]),
            3 => Ok(
                self.data[0][0] * self.data[1][1] * self.data[2][2] +
                    self.data[0][1] * self.data[1][2] * self.data[2][0] +
                    self.data[0][2] * self.data[1][0] * self.data[2][1] -
                    self.data[2][0] * self.data[1][1] * self.data[0][2] -
                    self.data[2][1] * self.data[1][2] * self.data[0][0] -
                    self.data[2][2] * self.data[1][0] * self.data[0][1]
            ),
            _ => Err("Determinant calculation not implemented for this size".to_string())
        }
    }
}

impl Index<usize> for Matrix {
    type Output = [f64];
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for Matrix {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for r in 0..self.rows {
            s.push_str(&self.data[r].iter().map(|&v| v.to_string()).collect::<Vec<String>>().join(","));
            s.push(';');
        }
        if !s.is_empty() {
            s.pop(); // Remove trailing semicolon
        }
        write!(f, "Matrix({}x{};{})", self.rows, self.columns, s)
    }
}