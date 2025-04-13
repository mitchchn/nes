.zeropage
LO: .res 2
HI: .res 2
TMP: .res 2

.segment "CODE"

RESTART:
COLD_START:    
    LDA #0
    JSR PRINTDIG
    LDA #','
    JSR MONCOUT
    LDA #1
    JSR PRINTDIG
    LDA #','
    JSR MONCOUT
    LDY #15
    ; set up zero page
    LDA #0
    STA LO
    LDA #1
    STA HI    
FIBONACCI:
    ; F(n) = F(n-1) + F(n-2)
    LDA LO
    SED
    CLC
    ADC HI
    CLD
    STA TMP
    JSR PRINTNUM
    LDA #','
    JSR MONCOUT
    LDA HI
    STA LO
    LDA TMP
    STA HI   
    DEY
    BNE FIBONACCI
    JMP DONE
PRINTNUM:
    TAX
    AND #$F0
    BEQ PRINTDIG0
    LSR
    LSR
    LSR
    LSR
    JSR PRINTDIG
    TXA
    AND #$0F
    JSR PRINTDIG
    RTS
PRINTDIG0:
    TXA
PRINTDIG:
    CLC
    ADC #$30 ; ASCII digit 0
    JSR MONCOUT
    RTS
DONE:
    NOP
    JMP DONE

.include "minmon.asm"
