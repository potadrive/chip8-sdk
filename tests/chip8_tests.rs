use chip8_emulator::chip8::Chip8;

#[test]
fn test_initial_state() {
    let chip8 = Chip8::new();
    assert_eq!(chip8.pc, 0x200);
    assert_eq!(chip8.i, 0);
    assert_eq!(chip8.sp, 0);
    assert_eq!(chip8.dt, 0);
    assert_eq!(chip8.st, 0);
    // Все пиксели должны быть фонового цвета
    for &pixel in &chip8.display.buffer {
        assert_eq!(pixel, 0xAAAAAA);
    }
}

#[test]
fn test_load_fontset() {
    let mut chip8 = Chip8::new();
    chip8.load_fontset(&Chip8::SYSFONT);
    for i in 0..Chip8::SYSFONT.len() {
        assert_eq!(chip8.mem[Chip8::SYSFONT_START_ADDR + i], Chip8::SYSFONT[i]);
    }
}

#[test]
fn test_00e0_clear_screen() {
    let mut chip8 = Chip8::new();
    // Сначала что-то нарисуем (DRW V0, V1, 1)
    chip8.i = 0x300;
    chip8.mem[0x300] = 0xFF;
    chip8.v[0] = 10;
    chip8.v[1] = 10;
    chip8.mem[0x200] = 0xD0;
    chip8.mem[0x201] = 0x11;
    chip8.do_instruction_cycle();
    let idx = (10 * 64 + 10) as usize;
    assert_eq!(chip8.display.buffer[idx], 0x0); // должен стать цветом переднего плана

    // CLS
    chip8.pc = 0x200;
    chip8.mem[0x200] = 0x00;
    chip8.mem[0x201] = 0xE0;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.display.buffer[idx], 0xAAAAAA); // вернулся фон
}

#[test]
fn test_1nnn_jump() {
    let mut chip8 = Chip8::new();
    chip8.mem[0x200] = 0x13;
    chip8.mem[0x201] = 0x45;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x345);
}

#[test]
fn test_2nnn_call_and_00ee_return() {
    let mut chip8 = Chip8::new();
    // CALL 0x300
    chip8.mem[0x200] = 0x23;
    chip8.mem[0x201] = 0x00;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x300);
    assert_eq!(chip8.sp, 1);
    assert_eq!(chip8.stack[0], 0x202);

    // RET
    chip8.mem[0x300] = 0x00;
    chip8.mem[0x301] = 0xEE;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x202);
    assert_eq!(chip8.sp, 0);
}

#[test]
fn test_3xnn_skip_if_equal() {
    let mut chip8 = Chip8::new();
    chip8.v[1] = 0x42;
    // SE V1, 0x42 – должен пропустить
    chip8.mem[0x200] = 0x31;
    chip8.mem[0x201] = 0x42;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x204);

    // Не равно – не пропускаем
    chip8.pc = 0x200;
    chip8.mem[0x201] = 0x41;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x202);
}

#[test]
fn test_4xnn_skip_if_not_equal() {
    let mut chip8 = Chip8::new();
    chip8.v[2] = 0x42;
    // SNE V2, 0x43 – должно пропустить (не равно)
    chip8.mem[0x200] = 0x42;
    chip8.mem[0x201] = 0x43;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x204);

    // равно – не пропускаем
    chip8.pc = 0x200;
    chip8.mem[0x201] = 0x42;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x202);
}

#[test]
fn test_5xy0_skip_if_vx_eq_vy() {
    let mut chip8 = Chip8::new();
    chip8.v[3] = 0x10;
    chip8.v[4] = 0x10;
    chip8.mem[0x200] = 0x53;
    chip8.mem[0x201] = 0x40;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x204);

    chip8.v[4] = 0x20;
    chip8.pc = 0x200;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x202);
}

#[test]
fn test_6xnn_load_register() {
    let mut chip8 = Chip8::new();
    chip8.mem[0x200] = 0x6A;
    chip8.mem[0x201] = 0x42;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[0xA], 0x42);
}

#[test]
fn test_7xnn_add() {
    let mut chip8 = Chip8::new();
    chip8.v[5] = 0x10;
    chip8.mem[0x200] = 0x75;
    chip8.mem[0x201] = 0x20;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[5], 0x30);

    // Переполнение
    chip8.v[5] = 0xF0;
    chip8.pc = 0x200;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[5], 0x10);
}

#[test]
fn test_8xy0_load() {
    let mut chip8 = Chip8::new();
    chip8.v[6] = 0xAB;
    chip8.mem[0x200] = 0x86;
    chip8.mem[0x201] = 0x70;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[7], 0xAB);
}

#[test]
fn test_8xy1_or() {
    let mut chip8 = Chip8::new();
    chip8.v[8] = 0xF0;
    chip8.v[9] = 0x0F;
    chip8.mem[0x200] = 0x88;
    chip8.mem[0x201] = 0x91;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[8], 0xFF);
    assert_eq!(chip8.v[0xF], 0);
}

