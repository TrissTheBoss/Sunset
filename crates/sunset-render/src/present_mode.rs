use wgpu::PresentMode;

pub fn select_present_mode(supported: &[PresentMode]) -> PresentMode {
    const ORDER: [PresentMode; 3] = [
        PresentMode::Immediate,
        PresentMode::Mailbox,
        PresentMode::Fifo,
    ];
    for mode in ORDER {
        if supported.contains(&mode) {
            return mode;
        }
    }
    PresentMode::Fifo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_immediate() {
        let supported = [
            PresentMode::Fifo,
            PresentMode::Mailbox,
            PresentMode::Immediate,
        ];
        assert_eq!(select_present_mode(&supported), PresentMode::Immediate);
    }

    #[test]
    fn falls_back_to_mailbox() {
        let supported = [PresentMode::Fifo, PresentMode::Mailbox];
        assert_eq!(select_present_mode(&supported), PresentMode::Mailbox);
    }

    #[test]
    fn falls_back_to_fifo() {
        let supported = [PresentMode::Fifo];
        assert_eq!(select_present_mode(&supported), PresentMode::Fifo);
    }

    #[test]
    fn empty_slice_defaults_to_fifo() {
        assert_eq!(select_present_mode(&[]), PresentMode::Fifo);
    }
}
