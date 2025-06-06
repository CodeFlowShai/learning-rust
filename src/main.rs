use std::{env, io::Read};

struct Cpu {
    ax: u16,
    bx: u16,
    cx: u16,
    dx: u16,

    sp: u16,
    bp: u16,
    si: u16,
    di: u16,

    cs: u16,
    ip: u16,
    flags: u16,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        println!("Usage: {} <bootloader.bin> [--debug]", args[0]);
        return;
    }

    let filename = &args[1];
    let debug = args.len() == 3 && args[2] == "--debug";

    let mut mem: [u8; 1024 * 1024] = [0; 1024 * 1024];
    let mut boot_sector: Vec<u8> = Vec::new();

    // Read file
    {
        let file = std::fs::File::open(filename);
        if file.is_err() {
            println!("Error: {}", file.err().unwrap());
            return;
        }
        let mut file = file.unwrap();
        file.read_to_end(&mut boot_sector).unwrap();
    }

    if boot_sector.len() < 512 {
        println!("Error: Boot sector is less than 512 bytes");
        return;
    }

    if boot_sector[510] != 0x55 || boot_sector[511] != 0xAA {
        println!("Error: Invalid boot sector signature");
        return;
    }

    mem[0x7C00..0x7C00 + 512].copy_from_slice(&boot_sector[..512]);

    let mut cpu = Cpu {
        ax: 0, bx: 0, cx: 0, dx: 0,
        sp: 0, bp: 0, si: 0, di: 0,
        cs: 0x0000,
        ip: 0x7C00,
        flags: 0,
    };

    execute_loop(&mut cpu, &mut mem, debug);
}

fn execute_loop(cpu: &mut Cpu, mem: &mut [u8], debug: bool) {
    loop {
        let ok = execute_inst(cpu, mem, debug);
        if !ok {
            break;
        }
    }
}

fn log(debug: bool, msg: &str) {
    if debug {
        println!("\n[{}]\n", msg);
    }
}

fn execute_inst(cpu: &mut Cpu, mem: &mut [u8], debug: bool) -> bool {
    let cs_ip = ((cpu.cs as u32) << 4) + (cpu.ip as u32);
    let opcode = mem[cs_ip as usize];

    match opcode {
        0xEB => { // JMP rel8
            let offset = mem[(cs_ip + 1) as usize] as i8;
            let new_ip = cpu.ip.wrapping_add(2).wrapping_add(offset as u16);
            log(debug, &format!("JMP short to IP = {:04X}", new_ip));
            cpu.ip = new_ip;
        }

        0xCD => { // INT instruction
            let int_num = mem[(cs_ip + 1) as usize];

            match int_num {
                0x10 => {
                    let ah = (cpu.ax >> 8) as u8;
                    let al = (cpu.ax & 0xFF) as u8;

                    if ah == 0x0E {
                        log(debug, &format!("INT 10h: print '{}'", al as char));
                        print!("{}", al as char);
                    } else {
                        log(debug, &format!("INT 10h AH={:02X} — unhandled subfunction", ah));
                    }
                }
                _ => {
                    log(debug, &format!("Unhandled interrupt INT {:02X}", int_num));
                }
            }

            cpu.ip = cpu.ip.wrapping_add(2);
        }

        0xB0 => { // MOV AL, imm8
            let imm = mem[(cs_ip + 1) as usize];
            cpu.ax = (cpu.ax & 0xFF00) | imm as u16;
            log(debug, &format!("MOV AL, {:02X}", imm));
            cpu.ip = cpu.ip.wrapping_add(2);
        }

        0xB4 => { // MOV AH, imm8
            let imm = mem[(cs_ip + 1) as usize];
            cpu.ax = (cpu.ax & 0x00FF) | ((imm as u16) << 8);
            log(debug, &format!("MOV AH, {:02X}", imm));
            cpu.ip = cpu.ip.wrapping_add(2);
        }

        0xF4 => { // HLT
            log(debug, "HLT encountered. Halting execution.");
            println!();
            return false;
        }

        0x90 => { // NOP
            log(debug, "NOP");
            cpu.ip = cpu.ip.wrapping_add(1);
        }

        _ => {
            println!(
                "Unknown opcode {:02X} at CS:IP {:04X}:{:04X}",
                opcode, cpu.cs, cpu.ip
            );
            return false;
        }
    }

    true
}