#[test]
fn test_8xy2_and() {
    let mut chip8 = Chip8::new();
    chip8.v[8] = 0xF0;
    chip8.v[9] = 0x0F;
    chip8.mem[0x200] = 0x88;
    chip8.mem[0x201] = 0x92;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[8], 0x00);
    assert_eq!(chip8.v[0xF], 0);
}

#[test]
fn test_8xy3_xor() {
    let mut chip8 = Chip8::new();
    chip8.v[8] = 0xF0;
    chip8.v[9] = 0x0F;
    chip8.mem[0x200] = 0x88;
    chip8.mem[0x201] = 0x93;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[8], 0xFF);
    assert_eq!(chip8.v[0xF], 0);
}

#[test]
fn test_8xy4_add_with_carry() {
    let mut chip8 = Chip8::new();
    chip8.v[0] = 0x01;
    chip8.v[1] = 0x01;
    chip8.mem[0x200] = 0x80;
    chip8.mem[0x201] = 0x14;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[0], 0x02);
    assert_eq!(chip8.v[0xF], 0);

    // С переносом
    chip8.v[0] = 0xFF;
    chip8.v[1] = 0x01;
    chip8.pc = 0x200;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[0], 0x00);
    assert_eq!(chip8.v[0xF], 1);
}

#[test]
fn test_8xy5_sub_borrow() {
    let mut chip8 = Chip8::new();
    chip8.v[0] = 0x05;
    chip8.v[1] = 0x03;
    chip8.mem[0x200] = 0x80;
    chip8.mem[0x201] = 0x15;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[0], 0x02);
    assert_eq!(chip8.v[0xF], 1);

    // С заимствованием
    chip8.v[0] = 0x02;
    chip8.v[1] = 0x05;
    chip8.pc = 0x200;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[0], 0xFD);
    assert_eq!(chip8.v[0xF], 0);
}

#[test]
fn test_8xy6_shr() {
    let mut chip8 = Chip8::new();
    chip8.v[2] = 0x05;
    chip8.mem[0x200] = 0x82;
    chip8.mem[0x201] = 0x26;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[2], 0x02);
    assert_eq!(chip8.v[0xF], 1);

    chip8.v[2] = 0x04;
    chip8.pc = 0x200;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[2], 0x02);
    assert_eq!(chip8.v[0xF], 0);
}

#[test]
fn test_8xy7_subn() {
    let mut chip8 = Chip8::new();
    chip8.v[0] = 0x02;
    chip8.v[1] = 0x05;
    chip8.mem[0x200] = 0x80;
    chip8.mem[0x201] = 0x17;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[0], 0x03);
    assert_eq!(chip8.v[0xF], 1);

    chip8.v[0] = 0x05;
    chip8.v[1] = 0x02;
    chip8.pc = 0x200;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[0], 0xFD);
    assert_eq!(chip8.v[0xF], 0);
}

#[test]
fn test_8xye_shl() {
    let mut chip8 = Chip8::new();
    chip8.v[3] = 0x85;
    chip8.mem[0x200] = 0x83;
    chip8.mem[0x201] = 0x3E;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[3], 0x0A);
    assert_eq!(chip8.v[0xF], 1);

    chip8.v[3] = 0x45;
    chip8.pc = 0x200;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[3], 0x8A);
    assert_eq!(chip8.v[0xF], 0);
}

#[test]
fn test_9xy0_skip_if_not_equal() {
    let mut chip8 = Chip8::new();
    chip8.v[0] = 0x10;
    chip8.v[1] = 0x20;
    chip8.mem[0x200] = 0x90;
    chip8.mem[0x201] = 0x10;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x204);

    chip8.v[1] = 0x10;
    chip8.pc = 0x200;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x202);
}

#[test]
fn test_annn_load_i() {
    let mut chip8 = Chip8::new();
    chip8.mem[0x200] = 0xA3;
    chip8.mem[0x201] = 0x45;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.i, 0x345);
}

#[test]
fn test_bnnn_jump_v0() {
    let mut chip8 = Chip8::new();
    chip8.v[0] = 0x10;
    chip8.mem[0x200] = 0xB3;
    chip8.mem[0x201] = 0x45;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x345 + 0x10);
}

#[test]
fn test_cxnn_random() {
    let mut chip8 = Chip8::new();
    chip8.mem[0x200] = 0xC0;
    chip8.mem[0x201] = 0x0F;
    chip8.do_instruction_cycle();
    let val = chip8.v[0];
    assert!(val <= 0x0F);
    assert_eq!(val & !0x0F, 0);
}

