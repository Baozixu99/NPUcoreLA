use bit_field::BitField;

use crate::arch::board::ACPI_BASE;

const PM1_CNT_ADDR: usize = ACPI_BASE + 0x14;

impl_define_mem_reg!(
    Pm1Cnt,
    PM1_CNT_ADDR,
    "电源管理1控制寄存器,<br>
    Power Management 1 Control Register "
);

impl Pm1Cnt {
    impl_get_set!(get_slp_en, set_slp_en, 13,
        "该位写1将会使系统进入SLP_TYP声明的休眠状态，进入相关休眠状态后该位自动恢复为0");
    impl_get_set!(get_slp_typ, set_slp_typ, 10..=12,
        "该3bit表示系统的休眠状态");
    /// 将系统设置为s5状态
    pub fn set_s5(&mut self) -> &mut Self{
        self.set_slp_typ(SleepType::S5.into());
        self.set_slp_en(true);
        self
    }
}

#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive, Debug)]
#[repr(usize)]
/// 该3bit表示系统的休眠状态
pub enum SleepType {
    /// 该模式下系统全部工作
    S0 = 0b000,
    /// Suspend to RAM(STR)，上下文保存到内存
    S3 = 0b101,
    /// Suspend to Disk(STD)，保存到硬盘，除唤醒电路全部掉电
    S4 = 0b110,
    /// Soft off，只有唤醒电路上电，“软关机”
    S5 = 0b111,
}