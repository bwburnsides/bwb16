#bits 16

#subruledef reg {
    r0  => 0
    r1  => 1
    r2  => 2
    r3  => 3
    r4  => 4
    r5  => 5
    r6  => 6
    r7  => 7
    r8  => 8
    r9  => 9
    r10 => 10
    r11 => 11
    r12 => 12
    r13 => 13
    r14 => 14
    r15 => 15
}

#fn alu2(func, rd, rs)    => le(rs`4     @ rd`4      @ func    @ 0b0000                                             )
#fn alu1(func, rd)        => le(func     @ rd`4      @ 0b1111  @ 0b0000                                             )
#fn imm4(func, r, im)     => le(func     @ r`4       @ im[3:3] @ im[2:0]   @ 0b1000                                 )
#fn imm6(func, r, im)     => le(func     @ im[4:3]   @ r`4     @ im[5:5]   @ im[2:0]  @ 0b1000                      )
#fn ctrl(func)            => le(0b1111   @ func      @ 0b1111  @ 0b0000                                             )
#fn mem(func, rd, rs, im) => le(rs`4     @ rd`4      @ im[3:3] @ im[2:0]   @ func                                   )
#fn jump(cond, im)        => le(im[6:3]  @ im[8:8]   @ cond    @ im[9:9]   @ im[2:1]   @ im[7:7]  @ 0b1110          )
#fn uimm(func, rd, im)    => le(im[6:6]  @ im[10:8]  @ rd`4    @ im[15:15] @ im[14:13] @ im[7:7]  @ im[12:11] @ func)

#ruledef cpu16 {
    add {rd: reg}, {rs: reg}  => alu2(0b0000, rd, rs)
    sub {rd: reg}, {rs: reg}  => alu2(0b0001, rd, rs)
    addc {rd: reg}, {rs: reg} => alu2(0b0010, rd, rs)
    subb {rd: reg}, {rs: reg} => alu2(0b0011, rd, rs)
    and {rd: reg}, {rs: reg}  => alu2(0b0100, rd, rs)
    or {rd: reg}, {rs: reg}   => alu2(0b0101, rd, rs)
    xor {rd: reg}, {rs: reg}  => alu2(0b0110, rd, rs)
    shl {rd: reg}, {rs: reg}  => alu2(0b0111, rd, rs)
    lsr {rd: reg}, {rs: reg}  => alu2(0b1000, rd, rs)
    asr {rd: reg}, {rs: reg}  => alu2(0b1001, rd, rs)
    cmp {rd: reg}, {rs: reg}  => alu2(0b1101, rd, rs)
    mov {rd: reg}, {rs: reg}  => alu2(0b1110, rd, rs)

    not {rd: reg}  => alu1(0b0000, rd)
    neg {rd: reg}  => alu1(0b0001, rd)
    negb {rd: reg} => alu1(0b0010, rd)

    nop   => ctrl(0b0000)
    break => ctrl(0b0001)
    halt  => ctrl(0b0010)
    error => ctrl(0b0011)
    sys   => ctrl(0b0100)
    sret  => ctrl(0b0101)

    addi {rd: reg}, {im: i6} => imm6(0b00, rd, im)
    cmpi {rd: reg}, {im: i6} => imm6(0b01, rd, im)
    jal {rs: reg}, {im: i6}  => imm6(0b10, rs, im)

    shli {rd: reg}, {im: i4} => imm4(0b1100, rd, im)
    lsri {rd: reg}, {im: i4} => imm4(0b1101, rd, im)
    asri {rd: reg}, {im: i4} => imm4(0b1110, rd, im)
    andi {rd: reg}, {im: i4} => imm4(0b1111, rd, im)

    ld {rd: reg}, {rs: reg}, {im: i4}   => mem(0b0010, rd, rs, im)
    ld8 {rd: reg}, {rs: reg}, {im: i4}  => mem(0b1010, rd, rs, im)
    st {rs: reg}, {im: i4}, {rd: reg}   => mem(0b0100, rd, rs, im)
    st8 {rs: reg}, {im: i4}, {rd: reg}  => mem(0b1100, rd, rs, im)
    ld8s {rd: reg}, {rs: reg}, {im: i4} => mem(0b0110, rd, rs, im)

    br.eq {im: i9}  => jump(0b000, im)
    br.ne {im: i9}  => jump(0b001, im)
    br.lt {im: i9}  => jump(0b010, im)
    br.ge {im: i9}  => jump(0b011, im)
    br.lts {im: i9} => jump(0b100, im)
    br.ges {im: i9} => jump(0b101, im)
    jr {im: i9}     => jump(0b110, im)
    jral {im: i9}   => jump(0b111, im)

    lui {rd: reg}, {im: i10}   => uimm(0b01, rd, im)
    auipc {rd: reg}, {im: i10} => uimm(0b11, rd, im)
}

#ruledef pseudo {
    ldi {rd: reg}, {im: i16} => asm {
        lui {rd}, im[15:6]
        addi {rd}, im[5:0]
    }
}