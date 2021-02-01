    *= $0000
    brk


    *= $0050

    factor1     .byte 5
    factor2     .byte 20
    SRC         .word $0600
    DST         .word $0200

    *= $0600

    .text "HELLO WORLD BROS\n"

    *= $4000

START
    lda #$1
    bne MULTIPLY
    jmp TOLOWER

INCLUDES

    *= $4050
    .include "mul.asm"
    
    *= $4075
    .include "tolower.asm"


