#[derive(Debug)]
pub enum DeviceMemory {
    //         Address      Writeable bits  
    VSYNC ,   // 0x00   VSYNC   ......1.  vertical sync set-clear
    VBLANK ,   // 0x01   VBLANK  11....1.  vertical blank set-clear
    WSYNC ,   // 0x02   WSYNC   <strobe>  wait for leading edge of horizontal blank
    RSYNC ,   // 0x03   RSYNC   <strobe>  reset horizontal sync counter
    NUSIZ0 ,   // 0x04   NUSIZ0  ..111111  number-size player-missile 0
    NUSIZ1 ,   // 0x05   NUSIZ1  ..111111  number-size player-missile 1
    COLUP0 ,   // 0x06   COLUP0  1111111.  color-lum player 0 and missile 0
    COLUP1 ,   // 0x07   COLUP1  1111111.  color-lum player 1 and missile 1
    COLUPF ,   // 0x08   COLUPF  1111111.  color-lum playfield and ball
    COLUBK ,   // 0x09   COLUBK  1111111.  color-lum background
    CTRLPF ,   // 0x0A   CTRLPF  ..11.111  control playfield ball size & collisions
    REFP0 ,   // 0x0B   REFP0   ....1...  reflect player 0
    REFP1 ,   // 0x0C   REFP1   ....1...  reflect player 1
    PF0 ,   // 0x0D   PF0     1111....  playfield register byte 0
    PF1 ,   // 0x0E   PF1     11111111  playfield register byte 1
    PF2 ,   // 0x0F   PF2     11111111  playfield register byte 2
    RESP0 ,   // 0x10   RESP0   <strobe>  reset player 0
    RESP1 ,   // 0x11   RESP1   <strobe>  reset player 1
    RESM0 ,   // 0x12   RESM0   <strobe>  reset missile 0
    RESM1 ,   // 0x13   RESM1   <strobe>  reset missile 1
    RESBL ,   // 0x14   RESBL   <strobe>  reset ball
    AUDC0 ,   // 0x15   AUDC0   ....1111  audio control 0
    AUDC1 ,   // 0x16   AUDC1   ....1111  audio control 1
    AUDF0 ,   // 0x17   AUDF0   ...11111  audio frequency 0
    AUDF1 ,   // 0x18   AUDF1   ...11111  audio frequency 1
    AUDV0 ,   // 0x19   AUDV0   ....1111  audio volume 0
    AUDV1 ,   // 0x1A   AUDV1   ....1111  audio volume 1
    GRP0 ,   // 0x1B   GRP0    11111111  graphics player 0
    GRP1 ,   // 0x1C   GRP1    11111111  graphics player 1
    ENAM0 ,   // 0x1D   ENAM0   ......1.  graphics (enable) missile 0
    ENAM1 ,   // 0x1E   ENAM1   ......1.  graphics (enable) missile 1
    ENABL ,   // 0x1F   ENABL   ......1.  graphics (enable) ball
    HMP0 ,   // 0x20   HMP0    1111....  horizontal motion player 0
    HMP1 ,   // 0x21   HMP1    1111....  horizontal motion player 1
    HMM0 ,   // 0x22   HMM0    1111....  horizontal motion missile 0
    HMM1 ,   // 0x23   HMM1    1111....  horizontal motion missile 1
    HMBL ,   // 0x24   HMBL    1111....  horizontal motion ball
    VDELP0 ,   // 0x25   VDELP0  .......1  vertical delay player 0
    VDELP1 ,   // 0x26   VDELP1  .......1  vertical delay player 1
    VDELBL ,   // 0x27   VDELBL  .......1  vertical delay ball
    RESMP0 ,   // 0x28   RESMP0  ......1.  reset missile 0 to player 0
    RESMP1 ,   // 0x29   RESMP1  ......1.  reset missile 1 to player 1
    HMOVE ,   // 0x2A   HMOVE   <strobe>  apply horizontal motion
    HMCLR ,   // 0x2B   HMCLR   <strobe>  clear horizontal motion registers
    CXCLR ,   // 0x2C   CXCLR   <strobe>  clear collision latches
}


