use core::fmt::Debug;

use super::VPNRange;
use super::FrameTracker;
use super::VirtPageNum;
use super::map_area::Frame;

#[cfg(feature = "oom_handler")]
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;

#[derive(Clone)]
pub struct LinearMap {
    pub vpn_range: VPNRange,
    pub frames: Vec<Frame>,
    #[cfg(feature = "oom_handler")]
    pub active: VecDeque<u16>,
    #[cfg(feature = "oom_handler")]
    pub compressed: usize,
    #[cfg(feature = "oom_handler")]
    pub swapped: usize,
}

impl LinearMap {
    pub fn new(vpn_range: VPNRange) -> Self {
        let len = vpn_range.get_end().0 - vpn_range.get_start().0;
        let mut new_dict = Self {
            vpn_range,
            frames: Vec::with_capacity(len),
            #[cfg(feature = "oom_handler")]
            active: VecDeque::new(),
            #[cfg(feature = "oom_handler")]
            compressed: 0,
            #[cfg(feature = "oom_handler")]
            swapped: 0,
        };
        new_dict.frames.resize(len, Frame::Unallocated);
        new_dict
    }
    pub fn get_mut(&mut self, key: &VirtPageNum) -> &mut Frame {
        &mut self.frames[key.0 - self.vpn_range.get_start().0]
    }
    /// # Warning
    /// a key which exceeds the end of `vpn_range` would cause panic
    pub fn get_in_memory(&self, key: &VirtPageNum) -> Option<&Arc<FrameTracker>> {
        match &self.frames[key.0 - self.vpn_range.get_start().0] {
            Frame::InMemory(tracker) => Some(tracker),
            _ => None,
        }
    }
    /// # Warning
    /// a key which exceeds the end of `vpn_range` would cause panic
    pub fn alloc_in_memory(&mut self, key: VirtPageNum, value: Arc<FrameTracker>) {
        let idx = key.0 - self.vpn_range.get_start().0;
        #[cfg(feature = "oom_handler")]
        self.active.push_back(idx as u16);
        self.frames[idx].insert_in_memory(value).unwrap()
    }
    /// # Warning
    /// a key which exceeds the end of `vpn_range` would cause panic
    pub fn remove_in_memory(&mut self, key: &VirtPageNum) -> Option<Arc<FrameTracker>> {
        let idx = key.0 - self.vpn_range.get_start().0;
        #[cfg(feature = "oom_handler")]
        self.active.retain(|&elem| elem as usize != idx);
        self.frames[idx].take_in_memory()
    }
    // /// # Warning
    // /// a key which exceeds the end of `vpn_range` would cause panic
    pub fn set_start(&mut self, new_vpn_start: VirtPageNum) -> Result<(), ()> {
        let vpn_start = self.vpn_range.get_start();
        let vpn_end = self.vpn_range.get_end();
        if new_vpn_start > vpn_end {
            return Err(());
        }
        self.vpn_range = VPNRange::new(new_vpn_start, vpn_end);
        if new_vpn_start < vpn_start {
            self.frames.rotate_left(vpn_start.0 - new_vpn_start.0);
        } else {
            self.frames.rotate_left(new_vpn_start.0 - vpn_start.0);
        }
        self.frames
            .resize(vpn_end.0 - new_vpn_start.0, Frame::Unallocated);
        Ok(())
    }
    pub fn set_end(&mut self, new_vpn_end: VirtPageNum) -> Result<(), ()> {
        let vpn_start = self.vpn_range.get_start();
        self.vpn_range = VPNRange::new(vpn_start, new_vpn_end);
        if vpn_start > new_vpn_end {
            return Err(());
        }
        self.frames
            .resize(new_vpn_end.0 - vpn_start.0, Frame::Unallocated);
        Ok(())
    }
    #[inline(always)]
    pub fn into_two(&mut self, cut: VirtPageNum) -> Result<Self, ()> {
        let vpn_start = self.vpn_range.get_start();
        let vpn_end = self.vpn_range.get_end();
        if cut <= vpn_start || cut >= vpn_end {
            return Err(());
        }
        let second_frames = self.frames.split_off(cut.0 - vpn_start.0);

        #[cfg(feature = "oom_handler")]
        let ((first_active, second_active), (first_compressed, first_swapped)) = (
            LinearMap::split_active_into_two(&self.active, cut.0 - vpn_start.0),
            self.count_compressed_and_swapped(0, cut.0 - vpn_start.0),
        );

        let second = LinearMap {
            vpn_range: VPNRange::new(cut, vpn_end),
            frames: second_frames,
            #[cfg(feature = "oom_handler")]
            active: second_active,
            #[cfg(feature = "oom_handler")]
            compressed: self.compressed - first_compressed,
            #[cfg(feature = "oom_handler")]
            swapped: self.swapped - first_swapped,
        };

        self.vpn_range = VPNRange::new(vpn_start, cut);

        #[cfg(feature = "oom_handler")]
        {
            self.active = first_active;
            self.compressed = first_compressed;
            self.swapped = first_swapped;
        }
        Ok(second)
    }
    pub fn into_three(
        &mut self,
        first_cut: VirtPageNum,
        second_cut: VirtPageNum,
    ) -> Result<(Self, Self), ()> {
        if let Ok(mut second) = self.into_two(first_cut) {
            if let Ok(third) = second.into_two(second_cut) {
                return Ok((second, third));
            }
        }
        return Err(());
    }
}
#[cfg(feature = "oom_handler")]
impl LinearMap {
    fn count_compressed_and_swapped(&self, start: usize, end: usize) -> (usize, usize) {
        if self.compressed == 0 && self.swapped == 0 {
            (0, 0)
        } else {
            self.frames[start..end].iter().fold(
                (0, 0),
                |(compressed, swapped), frame| match frame {
                    Frame::Compressed(_) => (compressed + 1, swapped),
                    Frame::SwappedOut(_) => (compressed, swapped + 1),
                    _ => (compressed, swapped),
                },
            )
        }
    }
    fn split_active_into_two(
        active: &VecDeque<u16>,
        cut_idx: usize,
    ) -> (VecDeque<u16>, VecDeque<u16>) {
        if active.is_empty() {
            (VecDeque::new(), VecDeque::new())
        } else {
            active.iter().fold(
                (VecDeque::new(), VecDeque::new()),
                |(mut first_active, mut second_active), &idx| {
                    if (idx as usize) < cut_idx {
                        first_active.push_back(idx);
                    } else {
                        second_active.push_back(idx - cut_idx as u16);
                    }
                    (first_active, second_active)
                },
            )
        }
    }
    #[allow(unused)]
    fn split_active_into_three(
        active: &VecDeque<u16>,
        first_cut_idx: usize,
        second_cut_idx: usize,
    ) -> (VecDeque<u16>, VecDeque<u16>, VecDeque<u16>) {
        assert!(first_cut_idx < second_cut_idx);
        if active.is_empty() {
            (VecDeque::new(), VecDeque::new(), VecDeque::new())
        } else {
            active.iter().fold(
                (VecDeque::new(), VecDeque::new(), VecDeque::new()),
                |(mut first_active, mut second_active, mut third_active), &idx| {
                    if (idx as usize) < first_cut_idx {
                        first_active.push_back(idx);
                    } else if (idx as usize) < second_cut_idx {
                        second_active.push_back(idx - first_cut_idx as u16);
                    } else {
                        third_active.push_back(idx - second_cut_idx as u16)
                    }
                    (first_active, second_active, third_active)
                },
            )
        }
    }
}

impl Debug for LinearMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(feature = "oom_handler")]
        return f
            .debug_struct("LinearMap")
            .field("vpn_range", &self.vpn_range)
            .field("active", &self.active.len())
            .field("compressed", &self.compressed)
            .field("swapped", &self.swapped)
            .finish();
        #[cfg(not(feature = "oom_handler"))]
        return f
            .debug_struct("LinearMap")
            .field("vpn_range", &self.vpn_range)
            .field("frames", &self.frames)
            .finish();
    }
}