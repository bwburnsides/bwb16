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

#ruledef cpu16 {
    lui {rd: reg}, {imm: i10} => le(imm[0:0] @ imm[4:2] @ rd`4 @ imm[9:9] @ imm[8:7] @ imm[1:1] @ imm[6:5] @ 0b01)
    mv {rd: reg}, {rs: reg} => le(rs`4 @ rd`4 @ 0b1110 @ 0b0000)
    addi {rd: reg}, {imm: i6} => le(0b00 @ imm[4:3] @ rd`4 @ imm[5:5] @ imm[2:0] @ 0b1000)
}
