//! Placement Strategy — Domain Service para posicionamento de janelas
//!
//! Implementa diferentes estratégias de posicionamento:
//! - Center: centraliza na tela
//! - Smart: minimiza overlap com janelas existentes
//! - Cascade: posiciona em cascata com offset incremental

use de_core::{Rectangle, Window};

/// Estratégia de posicionamento de janelas (Strategy Pattern)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementStrategy {
    /// Centraliza na tela
    Center,
    /// Minimiza overlap com janelas existentes (grid search)
    Smart,
    /// Posiciona em cascata (offset incremental)
    Cascade,
}

impl PlacementStrategy {
    /// Calcula posição para uma nova janela
    ///
    /// # Argumentos
    /// - `window_size`: (width, height) da janela
    /// - `screen`: Geometria da tela
    /// - `existing_windows`: Janelas já posicionadas no workspace
    ///
    /// # Retorna
    /// - `Rectangle`: Geometria calculada para a nova janela
    pub fn place(
        &self,
        window_size: (u32, u32),
        screen: &Rectangle,
        existing_windows: &[&Window],
    ) -> Rectangle {
        match self {
            Self::Center => Self::place_center(window_size, screen),
            Self::Smart => Self::place_smart(window_size, screen, existing_windows),
            Self::Cascade => Self::place_cascade(window_size, screen, existing_windows),
        }
    }

    /// Centraliza a janela na tela
    fn place_center(window_size: (u32, u32), screen: &Rectangle) -> Rectangle {
        let (width, height) = window_size;
        let x = (screen.size.width().saturating_sub(width)) / 2;
        let y = (screen.size.height().saturating_sub(height)) / 2;

        Rectangle::new(x as i32, y as i32, width, height)
    }

    /// Minimiza overlap com janelas existentes (grid search)
    fn place_smart(
        window_size: (u32, u32),
        screen: &Rectangle,
        existing_windows: &[&Window],
    ) -> Rectangle {
        let (width, height) = window_size;

        // Se não há janelas, centraliza
        if existing_windows.is_empty() {
            return Self::place_center(window_size, screen);
        }

        // Grid search: testa posições em grid de 50px
        const GRID_SIZE: i32 = 50;
        let mut best_rect = Self::place_center(window_size, screen);
        let mut min_overlap = Self::calculate_total_overlap(&best_rect, existing_windows);

        // Se já não tem overlap, retorna
        if min_overlap == 0 {
            return best_rect;
        }

        // Testar posições no grid
        let max_x = (screen.size.width() as i32) - (width as i32);
        let max_y = (screen.size.height() as i32) - (height as i32);

        for y in (0..=max_y).step_by(GRID_SIZE as usize) {
            for x in (0..=max_x).step_by(GRID_SIZE as usize) {
                let candidate = Rectangle::new(x, y, width, height);
                let overlap = Self::calculate_total_overlap(&candidate, existing_windows);

                if overlap < min_overlap {
                    min_overlap = overlap;
                    best_rect = candidate;

                    // Se encontrou posição sem overlap, retorna
                    if overlap == 0 {
                        return best_rect;
                    }
                }
            }
        }

        best_rect
    }

    /// Posiciona em cascata (cada janela deslocada 30px)
    fn place_cascade(
        window_size: (u32, u32),
        screen: &Rectangle,
        existing_windows: &[&Window],
    ) -> Rectangle {
        let (width, height) = window_size;
        const CASCADE_OFFSET: i32 = 30;

        // Calcular offset baseado no número de janelas existentes
        let offset = (existing_windows.len() as i32 * CASCADE_OFFSET) % 200;

        let x = 100 + offset;
        let y = 100 + offset;

        // Garantir que não sai da tela
        let x = x.min((screen.size.width() as i32) - (width as i32) - 50);
        let y = y.min((screen.size.height() as i32) - (height as i32) - 50);

        Rectangle::new(x, y, width, height)
    }

    /// Calcula área total de overlap com janelas existentes
    fn calculate_total_overlap(candidate: &Rectangle, existing_windows: &[&Window]) -> u32 {
        existing_windows
            .iter()
            .filter_map(|w| candidate.intersection(w.geometry))
            .map(|intersection| intersection.size.width() * intersection.size.height())
            .sum()
    }
}
