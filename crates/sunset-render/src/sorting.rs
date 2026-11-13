#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TranslucentSection {
    pub index: usize,
    pub distance_sq: f32,
}

pub fn sort_translucent_back_to_front(sections: &mut [TranslucentSection]) {
    sections.sort_by(|a, b| b.distance_sq.total_cmp(&a.distance_sq));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_farthest_first() {
        let mut sections = vec![
            TranslucentSection {
                index: 0,
                distance_sq: 1.0,
            },
            TranslucentSection {
                index: 1,
                distance_sq: 9.0,
            },
            TranslucentSection {
                index: 2,
                distance_sq: 4.0,
            },
        ];
        sort_translucent_back_to_front(&mut sections);
        let order: Vec<usize> = sections.iter().map(|s| s.index).collect();
        assert_eq!(order, vec![1, 2, 0]);
    }

    #[test]
    fn empty_and_single_are_noops() {
        let mut empty: Vec<TranslucentSection> = Vec::new();
        sort_translucent_back_to_front(&mut empty);

        let mut single = vec![TranslucentSection {
            index: 3,
            distance_sq: 2.0,
        }];
        sort_translucent_back_to_front(&mut single);
        assert_eq!(single[0].index, 3);
    }
}
