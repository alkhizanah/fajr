use lazy_static::lazy_static;

use crate::{
    memory::{align_down, align_up},
    paging::{self, MIN_PAGE_SIZE},
    requests::RSDP_REQUEST,
};

/// System Description Table Header
#[repr(C, packed)]
pub struct SdtHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oemid: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

/// Root System Description Pointer
#[derive(Debug)]
#[repr(C, packed)]
pub struct Rsdp {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oemid: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
    pub length: u32,
    pub xsdt_address: u32,
    pub extra_checksum: u8,
    pub reserved: [u8; 3],
}

/// Root System Description Table
#[repr(C, packed)]
pub struct Rsdt {
    pub header: SdtHeader,
    pub entries: [u32; 256],
}

/// Fixed ACPI Description Table
#[repr(C, packed)]
pub struct Fadt {
    pub header: SdtHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,
    pub reserved_1: u32,
    pub preferred_power_management_profile: u8,
    pub sci_interrupt: u16,
    pub smi_command_port: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_control: u8,
    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
    pub gpe0_block: u32,
    pub gpe1_block: u32,
    pub pm1_event_length: u8,
    pub pm1_control_length: u8,
    pub pm2_control_length: u8,
    pub pm_timer_length: u8,
    pub gpe0_length: u8,
    pub gpe1_length: u8,
    pub gpe1_base: u8,
    pub cstate_control: u8,
    pub worst_c2_latency: u16,
    pub worst_c3_latency: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alarm: u8,
    pub month_alarm: u8,
    pub century: u8,
    pub reserved_2: u16,
    pub reserved_3: u8,
    pub flags: u32,
    pub reset_reg: [u8; 12],
    pub reset_value: u8,
    pub reserved_4: u16,
    pub reserved_5: u8,
}

/// Differentiated Description Table
pub struct Dsdt {
    pub header: SdtHeader,
}

/// Multiple APIC Description Table
#[repr(C, packed)]
pub struct Madt {
    pub header: SdtHeader,
    pub lapic_address: u32,
    pub flags: u32,
}

impl Madt {
    pub fn get_ioapic_base(&self) -> usize {
        let mut madt = self as *const _ as usize;

        madt += size_of::<Madt>();

        loop {
            unsafe {
                let madt_entry_pointer = madt as *const u8;
                let madt_entry_type = *madt_entry_pointer;
                let madt_entry_length = (*madt_entry_pointer.add(1)) as usize;

                if madt_entry_type == 1 {
                    let ioapic_phys_base = u32::from_le_bytes([
                        *madt_entry_pointer.add(4),
                        *madt_entry_pointer.add(5),
                        *madt_entry_pointer.add(6),
                        *madt_entry_pointer.add(7),
                    ]) as usize;

                    let ioapic_virt_base = paging::offset(ioapic_phys_base);

                    paging::get_active_table()
                        .map(
                            align_down(ioapic_virt_base, MIN_PAGE_SIZE),
                            align_down(ioapic_phys_base, MIN_PAGE_SIZE),
                        )
                        .set_writable(true)
                        .set_write_through(true)
                        .set_cachability(false);

                    return ioapic_virt_base;
                }

                madt += madt_entry_length;
            }
        }
    }
}

pub struct Acpi<'a> {
    pub rsdt: &'a Rsdt,
    pub fadt: Option<&'a Fadt>,
    pub dsdt: Option<&'a Dsdt>,
    pub madt: Option<&'a Madt>,
}

fn unmap<T>(object: &'static T) {
    let virt = object as *const _ as usize;

    let mut aligned_virt = align_down(virt, MIN_PAGE_SIZE);
    let aligned_virt_end = align_up(virt + size_of::<T>(), MIN_PAGE_SIZE);

    let page_table = paging::get_active_table();

    while aligned_virt < aligned_virt_end {
        page_table.unmap(aligned_virt);

        aligned_virt += MIN_PAGE_SIZE;
    }
}

fn map<T>(phys: usize) -> &'static T {
    let mut aligned_phys = align_down(phys, MIN_PAGE_SIZE);

    let virt = paging::offset(phys);

    let mut aligned_virt = align_down(virt, MIN_PAGE_SIZE);
    let aligned_virt_end = align_up(virt + size_of::<T>(), MIN_PAGE_SIZE);

    let page_table = paging::get_active_table();

    while aligned_virt < aligned_virt_end {
        page_table.map(aligned_virt, aligned_phys);

        aligned_virt += MIN_PAGE_SIZE;
        aligned_phys += MIN_PAGE_SIZE;
    }

    unsafe { &*(virt as *const _) }
}

lazy_static! {
    pub static ref ACPI: Acpi<'static> = unsafe {
        let rsdp_address = RSDP_REQUEST
            .get_response()
            .expect("could not ask limine to get rsdp")
            .address();

        let rsdp = map::<Rsdp>(rsdp_address);

        if rsdp.signature != "RSD PTR ".as_bytes() {
            panic!("bad rsdp signature");
        }

        match rsdp.revision {
            0 => {
                let rsdp_checksum: usize =
                    core::mem::transmute::<&Rsdp, &[u8; size_of::<Rsdp>()]>(rsdp)
                        .iter()
                        .map(|&x| x as usize)
                        .sum();

                if (rsdp_checksum & 0xff) != 0 {
                    panic!("bad rsdp checksum");
                }

                let rsdt_address = rsdp.rsdt_address as usize;

                let rsdt = map::<Rsdt>(rsdt_address);

                let mut fadt = None;
                let mut dsdt = None;
                let mut madt = None;

                let rsdt_entry_count = (rsdt.header.length as usize - size_of::<SdtHeader>()) / 4;

                for i in 0..rsdt_entry_count {
                    let rsdt_entry = rsdt.entries[i] as usize;

                    let sdt_header = map::<SdtHeader>(rsdt_entry);

                    let rsdt_entry_signature = sdt_header.signature;

                    if rsdt_entry_signature == "FACP".as_bytes() {
                        fadt = Some({
                            let fadt = map::<Fadt>(rsdt_entry);

                            let dsdt_address = fadt.dsdt as usize;

                            dsdt = Some({
                                let dsdt = map::<Dsdt>(dsdt_address);

                                if dsdt.header.signature != "DSDT".as_bytes() {
                                    panic!("bad dsdt signature");
                                }

                                dsdt
                            });

                            fadt
                        });
                    } else if rsdt_entry_signature == "APIC".as_bytes() {
                        madt = Some(map(rsdt_entry));
                    }
                }

                unmap(rsdp);

                Acpi {
                    rsdt,
                    fadt,
                    dsdt,
                    madt,
                }
            }

            _ => panic!("unsupported version of acpi"),
        }
    };
}
