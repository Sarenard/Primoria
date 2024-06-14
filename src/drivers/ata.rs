use crate::system::ports::{
    port_byte_in, port_byte_out,
    port_long_in, port_long_out,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Status {
    BSY = 0x80,
    RDY = 0x40,
    DRQ = 0x08,
    DR  = 0x20,
    ERR = 0x01
}

fn write_sector(LBA: u32, data: &[u32; 512]) {

    unsafe {
        wait_bsy();
        wait_drq();
    }

    unsafe {
        port_byte_out(0x1F6, 0xE0 | ((LBA >> 24) & 0x0F) as u8);
        port_byte_out(0x1F1, 0x00);
        port_byte_out(0x1F2, 0x01);
        port_byte_out(0x1F3, LBA as u8);
        port_byte_out(0x1F4, (LBA >> 8) as u8);
        port_byte_out(0x1F5, (LBA >> 16)  as u8);
        port_byte_out(0x1F7, 0x30);
    }

    unsafe {
        wait_bsy();
        wait_drq();
    }

    for i in 0..128 {
        unsafe {
            port_long_out(0x1F0, data[i]);
        }
    }
}

fn read_sector(LBA: u32, data: &mut [u32; 512]) {
    unsafe {
        wait_bsy();
        wait_drq();
    }

    unsafe {
        port_byte_out(0x1F6, 0xE0 | ((LBA >> 24) & 0x0F) as u8);
        port_byte_out(0x1F1, 0x00);
        port_byte_out(0x1F2, 0x01);
        port_byte_out(0x1F3, LBA as u8);
        port_byte_out(0x1F4, (LBA >> 8) as u8);
        port_byte_out(0x1F5, (LBA >> 16) as u8);
        port_byte_out(0x1F7, 0x20);
    }

    unsafe {
        wait_bsy();
        wait_drq();
    }


    for i in 0..128 {
        unsafe {
            data[i] = port_long_in(0x1F0);
        }
    }
}

unsafe fn wait_bsy() {
    while ((port_byte_in(0x1F7) & Status::BSY as u8) != 0) {}
}

// TODO : investigate why the fuck this is named "drq" and waits for ready
unsafe fn wait_drq() {
    while (!port_byte_in(0x1F7) & (Status::RDY as u8) != 0) {}
}
