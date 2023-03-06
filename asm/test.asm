#include "bwb16.asm"

nop
xor r0, r0
xor r1, r1

addi r0, 5
addi r1, 3

add r0, r1
mov r1, r0

nop
nop
nop
nop

xor r0, r0
not r0

nop
nop
nop
nop

lui r0, 0xFF
ldi r0, 0xFFFF