impl TryFrom<u8> for DeviceMemory {
    type Error = ();
    fn try_from(value: u8) -> Result<DeviceMemory,  Self::Error> {
        // Single byte and special multibyte carveout as an exception
        match value {
            0x00 => Ok(DeviceMemory::VSYNC),
            0x01 => Ok(DeviceMemory::VBLANK),
            0x02 => Ok(DeviceMemory::WSYNC),
            0x03 => Ok(DeviceMemory::RSYNC),
            0x04 => Ok(DeviceMemory::NUSIZ0),
            0x05 => Ok(DeviceMemory::NUSIZ1),
            0x06 => Ok(DeviceMemory::COLUP0),
            0x07 => Ok(DeviceMemory::COLUP1),
            0x08 => Ok(DeviceMemory::COLUPF),
            0x09 => Ok(DeviceMemory::COLUBK),
            0x0A => Ok(DeviceMemory::CTRLPF),
            0x0B => Ok(DeviceMemory::REFP0),
            0x0C => Ok(DeviceMemory::REFP1),
            0x0D => Ok(DeviceMemory::PF0),
            0x0E => Ok(DeviceMemory::PF1),
            0x0F => Ok(DeviceMemory::PF2),
            0x10 => Ok(DeviceMemory::RESP0),
            0x11 => Ok(DeviceMemory::RESP1),
            0x12 => Ok(DeviceMemory::RESM0),
            0x13 => Ok(DeviceMemory::RESM1),
            0x14 => Ok(DeviceMemory::RESBL),
            0x15 => Ok(DeviceMemory::AUDC0),
            0x16 => Ok(DeviceMemory::AUDC1),
            0x17 => Ok(DeviceMemory::AUDF0),
            0x18 => Ok(DeviceMemory::AUDF1),
            0x19 => Ok(DeviceMemory::AUDV0),
            0x1A => Ok(DeviceMemory::AUDV1),
            0x1B => Ok(DeviceMemory::GRP0),
            0x1C => Ok(DeviceMemory::GRP1),
            0x1D => Ok(DeviceMemory::ENAM0),
            0x1E => Ok(DeviceMemory::ENAM1),
            0x1F => Ok(DeviceMemory::ENABL),
            0x20 => Ok(DeviceMemory::HMP0),
            0x21 => Ok(DeviceMemory::HMP1),
            0x22 => Ok(DeviceMemory::HMM0),
            0x23 => Ok(DeviceMemory::HMM1),
            0x24 => Ok(DeviceMemory::HMBL),
            0x25 => Ok(DeviceMemory::VDELP0),
            0x26 => Ok(DeviceMemory::VDELP1),
            0x27 => Ok(DeviceMemory::VDELBL),
            0x28 => Ok(DeviceMemory::RESMP0),
            0x29 => Ok(DeviceMemory::RESMP1),
            0x2A => Ok(DeviceMemory::HMOVE),
            0x2B => Ok(DeviceMemory::HMCLR),
            0x2C => Ok(DeviceMemory::CXCLR),
            _ => Err(())
        }
        
    }
}
impl TryFrom<DeviceMemory> for u8 {
    type Error = ();
    fn try_from(device_memory: DeviceMemory) -> Result<u8,  Self::Error> {
        match device_memory {
            DeviceMemory::VSYNC => Ok(0x00),
            DeviceMemory::VBLANK => Ok(0x01),
            DeviceMemory::WSYNC => Ok(0x02),
            DeviceMemory::RSYNC => Ok(0x03),
            DeviceMemory::NUSIZ0 => Ok(0x04),
            DeviceMemory::NUSIZ1 => Ok(0x05),
            DeviceMemory::COLUP0 => Ok(0x06),
            DeviceMemory::COLUP1 => Ok(0x07),
            DeviceMemory::COLUPF => Ok(0x08),
            DeviceMemory::COLUBK => Ok(0x09),
            DeviceMemory::CTRLPF => Ok(0x0A),
            DeviceMemory::REFP0 => Ok(0x0B),
            DeviceMemory::REFP1 => Ok(0x0C),
            DeviceMemory::PF0 => Ok(0x0D),
            DeviceMemory::PF1 => Ok(0x0E),
            DeviceMemory::PF2 => Ok(0x0F),
            DeviceMemory::RESP0 => Ok(0x10),
            DeviceMemory::RESP1 => Ok(0x11),
            DeviceMemory::RESM0 => Ok(0x12),
            DeviceMemory::RESM1 => Ok(0x13),
            DeviceMemory::RESBL => Ok(0x14),
            DeviceMemory::AUDC0 => Ok(0x15),
            DeviceMemory::AUDC1 => Ok(0x16),
            DeviceMemory::AUDF0 => Ok(0x17),
            DeviceMemory::AUDF1 => Ok(0x18),
            DeviceMemory::AUDV0 => Ok(0x19),
            DeviceMemory::AUDV1 => Ok(0x1A),
            DeviceMemory::GRP0 => Ok(0x1B),
            DeviceMemory::GRP1 => Ok(0x1C),
            DeviceMemory::ENAM0 => Ok(0x1D),
            DeviceMemory::ENAM1 => Ok(0x1E),
            DeviceMemory::ENABL => Ok(0x1F),
            DeviceMemory::HMP0 => Ok(0x20),
            DeviceMemory::HMP1 => Ok(0x21),
            DeviceMemory::HMM0 => Ok(0x22),
            DeviceMemory::HMM1 => Ok(0x23),
            DeviceMemory::HMBL => Ok(0x24),
            DeviceMemory::VDELP0 => Ok(0x25),
            DeviceMemory::VDELP1 => Ok(0x26),
            DeviceMemory::VDELBL => Ok(0x27),
            DeviceMemory::RESMP0 => Ok(0x28),
            DeviceMemory::RESMP1 => Ok(0x29),
            DeviceMemory::HMOVE => Ok(0x2A),
            DeviceMemory::HMCLR => Ok(0x2B),
            DeviceMemory::CXCLR => Ok(0x2C),
            _ => Err(())
        }
    }
}