#[test]
fn test_dxyn_draw() {
    let mut chip8 = Chip8::new();
    chip8.i = 0x300;
    chip8.mem[0x300] = 0b11110000;
    chip8.mem[0x301] = 0b11001100;
    chip8.mem[0x302] = 0b10101010;
    chip8.v[0] = 10;
    chip8.v[1] = 5;
    chip8.mem[0x200] = 0xD0;
    chip8.mem[0x201] = 0x16; // n=3
    chip8.do_instruction_cycle();

    let bg = 0xAAAAAA;
    let fg = 0x0;
    let idx = (5 * 64 + 10) as usize;
    assert_eq!(chip8.display.buffer[idx], fg);
    assert_eq!(chip8.display.buffer[5 * 64 + 11], fg);
    assert_eq!(chip8.display.buffer[5 * 64 + 14], bg);
    assert_eq!(chip8.v[0xF], 0);

    // Повтор – должны стереться и установить VF
    chip8.pc = 0x200;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.display.buffer[idx], bg);
    assert_eq!(chip8.v[0xF], 1);
}

#[test]
fn test_ex9e_skip_if_key_pressed() {
    let mut chip8 = Chip8::new();
    chip8.v[0] = 0xA;
    chip8.keyboard[0xA] = true;
    chip8.mem[0x200] = 0xE0;
    chip8.mem[0x201] = 0x9E;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x204);

    chip8.keyboard[0xA] = false;
    chip8.pc = 0x200;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x202);
}

#[test]
fn test_exa1_skip_if_key_not_pressed() {
    let mut chip8 = Chip8::new();
    chip8.v[0] = 0xA;
    chip8.keyboard[0xA] = false;
    chip8.mem[0x200] = 0xE0;
    chip8.mem[0x201] = 0xA1;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x204);

    chip8.keyboard[0xA] = true;
    chip8.pc = 0x200;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x202);
}

#[test]
fn test_fx07_load_delay_timer() {
    let mut chip8 = Chip8::new();
    chip8.dt = 0x42;
    chip8.mem[0x200] = 0xF0;
    chip8.mem[0x201] = 0x07;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.v[0], 0x42);
}

#[test]
fn test_fx0a_wait_key() {
    let mut chip8 = Chip8::new();
    chip8.mem[0x200] = 0xF0;
    chip8.mem[0x201] = 0x0A;
    // Нет нажатой клавиши – повтор
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x200);
    // Нажимаем клавишу 5
    chip8.keyboard[5] = true;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.pc, 0x202);
    assert_eq!(chip8.v[0], 5);
}

#[test]
fn test_fx15_set_delay_timer() {
    let mut chip8 = Chip8::new();
    chip8.v[0] = 0x42;
    chip8.mem[0x200] = 0xF0;
    chip8.mem[0x201] = 0x15;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.dt, 0x42);
}

#[test]
fn test_fx18_set_sound_timer() {
    let mut chip8 = Chip8::new();
    chip8.v[0] = 0x42;
    chip8.mem[0x200] = 0xF0;
    chip8.mem[0x201] = 0x18;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.st, 0x42);
}

#[test]
fn test_fx1e_add_i_and_vx() {
    let mut chip8 = Chip8::new();
    chip8.i = 0x100;
    chip8.v[0] = 0x20;
    chip8.mem[0x200] = 0xF0;
    chip8.mem[0x201] = 0x1E;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.i, 0x120);
}

#[test]
fn test_fx29_load_sprite() {
    let mut chip8 = Chip8::new();
    chip8.load_fontset(&Chip8::SYSFONT);
    chip8.v[0] = 0xA;
    chip8.mem[0x200] = 0xF0;
    chip8.mem[0x201] = 0x29;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.i, Chip8::SYSFONT_START_ADDR + 5 * 10);
}

#[test]
fn test_fx33_store_bcd() {
    let mut chip8 = Chip8::new();
    chip8.i = 0x300;
    chip8.v[0] = 123;
    chip8.mem[0x200] = 0xF0;
    chip8.mem[0x201] = 0x33;
    chip8.do_instruction_cycle();
    assert_eq!(chip8.mem[0x300], 1);
    assert_eq!(chip8.mem[0x301], 2);
    assert_eq!(chip8.mem[0x302], 3);
}

#[test]
fn test_fx55_store_registers() {
    let mut chip8 = Chip8::new();
    chip8.i = 0x400;
    for i in 0..=5 {
        chip8.v[i] = i as u8;
    }
    chip8.mem[0x200] = 0xF5;
    chip8.mem[0x201] = 0x55;
    chip8.do_instruction_cycle();
    for i in 0..=5 {
        assert_eq!(chip8.mem[0x400 + i], i as u8);
    }
    assert_eq!(chip8.i, 0x400 + 6);
}

#[test]
fn test_fx65_load_registers() {
    let mut chip8 = Chip8::new();
    chip8.i = 0x400;
    for i in 0..=5 {
        chip8.mem[0x400 + i] = (i + 10) as u8;
    }
    chip8.mem[0x200] = 0xF5;
    chip8.mem[0x201] = 0x65;
    chip8.do_instruction_cycle();
    for i in 0..=5 {
        assert_eq!(chip8.v[i], (i + 10) as u8);
    }
    assert_eq!(chip8.i, 0x400 + 6);
}