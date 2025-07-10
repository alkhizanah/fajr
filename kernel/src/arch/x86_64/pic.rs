use super::io_ports::outb;

pub const MASTER_COMMAND_PORT: u16 = 0x20;
pub const MASTER_DATA_PORT: u16 = MASTER_COMMAND_PORT + 0x01;

pub const SLAVE_COMMAND_PORT: u16 = 0xA0;
pub const SLAVE_DATA_PORT: u16 = SLAVE_COMMAND_PORT + 0x01;

pub fn disable() {
    outb(MASTER_DATA_PORT, 0xff);
    outb(SLAVE_DATA_PORT, 0xff);
}
