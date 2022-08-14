*= $0000
brk

; PUTCHAR ($FF00)
*= $E000
PUTCHAR_     sta $200
rts

; BIOS API jump table
*= $FF00
PUTCHAR     jmp PUTCHAR_

*= $FFFF
brk