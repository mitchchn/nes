    *= $0000
    brk

    *= $00F0

    factor1     .byte 3
    factor2     .byte 80
    SRC         .word $0105
    DST         .word $0200

    *= $0105
    // .text "leetspeak is for hackers\n"
    .text "HELLO WORLDS\n"

    *= $0600

START
    // jsr MULTIPLY
    // jsr TOLOWER
    jsr ROT13

    brk

INCLUDES

    // *= $0650
    // .include "mul.asm"
    *= $0650
    .include "rot13.asm"
    
    // *= $0675
    // .include "tolower.asm"

    // *= $0675
    // .include "toleet.asm"
